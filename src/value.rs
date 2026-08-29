//! An owned Rust representation of a TJS variant.
//!
//! [`TJSVariant`] is useful when a value needs to be inspected or changed in
//! Rust without keeping a reference to a TJS object.  It intentionally has
//! the same six value kinds as `tTJSVariant`: void, integer, real, octet,
//! string, dictionary, and array.

use crate::tp_stub::tTJSVariant;
use serde::de::{Error as DeError, MapAccess, SeqAccess, Visitor};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;
use std::ops::{Index, IndexMut};

/// An owned Rust representation of a `tTJSVariant`.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum TJSVariant {
    /// The TJS void value.  This is serialized as a unit/null value.
    Void,
    Integer(i64),
    Real(f64),
    Octet(Vec<u8>),
    String(String),
    Dict(HashMap<String, TJSVariant>),
    Array(Vec<TJSVariant>),
}

enum MaybeIter<I> {
    Some(I),
    Empty,
}

impl<I> Iterator for MaybeIter<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Some(iter) => iter.next(),
            Self::Empty => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Some(iter) => iter.size_hint(),
            Self::Empty => (0, Some(0)),
        }
    }
}

impl<I> ExactSizeIterator for MaybeIter<I>
where
    I: ExactSizeIterator,
{
    fn len(&self) -> usize {
        match self {
            Self::Some(iter) => iter.len(),
            Self::Empty => 0,
        }
    }
}

impl Default for TJSVariant {
    fn default() -> Self {
        Self::Void
    }
}

impl TJSVariant {
    /// Creates an empty dictionary.
    pub fn object() -> Self {
        Self::Dict(HashMap::new())
    }

    /// Wraps a dictionary as a TJS value.
    pub fn from_object(object: HashMap<String, Self>) -> Self {
        Self::Dict(object)
    }

    /// Creates an empty array.
    pub fn array() -> Self {
        Self::Array(Vec::new())
    }

    /// Wraps a vector as a TJS value.
    pub fn from_array(array: Vec<Self>) -> Self {
        Self::Array(array)
    }

    /// Converts a TJS value into an owned Rust value.
    pub fn from_tjs(value: &mut tTJSVariant) -> Result<Self, crate::de::DeserError> {
        crate::de::from(value)
    }

    /// Converts this value into a TJS value.
    pub fn to_tjs(&self) -> Result<tTJSVariant, crate::ser::SerError> {
        crate::ser::to(self)
    }

    pub fn is_void(&self) -> bool {
        matches!(self, Self::Void)
    }

