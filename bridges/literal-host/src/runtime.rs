use crate::{BridgeError, Limits, Result, module_digest};
use std::sync::Arc;
use wasmi::{
    CompilationMode, Config, EnforcedLimits, Engine, Instance, Linker, Module, Store, StoreLimits,
    StoreLimitsBuilder,
};
use wow_render_contract::{ABI_VERSION, MAX_RESPONSE_BYTES, Request, Response};
const MAX_MODULE_BYTES: usize = 8 * 1024 * 1024;

struct Compiled {
    engine: Engine,
    module: Module,
    digest: String,
    limits: Limits,
}
/// Immutable compiled generation. Clones share code, never guest mutable memory.
#[derive(Clone)]
pub struct ModuleHandle(Arc<Compiled>);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub module_sha256: String,
    pub request_sha256: String,
    pub response_sha256: String,
    pub text: String,
    /// Metering covers the complete successful call, including ABI/buffer work.
    /// Failed calls have no success receipt and are not included in these values.
    pub limits: Limits,
    pub fuel_consumed: u64,
    pub memory_bytes: usize,
}
impl ModuleHandle {
    /// The expected digest must come from caller-approved metadata. A matching
    /// hash checks identity only; this is not signing, distribution or provenance.
    pub fn load(bytes: &[u8], expected_digest: &str, limits: Limits) -> Result<Self> {
        limits.validate()?;
        if bytes.len() > MAX_MODULE_BYTES || !bytes.starts_with(b"\0asm\x01\0\0\0") {
            return Err(BridgeError::ModuleSize);
        }
        let digest = module_digest(bytes);
        if digest != expected_digest {
            return Err(BridgeError::DigestMismatch);
        }
        let mut config = Config::default();
        config
            .consume_fuel(true)
            .compilation_mode(CompilationMode::Eager)
            .allow_start_fn(false)
            .wasm_multi_memory(false)
            .set_max_recursion_depth(256)
            .set_max_stack_height(1024 * 1024)
            .enforced_limits(EnforcedLimits::strict());
        let engine = Engine::new(&config);
        let module = Module::new(&engine, bytes).map_err(|_| BridgeError::InvalidModule)?;
        if module.imports().next().is_some() {
            return Err(BridgeError::ImportsDenied);
        }
        let handle = Self(Arc::new(Compiled {
            engine,
            module,
            digest,
            limits,
        }));
        // Validate all required export types and ABI before making it selectable.
        handle.instantiate()?;
        Ok(handle)
    }
    pub fn digest(&self) -> &str {
        &self.0.digest
    }
    pub fn limits(&self) -> Limits {
        self.0.limits
    }
    fn instantiate(&self) -> Result<(Store<StoreLimits>, Instance)> {
        let limits = StoreLimitsBuilder::new()
            .memory_size(self.0.limits.memory_bytes)
            .memories(1)
            .instances(1)
            .tables(1)
            .table_elements(16_384)
            .trap_on_grow_failure(true)
            .build();
        let mut store = Store::new(&self.0.engine, limits);
        store.limiter(|state| state);
        store
            .set_fuel(self.0.limits.fuel)
            .map_err(|_| BridgeError::InvalidLimits)?;
        let linker = Linker::<StoreLimits>::new(&self.0.engine);
        let instance = linker
            .instantiate_and_start(&mut store, &self.0.module)
            .map_err(execution_error)?;
        let abi = instance
            .get_typed_func::<(), i32>(&store, "wow_abi_version")
            .map_err(|_| BridgeError::IncompatibleAbi)?;
        if abi.call(&mut store, ()).map_err(execution_error)? != ABI_VERSION {
            return Err(BridgeError::IncompatibleAbi);
        }
        instance
            .get_typed_func::<i32, i32>(&store, "wow_request_buffer")
            .map_err(|_| BridgeError::IncompatibleAbi)?;
        for name in ["wow_render", "wow_response_ptr", "wow_response_len"] {
            instance
                .get_typed_func::<(), i32>(&store, name)
                .map_err(|_| BridgeError::IncompatibleAbi)?;
        }
        instance
            .get_memory(&store, "memory")
            .ok_or(BridgeError::IncompatibleAbi)?;
        Ok((store, instance))
    }
    pub fn render(&self, request: &Request) -> Result<Receipt> {
        let input = request.encode().map_err(BridgeError::Input)?;
        let (mut store, instance) = self.instantiate()?;
        let memory = instance
            .get_memory(&store, "memory")
            .ok_or(BridgeError::IncompatibleAbi)?;
        let allocate = instance
            .get_typed_func::<i32, i32>(&store, "wow_request_buffer")
            .map_err(|_| BridgeError::IncompatibleAbi)?;
        let pointer = allocate
            .call(&mut store, input.len() as i32)
            .map_err(execution_error)?;
        if pointer < 0 {
            return Err(BridgeError::InvalidRange);
        }
        memory
            .write(&mut store, pointer as usize, &input)
            .map_err(|_| BridgeError::InvalidRange)?;
        let invoke = |store: &mut Store<StoreLimits>, name: &str| -> Result<i32> {
            instance
                .get_typed_func::<(), i32>(&*store, name)
                .map_err(|_| BridgeError::IncompatibleAbi)?
                .call(store, ())
                .map_err(execution_error)
        };
        if invoke(&mut store, "wow_render")? != 0 {
            return Err(BridgeError::ExecutionFailed);
        }
        let pointer = invoke(&mut store, "wow_response_ptr")?;
        let length = invoke(&mut store, "wow_response_len")?;
        if pointer < 0 || length < 0 || length as usize > MAX_RESPONSE_BYTES {
            return Err(BridgeError::InvalidRange);
        }
        // Bound/validate the guest range before allocating a host output buffer.
        let end = (pointer as usize)
            .checked_add(length as usize)
            .ok_or(BridgeError::InvalidRange)?;
        if memory.data(&store).get(pointer as usize..end).is_none() {
            return Err(BridgeError::InvalidRange);
        }
        let mut output = vec![0; length as usize];
        memory
            .read(&store, pointer as usize, &mut output)
            .map_err(|_| BridgeError::InvalidRange)?;
        let response = Response::decode(&output, request.max_output_bytes)
            .map_err(|_| BridgeError::InvalidResponse)?;
        let text = response.result.map_err(BridgeError::Render)?;
        Ok(Receipt {
            module_sha256: self.0.digest.clone(),
            request_sha256: module_digest(&input),
            response_sha256: module_digest(&output),
            text,
            limits: self.0.limits,
            fuel_consumed: self
                .0
                .limits
                .fuel
                .checked_sub(store.get_fuel().map_err(|_| BridgeError::InvalidLimits)?)
                .ok_or(BridgeError::InvalidLimits)?,
            memory_bytes: memory.data(&store).len(),
        })
    }
}

// Preserve a fixed actionable failure class, never VM/source strings or traps
// supplied by guest code. This includes Wasmi's memory/table fuel failures.
fn execution_error(error: wasmi::Error) -> BridgeError {
    if error.as_trap_code() == Some(wasmi::TrapCode::OutOfFuel) {
        BridgeError::FuelExhausted
    } else {
        BridgeError::ExecutionFailed
    }
}
