//! Preserve the serializer's object entries until duplicate admission succeeds.
//!
//! Converting the whole input with `serde_json::to_value` first would discard
//! repeated keys, including collisions produced by flattened structs. This
//! serializer builds the canonical value directly and never replays a payload.

use std::collections::BTreeMap;
use std::fmt;

use serde::Serialize;
use serde::ser::{SerializeMap, SerializeSeq, SerializeStruct};
use serde_json::{Map, Value};

#[derive(Debug)]
pub(super) enum AdmissionError {
    DuplicateField,
    Invalid(&'static str),
}

impl fmt::Display for AdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateField => formatter.write_str("duplicate canonical object key"),
            Self::Invalid(reason) => formatter.write_str(reason),
        }
    }
}

impl std::error::Error for AdmissionError {}

impl serde::ser::Error for AdmissionError {
    fn custom<T: fmt::Display>(_message: T) -> Self {
        // Custom serializer errors can contain source text or credentials.
        // Neither format nor retain caller-controlled diagnostic prose.
        Self::Invalid("serializer rejected the value")
    }
}

type Result<T> = std::result::Result<T, AdmissionError>;

// serde_json's arbitrary_precision feature is unified by other workspace
// owners. Its Number Serialize implementation uses this structured marker.
// Delegate that scalar protocol to serde_json itself; do not parse or round
// numbers through f64 and do not reinterpret a normal map with this key.
const JSON_NUMBER: &str = "$serde_json::private::Number";
const JSON_RAW_VALUE: &str = "$serde_json::private::RawValue";

pub(super) struct CanonicalSerializer;

macro_rules! unsigned_scalar {
    ($method:ident, $ty:ty) => {
        fn $method(self, value: $ty) -> Result<Value> {
            self.serialize_u64(u64::from(value))
        }
    };
}

macro_rules! signed_scalar {
    ($method:ident, $ty:ty) => {
        fn $method(self, value: $ty) -> Result<Value> {
            self.serialize_i64(i64::from(value))
        }
    };
}

impl serde::Serializer for CanonicalSerializer {
    type Ok = Value;
    type Error = AdmissionError;
    type SerializeSeq = Sequence;
    type SerializeTuple = Sequence;
    type SerializeTupleStruct = Sequence;
    type SerializeTupleVariant = TupleVariant;
    type SerializeMap = Object;
    type SerializeStruct = Object;
    type SerializeStructVariant = StructVariant;

    fn serialize_bool(self, value: bool) -> Result<Value> {
        Ok(Value::Bool(value))
    }

    signed_scalar!(serialize_i8, i8);
    signed_scalar!(serialize_i16, i16);
    signed_scalar!(serialize_i32, i32);
    unsigned_scalar!(serialize_u8, u8);
    unsigned_scalar!(serialize_u16, u16);
    unsigned_scalar!(serialize_u32, u32);

    fn serialize_i64(self, value: i64) -> Result<Value> {
        self.serialize_u64(u64::try_from(value).map_err(|_| invalid_number())?)
    }

    fn serialize_i128(self, value: i128) -> Result<Value> {
        self.serialize_u64(u64::try_from(value).map_err(|_| invalid_number())?)
    }

    fn serialize_u64(self, value: u64) -> Result<Value> {
        Ok(Value::Number(value.into()))
    }

    fn serialize_u128(self, value: u128) -> Result<Value> {
        self.serialize_u64(u64::try_from(value).map_err(|_| invalid_number())?)
    }

    fn serialize_f32(self, _value: f32) -> Result<Value> {
        Err(invalid_number())
    }

    fn serialize_f64(self, _value: f64) -> Result<Value> {
        Err(invalid_number())
    }

    fn serialize_char(self, value: char) -> Result<Value> {
        Ok(Value::String(value.to_string()))
    }