    /// Alias for [`is_void`](Self::is_void), matching JSON value APIs.
    pub fn is_null(&self) -> bool {
        self.is_void()
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, Self::Integer(_))
    }

    pub fn is_real(&self) -> bool {
        matches!(self, Self::Real(_))
    }

    pub fn is_number(&self) -> bool {
        self.is_integer() || self.is_real()
    }

    pub fn is_octet(&self) -> bool {
        matches!(self, Self::Octet(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }

    pub fn is_dict(&self) -> bool {
        matches!(self, Self::Dict(_))
    }

    /// Alias for [`is_dict`](Self::is_dict).
    pub fn is_object(&self) -> bool {
        self.is_dict()
    }

    /// Returns the integer, if this is an integer value.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Integer(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns the number as an `f64`.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Integer(value) => Some(*value as f64),
            Self::Real(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns a TJS boolean when this integer is either zero or one.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Integer(0) => Some(false),
            Self::Integer(1) => Some(true),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Octet(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Self>> {
        match self {
            Self::Array(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Self>> {
        match self {
            Self::Array(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, Self>> {
        match self {
            Self::Dict(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_object_mut(&mut self) -> Option<&mut HashMap<String, Self>> {
        match self {
            Self::Dict(value) => Some(value),
            _ => None,
        }
    }

    /// Iterates over array members.  Non-array values produce an empty
    /// iterator.
    pub fn members(&self) -> impl Iterator<Item = &Self> + '_ {
        match self {
            Self::Array(array) => MaybeIter::Some(array.iter()),
            _ => MaybeIter::Empty,
        }
    }

    /// Iterates mutably over array members.  Non-array values produce an
    /// empty iterator.
    pub fn members_mut(&mut self) -> impl Iterator<Item = &mut Self> + '_ {
        match self {
            Self::Array(array) => MaybeIter::Some(array.iter_mut()),
            _ => MaybeIter::Empty,
        }
    }

    /// Iterates over dictionary entries.  Non-dictionary values produce an
    /// empty iterator.
    pub fn entries(&self) -> impl Iterator<Item = (&String, &Self)> + '_ {
        match self {
            Self::Dict(object) => MaybeIter::Some(object.iter()),
            _ => MaybeIter::Empty,
        }
    }

    /// Iterates mutably over dictionary entries.  Non-dictionary values
    /// produce an empty iterator.
    pub fn entries_mut(&mut self) -> impl Iterator<Item = (&String, &mut Self)> + '_ {
        match self {
            Self::Dict(object) => MaybeIter::Some(object.iter_mut()),
            _ => MaybeIter::Empty,
        }
    }

    /// Looks up an array element or dictionary member.
    pub fn get<I>(&self, index: I) -> Option<&Self>
    where
        I: TJSVariantIndex,
    {
        index.index_into(self)
    }

    /// Looks up an array element or dictionary member mutably.
    pub fn get_mut<I>(&mut self, index: I) -> Option<&mut Self>
    where
        I: TJSVariantIndex,
    {
        index.index_into_mut(self)
    }

    /// Inserts a dictionary member, returning its previous value.
    pub fn insert<K>(&mut self, key: K, value: Self) -> Option<Self>
    where
        K: Into<String>,
    {
        match self {
            Self::Dict(object) => object.insert(key.into(), value),
            _ => None,
        }
    }

    /// Removes a dictionary member, returning it when it existed.
    pub fn remove(&mut self, key: &str) -> Option<Self> {
        match self {
            Self::Dict(object) => object.remove(key),
            _ => None,
        }
    }

    /// Appends an element to an array.
    pub fn push(&mut self, value: Self) -> Result<(), &'static str> {
        match self {
            Self::Array(array) => {
                array.push(value);
                Ok(())
            }
            _ => Err("cannot push to a non-array TJSVariant"),
        }
    }
}

/// Index types accepted by [`TJSVariant::get`] and `value[index]`.
pub trait TJSVariantIndex {
    fn index_into<'a>(&self, value: &'a TJSVariant) -> Option<&'a TJSVariant>;
    fn index_into_mut<'a>(&self, value: &'a mut TJSVariant) -> Option<&'a mut TJSVariant>;
    fn index_or_insert<'a>(&self, value: &'a mut TJSVariant) -> &'a mut TJSVariant;

    fn dictionary_key(&self) -> Option<String> {
        None
    }
}

impl TJSVariantIndex for str {
    fn index_into<'a>(&self, value: &'a TJSVariant) -> Option<&'a TJSVariant> {
        value.as_object()?.get(self)
    }

    fn index_into_mut<'a>(&self, value: &'a mut TJSVariant) -> Option<&'a mut TJSVariant> {
        value.as_object_mut()?.get_mut(self)
    }

    fn index_or_insert<'a>(&self, value: &'a mut TJSVariant) -> &'a mut TJSVariant {
        if matches!(value, TJSVariant::Void) {
            *value = TJSVariant::object();
        }
        if let TJSVariant::Dict(object) = value {
            object.entry(self.to_owned()).or_insert(TJSVariant::Void)
        } else {
            value
        }
    }

    fn dictionary_key(&self) -> Option<String> {
        Some(self.to_owned())
    }
}

impl TJSVariantIndex for String {
    fn index_into<'a>(&self, value: &'a TJSVariant) -> Option<&'a TJSVariant> {
        self.as_str().index_into(value)
    }

    fn index_into_mut<'a>(&self, value: &'a mut TJSVariant) -> Option<&'a mut TJSVariant> {
        self.as_str().index_into_mut(value)
    }

    fn index_or_insert<'a>(&self, value: &'a mut TJSVariant) -> &'a mut TJSVariant {
        self.as_str().index_or_insert(value)
    }

    fn dictionary_key(&self) -> Option<String> {
        Some(self.clone())
    }
}

