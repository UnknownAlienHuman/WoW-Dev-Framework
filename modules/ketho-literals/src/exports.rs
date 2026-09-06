//! Export names are the only unsafe attributes. There are no unsafe blocks or
//! native pointers crossing the bridge: offsets refer exclusively to Wasm memory.
//! The host writes only while the guest is stopped and drops the instance after
//! each request. Shared buffers are private to that isolated single-thread store.
use std::sync::Mutex;
use wow_render_contract::{ABI_VERSION, MAX_REQUEST_BYTES};
struct Buffers {
    input: Vec<u8>,
    output: Vec<u8>,
    ready: bool,
}
static BUFFERS: Mutex<Buffers> = Mutex::new(Buffers {
    input: Vec::new(),
    output: Vec::new(),
    ready: false,
});

#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn wow_abi_version() -> i32 {
    ABI_VERSION
}

#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn wow_request_buffer(length: i32) -> i32 {
    let Ok(mut state) = BUFFERS.lock() else {
        return -1;
    };
    state.ready = false;
    state.input.clear();
    state.output.clear();
    if length <= 0 || length as usize > MAX_REQUEST_BYTES {
        return -1;
    }
    if state.input.try_reserve_exact(length as usize).is_err() {
        return -1;
    }
    state.input.resize(length as usize, 0);
    state.ready = true;
    state.input.as_mut_ptr() as usize as i32
}

#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn wow_render() -> i32 {
    let Ok(mut state) = BUFFERS.lock() else {
        return 2;
    };
    state.output.clear();
    if !state.ready {
        return 2;
    }
    state.ready = false;
    match super::evaluate(&state.input) {
        Ok(output) => {
            state.output = output;
            0
        }
        Err(_) => 2,
    }
}
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn wow_response_ptr() -> i32 {
    BUFFERS
        .lock()
        .map(|state| state.output.as_ptr() as usize as i32)
        .unwrap_or(-1)
}
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn wow_response_len() -> i32 {
    BUFFERS
        .lock()
        .map(|state| state.output.len() as i32)
        .unwrap_or(-1)
}
