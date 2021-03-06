//! Definition of a TOML value

use std::collections::BTreeMap;
use std::fmt;
use std::ops;
use std::str::FromStr;
use std::vec;

use serde::ser;
use serde::de;

pub use datetime::{Datetime, DatetimeParseError};
use datetime::{DatetimeFromString, SERDE_STRUCT_FIELD_NAME};

/// Representation of a TOML value.
#[derive(PartialEq, Clone, Debug)]
pub enum Value {
    /// Represents a TOML string
    String(String),
    /// Represents a TOML integer
    Integer(i64),
    /// Represents a TOML float
    Float(f64),
    /// Represents a TOML boolean
    Boolean(bool),
    /// Represents a TOML datetime
    Datetime(Datetime),
    /// Represents a TOML array
    Array(Array),
    /// Represents a TOML table
    Table(Table),
}

/// Type representing a TOML array, payload of the `Value::Array` variant
pub type Array = Vec<Value>;

/// Type representing a TOML table, payload of the `Value::Table` variant
pub type Table = BTreeMap<String, Value>;

impl Value {
    /// Convert a `T` into `toml::Value` which is an enum that can represent
    /// any valid TOML data.
    ///
    /// This conversion can fail if `T`'s implementation of `Serialize` decides to
    /// fail, or if `T` contains a map with non-string keys.
    pub fn try_from<T>(value: T) -> Result<Value, ::ser::Error>
        where T: ser::Serialize,
    {
        value.serialize(Serializer)
    }

    /// Interpret a `toml::Value` as an instance of type `T`.
    ///
    /// This conversion can fail if the structure of the `Value` does not match the
    /// structure expected by `T`, for example if `T` is a struct type but the
    /// `Value` contains something other than a TOML table. It can also fail if the
    /// structure is correct but `T`'s implementation of `Deserialize` decides that
    /// something is wrong with the data, for example required struct fields are
    /// missing from the TOML map or some number is too big to fit in the expected
    /// primitive type.
    pub fn try_into<T>(self) -> Result<T, ::de::Error>
        where T: de::Deserialize,
    {
        de::Deserialize::deserialize(self)
    }

    /// Index into a TOML array or map. A string index can be used to access a
    /// value in a map, and a usize index can be used to access an element of an
    /// array.
    ///
    /// Returns `None` if the type of `self` does not match the type of the
    /// index, for example if the index is a string and `self` is an array or a
    /// number. Also returns `None` if the given key does not exist in the map
    /// or the given index is not within the bounds of the array.
    pub fn get<I: Index>(&self, index: I) -> Option<&Value> {
        index.index(self)
    }

    /// Mutably index into a TOML array or map. A string index can be used to
    /// access a value in a map, and a usize index can be used to access an
    /// element of an array.
    ///
    /// Returns `None` if the type of `self` does not match the type of the
    /// index, for example if the index is a string and `self` is an array or a
    /// number. Also returns `None` if the given key does not exist in the map
    /// or the given index is not within the bounds of the array.
    pub fn get_mut<I: Index>(&mut self, index: I) -> Option<&mut Value> {
        index.index_mut(self)
    }

    /// Extracts the integer value if it is an integer.
    pub fn as_integer(&self) -> Option<i64> {
        match *self { Value::Integer(i) => Some(i), _ => None }
    }

    /// Tests whether this value is an integer
    pub fn is_integer(&self) -> bool {
        self.as_integer().is_some()
    }

    /// Extracts the float value if it is a float.
    pub fn as_float(&self) -> Option<f64> {
        match *self { Value::Float(f) => Some(f), _ => None }
    }

    /// Tests whether this value is a float
    pub fn is_float(&self) -> bool {
        self.as_float().is_some()
    }

    /// Extracts the boolean value if it is a boolean.
    pub fn as_bool(&self) -> Option<bool> {
        match *self { Value::Boolean(b) => Some(b), _ => None }
    }

    /// Tests whether this value is a boolean
    pub fn is_bool(&self) -> bool {
        self.as_bool().is_some()
    }