impl TJSVariantIndex for usize {
    fn index_into<'a>(&self, value: &'a TJSVariant) -> Option<&'a TJSVariant> {
        value.as_array()?.get(*self)
    }

    fn index_into_mut<'a>(&self, value: &'a mut TJSVariant) -> Option<&'a mut TJSVariant> {
        value.as_array_mut()?.get_mut(*self)
    }

    fn index_or_insert<'a>(&self, value: &'a mut TJSVariant) -> &'a mut TJSVariant {
        if matches!(value, TJSVariant::Void) {
            *value = TJSVariant::array();
        }
        if let TJSVariant::Array(array) = value {
            if *self >= array.len() {
                array.resize(*self + 1, TJSVariant::Void);
            }
            &mut array[*self]
        } else {
            value
        }
    }
}

impl TJSVariantIndex for i32 {
    fn index_into<'a>(&self, value: &'a TJSVariant) -> Option<&'a TJSVariant> {
        (*self)
            .try_into()
            .ok()
            .and_then(|index: usize| index.index_into(value))
    }

    fn index_into_mut<'a>(&self, value: &'a mut TJSVariant) -> Option<&'a mut TJSVariant> {
        (*self)
            .try_into()
            .ok()
            .and_then(|index: usize| index.index_into_mut(value))
    }

    fn index_or_insert<'a>(&self, value: &'a mut TJSVariant) -> &'a mut TJSVariant {
        let index: Result<usize, _> = (*self).try_into();
        match index {
            Ok(index) => index.index_or_insert(value),
            Err(_) => value,
        }
    }
}

impl TJSVariantIndex for &str {
    fn index_into<'a>(&self, value: &'a TJSVariant) -> Option<&'a TJSVariant> {
        (*self).index_into(value)
    }

    fn index_into_mut<'a>(&self, value: &'a mut TJSVariant) -> Option<&'a mut TJSVariant> {
        (*self).index_into_mut(value)
    }

    fn index_or_insert<'a>(&self, value: &'a mut TJSVariant) -> &'a mut TJSVariant {
        (*self).index_or_insert(value)
    }

    fn dictionary_key(&self) -> Option<String> {
        Some((*self).to_owned())
    }
}

impl TJSVariantIndex for &String {
    fn index_into<'a>(&self, value: &'a TJSVariant) -> Option<&'a TJSVariant> {
        self.as_str().index_into(value)
    }

    fn index_into_mut<'a>(&self, value: &'a mut TJSVariant) -> Option<&'a mut TJSVariant> {
        self.as_str().index_into_mut(value)
    }

    fn index_or_insert<'a>(&self, value: &'a mut TJSVariant) -> &'a mut TJSVariant {
        self.as_str().index_or_insert(value)
    }

    fn dictionary_key(&self) -> Option<String> {
        Some((*self).clone())
    }
}

static VOID: TJSVariant = TJSVariant::Void;

impl<I> Index<I> for TJSVariant
where
    I: TJSVariantIndex,
{
    type Output = TJSVariant;

    fn index(&self, index: I) -> &Self::Output {
        index.index_into(self).unwrap_or(&VOID)
    }
}

impl<I> IndexMut<I> for TJSVariant
where
    I: TJSVariantIndex,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        index.index_or_insert(self)
    }
}

impl Serialize for TJSVariant {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Void => serializer.serialize_unit(),
            Self::Integer(value) => serializer.serialize_i64(*value),
            Self::Real(value) => serializer.serialize_f64(*value),
            Self::Octet(value) => serializer.serialize_bytes(value),
            Self::String(value) => serializer.serialize_str(value),
            Self::Dict(object) => {
                let mut map = serializer.serialize_map(Some(object.len()))?;
                for (key, value) in object {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
            Self::Array(array) => {
                let mut seq = serializer.serialize_seq(Some(array.len()))?;
                for value in array {
                    seq.serialize_element(value)?;
                }
                seq.end()
            }
        }
    }
}