    fn serialize_str(self, value: &str) -> Result<Value> {
        Ok(Value::String(value.to_owned()))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Value> {
        Ok(Value::Array(
            value.iter().map(|byte| Value::from(*byte)).collect(),
        ))
    }

    fn serialize_none(self) -> Result<Value> {
        Err(AdmissionError::Invalid(
            "null is outside the E0 canonical subset",
        ))
    }

    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<Value> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Value> {
        self.serialize_none()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value> {
        self.serialize_none()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
    ) -> Result<Value> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Value> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Value> {
        validate_key(variant)?;
        tagged_value(variant.to_owned(), value.serialize(self)?)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Sequence> {
        // Hints are not trusted allocation sizes. Grow only for actual entries.
        Ok(Sequence(Vec::new()))
    }

    fn serialize_tuple(self, len: usize) -> Result<Sequence> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> Result<Sequence> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<TupleVariant> {
        validate_key(variant)?;
        Ok(TupleVariant {
            variant: variant.to_owned(),
            sequence: self.serialize_seq(Some(len))?,
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Object> {
        Ok(Object {
            entries: BTreeMap::new(),
            pending_key: None,
            json_number: false,
        })
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Object> {
        if name == JSON_RAW_VALUE {
            // Raw JSON would bypass key admission inside the embedded object.
            return Err(AdmissionError::Invalid(
                "raw JSON is not canonical material",
            ));
        }
        let mut object = self.serialize_map(Some(len))?;
        object.json_number = name == JSON_NUMBER;
        Ok(object)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<StructVariant> {
        validate_key(variant)?;
        Ok(StructVariant {
            variant: variant.to_owned(),
            object: self.serialize_map(Some(len))?,
        })
    }
}

fn invalid_number() -> AdmissionError {
    AdmissionError::Invalid("only unsigned 64-bit integers are canonical numbers")
}

pub(super) struct Sequence(Vec<Value>);

impl SerializeSeq for Sequence {
    type Ok = Value;
    type Error = AdmissionError;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<()> {
        self.0.push(value.serialize(CanonicalSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Array(self.0))
    }
}

impl serde::ser::SerializeTuple for Sequence {
    type Ok = Value;
    type Error = AdmissionError;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<()> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value> {
        SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleStruct for Sequence {
    type Ok = Value;
    type Error = AdmissionError;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<()> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value> {
        SerializeSeq::end(self)
    }
}

pub(super) struct Object {
    entries: BTreeMap<String, Value>,
    pending_key: Option<String>,
    json_number: bool,
}

impl Object {
    fn admit_key(&mut self, key: String) -> Result<()> {
        if self.pending_key.is_some() {
            return Err(AdmissionError::Invalid("canonical map key has no value"));
        }
        validate_key(&key)?;
        if self.entries.contains_key(&key) {
            return Err(AdmissionError::DuplicateField);
        }
        self.pending_key = Some(key);
        Ok(())
    }

    fn finish(self) -> Result<Value> {
        if self.pending_key.is_some() {
            return Err(AdmissionError::Invalid("canonical map key has no value"));
        }
        if self.json_number {
            return encode_json_number(self.entries);
        }
        // Sorted insertion stays bytewise even with serde_json/preserve_order.
        Ok(Value::Object(
            self.entries.into_iter().collect::<Map<_, _>>(),
        ))
    }
}

impl SerializeMap for Object {
    type Ok = Value;
    type Error = AdmissionError;

    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<()> {
        self.admit_key(json_key(key)?)
    }

    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<()> {
        let key = self
            .pending_key
            .take()
            .ok_or(AdmissionError::Invalid("canonical map value has no key"))?;
        self.entries
            .insert(key, value.serialize(CanonicalSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        self.finish()
    }
}

impl SerializeStruct for Object {
    type Ok = Value;
    type Error = AdmissionError;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<()> {
        self.admit_key(key.to_owned())?;
        SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> Result<Value> {
        self.finish()
    }
}

pub(super) struct TupleVariant {
    variant: String,
    sequence: Sequence,
}

impl serde::ser::SerializeTupleVariant for TupleVariant {
    type Ok = Value;
    type Error = AdmissionError;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<()> {
        SerializeSeq::serialize_element(&mut self.sequence, value)
    }

    fn end(self) -> Result<Value> {
        tagged_value(self.variant, SerializeSeq::end(self.sequence)?)
    }
}

pub(super) struct StructVariant {
    variant: String,
    object: Object,
}

impl serde::ser::SerializeStructVariant for StructVariant {
    type Ok = Value;
    type Error = AdmissionError;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<()> {
        SerializeStruct::serialize_field(&mut self.object, key, value)
    }

    fn end(self) -> Result<Value> {
        tagged_value(self.variant, self.object.finish()?)
    }
}

fn tagged_value(key: String, value: Value) -> Result<Value> {
    validate_key(&key)?;
    Ok(Value::Object([(key, value)].into_iter().collect()))
}

fn validate_key(key: &str) -> Result<()> {
    if key.is_empty() || key.bytes().any(|byte| byte.is_ascii_control()) {
        Err(AdmissionError::Invalid("invalid canonical object key"))
    } else {
        Ok(())
    }
}

fn json_key<T: Serialize + ?Sized>(key: &T) -> Result<String> {
    // Preserve serde_json's existing key spelling (e.g. 1 and "1" collide).
    // This one-entry scalar-key adapter contains no application value or map
    // inventory. The payload never goes through a lossy to_value projection.
    let invalid = || AdmissionError::Invalid("invalid canonical object key");
    let mut map = serde::Serializer::serialize_map(serde_json::value::Serializer, Some(1))
        .map_err(|_| invalid())?;
    map.serialize_entry(key, &true).map_err(|_| invalid())?;
    let Value::Object(map) = SerializeMap::end(map).map_err(|_| invalid())? else {
        return Err(invalid());
    };
    map.into_iter()
        .next()
        .map(|(key, _)| key)
        .ok_or_else(invalid)
}

fn encode_json_number(entries: BTreeMap<String, Value>) -> Result<Value> {
    if entries.len() != 1 {
        return Err(invalid_number());
    }
    let value = entries.get(JSON_NUMBER).ok_or_else(invalid_number)?;
    let Value::String(text) = value else {
        return Err(invalid_number());
    };
    // Admit the retained token before serde_json can normalize its spelling.
    // In particular, -0, +1 and 01 must not become canonical integers here.
    let integer = text.parse::<u64>().map_err(|_| invalid_number())?;
    if integer.to_string() != *text {
        return Err(invalid_number());
    }
    let mut number =
        serde::Serializer::serialize_struct(serde_json::value::Serializer, JSON_NUMBER, 1)
            .map_err(|_| invalid_number())?;
    SerializeStruct::serialize_field(&mut number, JSON_NUMBER, value)
        .map_err(|_| invalid_number())?;
    let value = SerializeStruct::end(number).map_err(|_| invalid_number())?;
    match value {
        Value::Number(ref number) if number.as_u64().is_none() => Err(invalid_number()),
        // Without arbitrary_precision this is an ordinary one-field struct,
        // matching serde_json's existing behavior. Its field is already checked.
        value => Ok(value),
    }
}