    /// Extracts the string of this value if it is a string.
    pub fn as_str(&self) -> Option<&str> {
        match *self { Value::String(ref s) => Some(&**s), _ => None }
    }

    /// Tests if this value is a string
    pub fn is_str(&self) -> bool {
        self.as_str().is_some()
    }

    /// Extracts the datetime value if it is a datetime.
    ///
    /// Note that a parsed TOML value will only contain ISO 8601 dates. An
    /// example date is:
    ///
    /// ```notrust
    /// 1979-05-27T07:32:00Z
    /// ```
    pub fn as_datetime(&self) -> Option<&Datetime> {
        match *self { Value::Datetime(ref s) => Some(s), _ => None }
    }

    /// Tests whether this value is a datetime
    pub fn is_datetime(&self) -> bool {
        self.as_datetime().is_some()
    }

    /// Extracts the array value if it is an array.
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match *self { Value::Array(ref s) => Some(s), _ => None }
    }

    /// Extracts the array value if it is an array.
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> {
        match *self { Value::Array(ref mut s) => Some(s), _ => None }
    }

    /// Tests whether this value is an array
    pub fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    /// Extracts the table value if it is a table.
    pub fn as_table(&self) -> Option<&Table> {
        match *self { Value::Table(ref s) => Some(s), _ => None }
    }

    /// Extracts the table value if it is a table.
    pub fn as_table_mut(&mut self) -> Option<&mut Table> {
        match *self { Value::Table(ref mut s) => Some(s), _ => None }
    }

    /// Extracts the table value if it is a table.
    pub fn is_table(&self) -> bool {
        self.as_table().is_some()
    }

    /// Tests whether this and another value have the same type.
    pub fn same_type(&self, other: &Value) -> bool {
        match (self, other) {
            (&Value::String(..), &Value::String(..)) |
            (&Value::Integer(..), &Value::Integer(..)) |
            (&Value::Float(..), &Value::Float(..)) |
            (&Value::Boolean(..), &Value::Boolean(..)) |
            (&Value::Datetime(..), &Value::Datetime(..)) |
            (&Value::Array(..), &Value::Array(..)) |
            (&Value::Table(..), &Value::Table(..)) => true,

            _ => false,
        }
    }

    /// Returns a human-readable representation of the type of this value.
    pub fn type_str(&self) -> &'static str {
        match *self {
            Value::String(..) => "string",
            Value::Integer(..) => "integer",
            Value::Float(..) => "float",
            Value::Boolean(..) => "boolean",
            Value::Datetime(..) => "datetime",
            Value::Array(..) => "array",
            Value::Table(..) => "table",
        }
    }
}

impl<I> ops::Index<I> for Value where I: Index {
    type Output = Value;

    fn index(&self, index: I) -> &Value {
        self.get(index).expect("index not found")
    }
}

impl<I> ops::IndexMut<I> for Value where I: Index {
    fn index_mut(&mut self, index: I) -> &mut Value {
        self.get_mut(index).expect("index not found")
    }
}

impl From<String> for Value {
    fn from(val: String) -> Value {
        Value::String(val)
    }
}

impl From<i64> for Value {
    fn from(val: i64) -> Value {
        Value::Integer(val)
    }
}

impl From<f64> for Value {
    fn from(val: f64) -> Value {
        Value::Float(val)
    }
}

impl From<bool> for Value {
    fn from(val: bool) -> Value {
        Value::Boolean(val)
    }
}

impl From<Array> for Value {
    fn from(val: Array) -> Value {
        Value::Array(val)
    }
}

impl From<Table> for Value {
    fn from(val: Table) -> Value {
        Value::Table(val)
    }
}

impl From<Datetime> for Value {
    fn from(val: Datetime) -> Value {
        Value::Datetime(val)
    }
}

/// Types that can be used to index a `toml::Value`
///
/// Currently this is implemented for `usize` to index arrays and `str` to index
/// tables.
///
/// This trait is sealed and not intended for implementation outside of the
/// `toml` crate.
pub trait Index: Sealed {
    #[doc(hidden)]
    fn index<'a>(&self, val: &'a Value) -> Option<&'a Value>;
    #[doc(hidden)]
    fn index_mut<'a>(&self, val: &'a mut Value) -> Option<&'a mut Value>;
}