struct TJSVariantVisitor;

impl<'de> Visitor<'de> for TJSVariantVisitor {
    type Value = TJSVariant;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a TJS void, integer, real, octet, string, dictionary, or array")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(TJSVariant::Void)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(TJSVariant::Void)
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(TJSVariant::Integer(if value { 1 } else { 0 }))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(TJSVariant::Integer(value))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        i64::try_from(value)
            .map(TJSVariant::Integer)
            .map_err(|_| E::custom("unsigned integer does not fit in a TJS integer"))
    }

    fn visit_i128<E>(self, value: i128) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        i64::try_from(value)
            .map(TJSVariant::Integer)
            .map_err(|_| E::custom("integer does not fit in a TJS integer"))
    }

    fn visit_u128<E>(self, value: u128) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        i64::try_from(value)
            .map(TJSVariant::Integer)
            .map_err(|_| E::custom("unsigned integer does not fit in a TJS integer"))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(TJSVariant::Real(value))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(TJSVariant::String(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(TJSVariant::String(value))
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(TJSVariant::Octet(value.to_vec()))
    }

    fn visit_byte_buf<E>(self, value: Vec<u8>) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(TJSVariant::Octet(value))
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut array = Vec::with_capacity(sequence.size_hint().unwrap_or(0));
        while let Some(value) = sequence.next_element()? {
            array.push(value);
        }
        Ok(TJSVariant::Array(array))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut object = HashMap::with_capacity(map.size_hint().unwrap_or(0));
        while let Some((key, value)) = map.next_entry::<String, TJSVariant>()? {
            object.insert(key, value);
        }
        Ok(TJSVariant::Dict(object))
    }
}

impl<'de> Deserialize<'de> for TJSVariant {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(TJSVariantVisitor)
    }
}

impl TryFrom<&mut tTJSVariant> for TJSVariant {
    type Error = crate::de::DeserError;

    fn try_from(value: &mut tTJSVariant) -> Result<Self, Self::Error> {
        Self::from_tjs(value)
    }
}

impl TryFrom<&TJSVariant> for tTJSVariant {
    type Error = crate::ser::SerError;

    fn try_from(value: &TJSVariant) -> Result<Self, Self::Error> {
        value.to_tjs()
    }
}

impl TryFrom<TJSVariant> for tTJSVariant {
    type Error = crate::ser::SerError;

    fn try_from(value: TJSVariant) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl From<i64> for TJSVariant {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

macro_rules! impl_integer_from {
    ($($ty:ty),* $(,)?) => {
        $(
            impl From<$ty> for TJSVariant {
                fn from(value: $ty) -> Self {
                    Self::Integer(value as i64)
                }
            }
        )*
    };
}

impl_integer_from!(i8, i16, i32, u8, u16, u32, u64, usize, isize);

impl From<bool> for TJSVariant {
    fn from(value: bool) -> Self {
        Self::Integer(if value { 1 } else { 0 })
    }
}

impl From<()> for TJSVariant {
    fn from(_: ()) -> Self {
        Self::Void
    }
}

impl From<&[u8]> for TJSVariant {
    fn from(value: &[u8]) -> Self {
        Self::Octet(value.to_vec())
    }
}

impl From<f64> for TJSVariant {
    fn from(value: f64) -> Self {
        Self::Real(value)
    }
}

impl From<String> for TJSVariant {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for TJSVariant {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<Vec<u8>> for TJSVariant {
    fn from(value: Vec<u8>) -> Self {
        Self::Octet(value)
    }
}

impl From<Vec<TJSVariant>> for TJSVariant {
    fn from(value: Vec<TJSVariant>) -> Self {
        Self::Array(value)
    }
}

impl From<HashMap<String, TJSVariant>> for TJSVariant {
    fn from(value: HashMap<String, TJSVariant>) -> Self {
        Self::Dict(value)
    }
}