/// An implementation detail that should not be implemented, this will change in
/// the future and break code otherwise.
#[doc(hidden)]
pub trait Sealed {}
impl Sealed for usize {}
impl Sealed for str {}
impl Sealed for String {}
impl<'a, T: Sealed + ?Sized> Sealed for &'a T {}

impl Index for usize {
    fn index<'a>(&self, val: &'a Value) -> Option<&'a Value> {
        match *val {
            Value::Array(ref a) => a.get(*self),
            _ => None,
        }
    }

    fn index_mut<'a>(&self, val: &'a mut Value) -> Option<&'a mut Value> {
        match *val {
            Value::Array(ref mut a) => a.get_mut(*self),
            _ => None,
        }
    }
}

impl Index for str {
    fn index<'a>(&self, val: &'a Value) -> Option<&'a Value> {
        match *val {
            Value::Table(ref a) => a.get(self),
            _ => None,
        }
    }

    fn index_mut<'a>(&self, val: &'a mut Value) -> Option<&'a mut Value> {
        match *val {
            Value::Table(ref mut a) => a.get_mut(self),
            _ => None,
        }
    }
}

impl Index for String {
    fn index<'a>(&self, val: &'a Value) -> Option<&'a Value> {
        self[..].index(val)
    }

    fn index_mut<'a>(&self, val: &'a mut Value) -> Option<&'a mut Value> {
        self[..].index_mut(val)
    }
}

impl<'s, T: ?Sized> Index for &'s T where T: Index {
    fn index<'a>(&self, val: &'a Value) -> Option<&'a Value> {
        (**self).index(val)
    }

    fn index_mut<'a>(&self, val: &'a mut Value) -> Option<&'a mut Value> {
        (**self).index_mut(val)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        ::ser::to_string(self).unwrap().fmt(f)
    }
}

impl FromStr for Value {
    type Err = ::de::Error;
    fn from_str(s: &str) -> Result<Value, Self::Err> {
        ::from_str(s)
    }
}

impl ser::Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: ser::Serializer
    {
        use serde::ser::SerializeMap;

        match *self {
            Value::String(ref s) => serializer.serialize_str(s),
            Value::Integer(i) => serializer.serialize_i64(i),
            Value::Float(f) => serializer.serialize_f64(f),
            Value::Boolean(b) => serializer.serialize_bool(b),
            Value::Datetime(ref s) => s.serialize(serializer),
            Value::Array(ref a) => a.serialize(serializer),
            Value::Table(ref t) => {
                let mut map = serializer.serialize_map(Some(t.len()))?;
                // Be sure to visit non-tables first (and also non
                // array-of-tables) as all keys must be emitted first.
                for (k, v) in t {
                    if !v.is_array() && !v.is_table() {
                        map.serialize_key(k)?;
                        map.serialize_value(v)?;
                    }
                }
                for (k, v) in t {
                    if v.is_array() {
                        map.serialize_key(k)?;
                        map.serialize_value(v)?;
                    }
                }
                for (k, v) in t {
                    if v.is_table() {
                        map.serialize_key(k)?;
                        map.serialize_value(v)?;
                    }
                }
                map.end()
            }
        }
    }
}

impl de::Deserialize for Value {
    fn deserialize<D>(deserializer: D) -> Result<Value, D::Error>
        where D: de::Deserializer
    {
        struct ValueVisitor;

        impl de::Visitor for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("any valid TOML value")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
                Ok(Value::Boolean(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
                Ok(Value::Integer(value))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Value, E> {
                Ok(Value::Float(value))
            }

            fn visit_str<E>(self, value: &str) -> Result<Value, E> {
                Ok(Value::String(value.into()))
            }

            fn visit_string<E>(self, value: String) -> Result<Value, E> {
                Ok(Value::String(value))
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Value, D::Error>
                where D: de::Deserializer,
            {
                de::Deserialize::deserialize(deserializer)
            }

            fn visit_seq<V>(self, visitor: V) -> Result<Value, V::Error>
                where V: de::SeqVisitor
            {
                let values = de::impls::VecVisitor::new().visit_seq(visitor)?;
                Ok(Value::Array(values))
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Value, V::Error>
                where V: de::MapVisitor
            {
                let mut key = String::new();
                let datetime = visitor.visit_key_seed(DatetimeOrTable {
                    key: &mut key,
                })?;
                match datetime {
                    Some(true) => {
                        let date: DatetimeFromString = visitor.visit_value()?;
                        return Ok(Value::Datetime(date.value))
                    }
                    None => return Ok(Value::Table(BTreeMap::new())),
                    Some(false) => {}
                }
                let mut map = BTreeMap::new();
                map.insert(key, visitor.visit_value()?);
                while let Some(key) = visitor.visit_key()? {
                    if map.contains_key(&key) {
                        let msg = format!("duplicate key: `{}`", key);
                        return Err(de::Error::custom(msg))
                    }
                    map.insert(key, visitor.visit_value()?);
                }
                Ok(Value::Table(map))
            }
        }

        deserializer.deserialize(ValueVisitor)
    }
}

impl de::Deserializer for Value {
    type Error = ::de::Error;

    fn deserialize<V>(self, visitor: V) -> Result<V::Value, ::de::Error>
        where V: de::Visitor,
    {
        match self {
            Value::Boolean(v) => visitor.visit_bool(v),
            Value::Integer(n) => visitor.visit_i64(n),
            Value::Float(n) => visitor.visit_f64(n),
            Value::String(v) => visitor.visit_string(v),
            Value::Datetime(v) => visitor.visit_string(v.to_string()),
            Value::Array(v) => {
                let len = v.len();
                let mut deserializer = SeqDeserializer::new(v);
                let seq = visitor.visit_seq(&mut deserializer)?;
                let remaining = deserializer.iter.len();
                if remaining == 0 {
                    Ok(seq)
                } else {
                    Err(de::Error::invalid_length(len, &"fewer elements in array"))
                }
            }
            Value::Table(v) => {
                let len = v.len();
                let mut deserializer = MapDeserializer::new(v);
                let map = visitor.visit_map(&mut deserializer)?;
                let remaining = deserializer.iter.len();
                if remaining == 0 {
                    Ok(map)
                } else {
                    Err(de::Error::invalid_length(len, &"fewer elements in map"))
                }
            }
        }
    }

    // `None` is interpreted as a missing field so be sure to implement `Some`
    // as a present field.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, ::de::Error>
        where V: de::Visitor
    {
        visitor.visit_some(self)
    }

    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit seq
        seq_fixed_size bytes byte_buf map unit_struct tuple_struct struct
        struct_field tuple ignored_any enum newtype_struct
    }
}

struct SeqDeserializer {
    iter: vec::IntoIter<Value>,
}

impl SeqDeserializer {
    fn new(vec: Vec<Value>) -> Self {
        SeqDeserializer {
            iter: vec.into_iter(),
        }
    }
}

impl de::SeqVisitor for SeqDeserializer {
    type Error = ::de::Error;

    fn visit_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, ::de::Error>
        where T: de::DeserializeSeed,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

struct MapDeserializer {
    iter: <BTreeMap<String, Value> as IntoIterator>::IntoIter,
    value: Option<(String, Value)>,
}

impl MapDeserializer {
    fn new(map: BTreeMap<String, Value>) -> Self {
        MapDeserializer {
            iter: map.into_iter(),
            value: None,
        }
    }
}

impl de::MapVisitor for MapDeserializer {
    type Error = ::de::Error;

    fn visit_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, ::de::Error>
        where T: de::DeserializeSeed,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some((key.clone(), value));
                seed.deserialize(Value::String(key)).map(Some)
            }
            None => Ok(None),
        }
    }

    fn visit_value_seed<T>(&mut self, seed: T) -> Result<T::Value, ::de::Error>
        where T: de::DeserializeSeed,
    {
        let (key, res) = match self.value.take() {
            Some((key, value)) => (key, seed.deserialize(value)),
            None => return Err(de::Error::custom("value is missing")),
        };
        res.map_err(|mut error| {
            error.add_key_context(&key);
            error
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

struct Serializer;

impl ser::Serializer for Serializer {
    type Ok = Value;
    type Error = ::ser::Error;

    type SerializeSeq = SerializeVec;
    type SerializeTuple = ser::Impossible<Value, ::ser::Error>;
    type SerializeTupleStruct = ser::Impossible<Value, ::ser::Error>;
    type SerializeTupleVariant = ser::Impossible<Value, ::ser::Error>;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = ser::Impossible<Value, ::ser::Error>;

    fn serialize_bool(self, value: bool) -> Result<Value, ::ser::Error> {
        Ok(Value::Boolean(value))
    }

    fn serialize_i8(self, value: i8) -> Result<Value, ::ser::Error> {
        self.serialize_i64(value.into())
    }

    fn serialize_i16(self, value: i16) -> Result<Value, ::ser::Error> {
        self.serialize_i64(value.into())
    }

    fn serialize_i32(self, value: i32) -> Result<Value, ::ser::Error> {
        self.serialize_i64(value.into())
    }

    fn serialize_i64(self, value: i64) -> Result<Value, ::ser::Error> {
        Ok(Value::Integer(value.into()))
    }

    fn serialize_u8(self, value: u8) -> Result<Value, ::ser::Error> {
        self.serialize_i64(value.into())
    }

    fn serialize_u16(self, value: u16) -> Result<Value, ::ser::Error> {
        self.serialize_i64(value.into())
    }

    fn serialize_u32(self, value: u32) -> Result<Value, ::ser::Error> {
        self.serialize_i64(value.into())
    }

    fn serialize_u64(self, value: u64) -> Result<Value, ::ser::Error> {
        if value <= i64::max_value() as u64 {
            self.serialize_i64(value as i64)
        } else {
            Err(ser::Error::custom("u64 value was too large"))
        }
    }

    fn serialize_f32(self, value: f32) -> Result<Value, ::ser::Error> {
        self.serialize_f64(value.into())
    }

    fn serialize_f64(self, value: f64) -> Result<Value, ::ser::Error> {
        Ok(Value::Float(value))
    }

    fn serialize_char(self, value: char) -> Result<Value, ::ser::Error> {
        let mut s = String::new();
        s.push(value);
        self.serialize_str(&s)
    }

    fn serialize_str(self, value: &str) -> Result<Value, ::ser::Error> {
        Ok(Value::String(value.to_owned()))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Value, ::ser::Error> {
        let vec = value.iter().map(|&b| Value::Integer(b.into())).collect();
        Ok(Value::Array(vec))
    }

    fn serialize_unit(self) -> Result<Value, ::ser::Error> {
        Err(::ser::Error::UnsupportedType)
    }

    fn serialize_unit_struct(self, _name: &'static str)
                             -> Result<Value, ::ser::Error> {
        Err(::ser::Error::UnsupportedType)
    }

    fn serialize_unit_variant(self,
                              _name: &'static str,
                              _variant_index: usize,
                              _variant: &'static str)
                              -> Result<Value, ::ser::Error> {
        Err(::ser::Error::UnsupportedType)
    }

    fn serialize_newtype_struct<T: ?Sized>(self,
                                           _name: &'static str,
                                           value: &T)
                                           -> Result<Value, ::ser::Error>
        where T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(self,
                                            _name: &'static str,
                                            _variant_index: usize,
                                            _variant: &'static str,
                                            _value: &T)
                                            -> Result<Value, ::ser::Error>
        where T: ser::Serialize,
    {
        Err(::ser::Error::UnsupportedType)
    }

    fn serialize_none(self) -> Result<Value, ::ser::Error> {
        Err(::ser::Error::UnsupportedNone)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Value, ::ser::Error>
        where T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>)
                     -> Result<Self::SerializeSeq, ::ser::Error>
    {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len.unwrap_or(0))
        })
    }

    fn serialize_seq_fixed_size(self, size: usize)
                                -> Result<Self::SerializeSeq, ::ser::Error> {
        self.serialize_seq(Some(size))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, ::ser::Error> {
        Err(::ser::Error::UnsupportedType)
    }

    fn serialize_tuple_struct(self, _name: &'static str, _len: usize)
                              -> Result<Self::SerializeTupleStruct, ::ser::Error> {
        Err(::ser::Error::UnsupportedType)
    }

    fn serialize_tuple_variant(self,
                               _name: &'static str,
                               _variant_index: usize,
                               _variant: &'static str,
                               _len: usize)
                               -> Result<Self::SerializeTupleVariant, ::ser::Error>
    {
        Err(::ser::Error::UnsupportedType)
    }

    fn serialize_map(self, _len: Option<usize>)
                     -> Result<Self::SerializeMap, ::ser::Error>
    {
        Ok(SerializeMap {
            map: BTreeMap::new(),
            next_key: None,
        })
    }

    fn serialize_struct(self, _name: &'static str, len: usize)
                        -> Result<Self::SerializeStruct, ::ser::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(self,
                                _name: &'static str,
                                _variant_index: usize,
                                _variant: &'static str,
                                _len: usize)
                                -> Result<Self::SerializeStructVariant, ::ser::Error>
    {
        Err(::ser::Error::UnsupportedType)
    }
}

struct SerializeVec {
    vec: Vec<Value>,
}

struct SerializeMap {
    map: BTreeMap<String, Value>,
    next_key: Option<String>,
}

impl ser::SerializeSeq for SerializeVec {
    type Ok = Value;
    type Error = ::ser::Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), ::ser::Error>
        where T: ser::Serialize
    {
        self.vec.push(Value::try_from(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value, ::ser::Error> {
        Ok(Value::Array(self.vec))
    }
}

impl ser::SerializeMap for SerializeMap {
    type Ok = Value;
    type Error = ::ser::Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), ::ser::Error>
        where T: ser::Serialize
    {
        match Value::try_from(key)? {
            Value::String(s) => self.next_key = Some(s),
            _ => return Err(::ser::Error::KeyNotString),
        };
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), ::ser::Error>
        where T: ser::Serialize
    {
        let key = self.next_key.take();
        let key = key.expect("serialize_value called before serialize_key");
        match Value::try_from(value) {
            Ok(value) => { self.map.insert(key, value); }
            Err(::ser::Error::UnsupportedNone) => {}
            Err(e) => return Err(e),
        }
        Ok(())
    }

    fn end(self) -> Result<Value, ::ser::Error> {
        Ok(Value::Table(self.map))
    }
}

impl ser::SerializeStruct for SerializeMap {
    type Ok = Value;
    type Error = ::ser::Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), ::ser::Error>
        where T: ser::Serialize
    {
        try!(ser::SerializeMap::serialize_key(self, key));
        ser::SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> Result<Value, ::ser::Error> {
        ser::SerializeMap::end(self)
    }
}

struct DatetimeOrTable<'a> {
    key: &'a mut String,
}

impl<'a> de::DeserializeSeed for DatetimeOrTable<'a> {
    type Value = bool;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: de::Deserializer
    {
        deserializer.deserialize(self)
    }
}

impl<'a> de::Visitor for DatetimeOrTable<'a> {
    type Value = bool;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string key")
    }

    fn visit_str<E>(self, s: &str) -> Result<bool, E>
        where E: de::Error,
    {
        if s == SERDE_STRUCT_FIELD_NAME {
            Ok(true)
        } else {
            self.key.push_str(s);
            Ok(false)
        }
    }

    fn visit_string<E>(self, s: String) -> Result<bool, E>
        where E: de::Error,
    {
        if s == SERDE_STRUCT_FIELD_NAME {
            Ok(true)
        } else {
            *self.key = s;
            Ok(false)
        }
    }
}
