use crate::{param::*, tp_stub::*};
use krkrz_plugin_base_macros::tjs_w;
use serde::de::{
    EnumAccess, Error, IntoDeserializer, MapAccess, SeqAccess, VariantAccess, Visitor,
};
use serde::{Deserialize, Deserializer};
use std::cell::RefCell;
use std::collections::HashSet;
use std::ptr;
use std::rc::Rc;

pub fn from<'de, T: Deserialize<'de>>(param: &mut tTJSVariant) -> Result<T, DeserError> {
    let de = TjsDeserializer::new(param);
    T::deserialize(de)
}

macro_rules! impl_int_deser {
    ($method:ident, $ty:ty, $visit:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            if self.value.can_as_integer() {
                let v = self.value.as_integer();
                let val = <$ty>::try_from(v).map_err(DeserError::custom)?;
                visitor.$visit(val)
            } else {
                Err(TypeError("integer").into())
            }
        }
    };
}

pub struct TjsDeserializer<'a> {
    value: &'a mut tTJSVariant,
}

struct TjsSeq {
    value: *mut iTJSDispatch2,
    count: i64,
    current: i64,
}

impl<'de> SeqAccess<'de> for TjsSeq {
    type Error = DeserError;
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if self.current < self.count {
            let mut value = tTJSVariant::new();
            let r = unsafe {
                (*self.value).prop_get_by_num(
                    TJS_MEMBERMUSTEXIST as u32,
                    self.current as i32,
                    &mut value,
                    self.value,
                )
            };
            if TJS_FAILED(r) {
                return Err(DeserError::custom(format!(
                    "Failed to get item {}",
                    self.current
                )));
            }
            self.current += 1;
            let de = TjsDeserializer { value: &mut value };
            seed.deserialize(de).map(|s| Some(s))
        } else {
            Ok(None)
        }
    }
    fn size_hint(&self) -> Option<usize> {
        Some(self.count as usize)
    }
}

impl<'a> TjsDeserializer<'a> {
    pub fn new(value: &'a mut tTJSVariant) -> Self {
        Self { value }
    }

    fn get_str_slice(&self) -> Result<&[u8], DeserError> {
        if self.value.is_octet() {
            let r = unsafe { self.value.as_octet_no_add_ref().as_ref() };
            if let Some(r) = r {
                let v = r.get_data();
                let len = r.get_length() as usize;
                Ok(unsafe { std::slice::from_raw_parts(v, len) })
            } else {
                Ok(b"")
            }
        } else {
            Err(TypeError("octet").into())
        }
    }
}

#[derive(Default)]
struct DictKeyCollector(Rc<RefCell<HashSet<String>>>);

impl TJSDispatch for DictKeyCollector {
    fn func_call(
        &mut self,
        _flag: tjs_uint32,
        _membername: *const tjs_char,
        _hint: *mut tjs_uint32,
        result: *mut tTJSVariant,
        numparams: tjs_int,
        param: *mut *mut tTJSVariant,
        _objthis: *mut iTJSDispatch2,
    ) -> tjs_error {
        if numparams > 1 {
            let p = unsafe { &mut *(*param.add(1)) };
            let flag = p.as_integer();
            if flag != TJS_HIDDENMEMBER as i64 {
                let key = unsafe { &mut *(*param.add(0)) };
                let s = key.as_string_no_add_ref();
                let t = ttstr::from(s);
                self.0.borrow_mut().insert(t.to_string());
            }
        }
        if let Some(r) = unsafe { result.as_mut() } {
            r.assign(true);
        }
        TJS_S_OK
    }
}

struct TjsMap {
    keys: Vec<String>,
    key_current: usize,
    last_key: Option<usize>,
    obj: *mut iTJSDispatch2,
}

impl<'de> MapAccess<'de> for TjsMap {
    type Error = DeserError;
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        if self.key_current < self.keys.len() {
            let key = self.keys[self.key_current].as_str();
            self.last_key = Some(self.key_current);
            self.key_current += 1;
            seed.deserialize(key.into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        if let Some(key) = self.last_key {
            let mut value = tTJSVariant::new();
            let key = &self.keys[key];
            let s = ttstr::from(key);
            let r = unsafe {
                (*self.obj).prop_get(
                    TJS_MEMBERMUSTEXIST as u32,
                    s.c_str(),
                    ptr::null_mut(),
                    &mut value,
                    self.obj,
                )
            };
            if TJS_FAILED(r) {
                return Err(DeserError::custom(format!("Failed to get item {}", key)));
            }
            let de = TjsDeserializer { value: &mut value };
            seed.deserialize(de)
        } else {
            crate::throw_exception_message!("Failed to deserialize.");
        }
    }
}

struct TjsEnum<'a> {
    obj: &'a mut tTJSVariant,
    inner_value: tTJSVariant,
}

impl<'de, 'a> EnumAccess<'de> for TjsEnum<'a> {
    type Error = DeserError;
    type Variant = Self;
    fn variant_seed<V>(mut self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        if self.obj.is_string() {
            let s = self.obj.as_string_no_add_ref();
            let s = ttstr::from(s);
            seed.deserialize(s.to_string().into_deserializer())
                .map(|val| (val, self))
        } else if self.obj.is_dict() {
            let obj = self.obj.as_object_no_add_ref();
            let collector = DictKeyCollector::default();
            let dict = collector.0.clone();
            let collector_disp = tTJSDispatch::new(collector);
            let mut val = tTJSVariant::new();
            val.assign(collector_disp);
            unsafe { (*collector_disp).release() };
            let mut closure = tTJSVariantClosure::new(collector_disp, ptr::null_mut());
            let r = unsafe {
                (*obj).enum_members(TJS_IGNOREPROP as u32, &mut closure, ptr::null_mut())
            };
            if TJS_FAILED(r) {
                return Err(TypeError("enum dict").into());
            }
            let keys = dict.borrow();
            let key_str = keys
                .iter()
                .next()
                .ok_or_else(|| DeserError::custom("Enum dict must have at least one key"))?
                .to_owned();
            let s = ttstr::from(&key_str);
            let r = unsafe {
                (*obj).prop_get(
                    TJS_MEMBERMUSTEXIST as u32,
                    s.c_str(),
                    ptr::null_mut(),
                    &mut self.inner_value,
                    obj,
                )
            };
            if TJS_FAILED(r) {
                return Err(DeserError::custom(format!(
                    "Failed to get enum variant payload for {}",
                    key_str
                )));
            }
            seed.deserialize(key_str.into_deserializer())
                .map(|val| (val, self))
        } else {
            Err(TypeError("enum").into())
        }
    }
}

impl<'de, 'a> VariantAccess<'de> for TjsEnum<'a> {
    type Error = DeserError;
    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn newtype_variant_seed<T>(mut self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        let de = TjsDeserializer {
            value: &mut self.inner_value,
        };
        seed.deserialize(de)
    }
    fn tuple_variant<V>(mut self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let de = TjsDeserializer {
            value: &mut self.inner_value,
        };
        de.deserialize_tuple(len, visitor)
    }
    fn struct_variant<V>(
        mut self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let de = TjsDeserializer {
            value: &mut self.inner_value,
        };
        de.deserialize_map(visitor)
    }
}

#[derive(Debug)]
pub enum DeserError {
    Type(TypeError),
    Custom(String),
    NotSupported(&'static str),
}

impl From<TypeError> for DeserError {
    fn from(value: TypeError) -> Self {
        Self::Type(value)
    }
}

impl std::fmt::Display for DeserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Type(t) => t.fmt(f),
            Self::Custom(s) => f.write_str(&s),
            Self::NotSupported(s) => write!(f, "{s} is not supported."),
        }
    }
}

impl std::error::Error for DeserError {}

impl serde::de::Error for DeserError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Custom(format!("{}", msg))
    }
}

impl<'de, 'a> Deserializer<'de> for TjsDeserializer<'a> {
    type Error = DeserError;
    #[allow(non_upper_case_globals)]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value.typ() {
            tTJSVariantType_tvtVoid => self.deserialize_unit(visitor),
            tTJSVariantType_tvtString => self.deserialize_string(visitor),
            tTJSVariantType_tvtInteger => self.deserialize_i64(visitor),
            tTJSVariantType_tvtReal => self.deserialize_f64(visitor),
            tTJSVariantType_tvtOctet => self.deserialize_bytes(visitor),
            tTJSVariantType_tvtObject => {
                if self.value.is_array() {
                    self.deserialize_seq(visitor)
                } else if self.value.is_dict() {
                    self.deserialize_map(visitor)
                } else {
                    self.deserialize_string(visitor)
                }
            }
            _ => Err(DeserError::NotSupported("Unknown types")),
        }
    }
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_integer() {
            let v = self.value.as_integer();
            if v == 1 {
                visitor.visit_bool(true)
            } else if v == 0 {
                visitor.visit_bool(false)
            } else {
                Err(TypeError("boolean").into())
            }
        } else {
            Err(TypeError("boolean").into())
        }
    }
    impl_int_deser!(deserialize_i8, i8, visit_i8);
    impl_int_deser!(deserialize_i16, i16, visit_i16);
    impl_int_deser!(deserialize_i32, i32, visit_i32);
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.can_as_integer() {
            let v = self.value.as_integer();
            visitor.visit_i64(v)
        } else {
            Err(TypeError("integer").into())
        }
    }
    impl_int_deser!(deserialize_u8, u8, visit_u8);
    impl_int_deser!(deserialize_u16, u16, visit_u16);
    impl_int_deser!(deserialize_u32, u32, visit_u32);
    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.can_as_integer() {
            let v = self.value.as_integer();
            // Cast i64 to u64 will keep same behavior which krkrz does.
            // DO NOT CHANGE THIS TO TryFrom
            visitor.visit_u64(v as u64)
        } else {
            Err(TypeError("integer").into())
        }
    }
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.can_as_real() {
            let v = self.value.as_real();
            visitor.visit_f32(v as f32)
        } else {
            Err(TypeError("real").into())
        }
    }
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.can_as_real() {
            let v = self.value.as_real();
            visitor.visit_f64(v)
        } else {
            Err(TypeError("real").into())
        }
    }
    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(DeserError::NotSupported("char"))
    }
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_string() {
            let r = self.value.as_string_no_add_ref();
            if r.is_null() {
                return Err(TypeError("string").into());
            }
            let s = ttstr::from(r);
            visitor.visit_string(s.to_string())
        } else {
            Err(TypeError("string").into())
        }
    }
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bytes(self.get_str_slice()?)
    }
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_byte_buf(self.get_str_slice()?.to_vec())
    }
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_void() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_void() {
            visitor.visit_unit()
        } else {
            Err(TypeError("void").into())
        }
    }
    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_array() {
            let obj = self.value.as_object_no_add_ref();
            let mut val = tTJSVariant::new();
            let r = unsafe {
                (*obj).prop_get(
                    TJS_MEMBERMUSTEXIST as u32,
                    tjs_w!("count"),
                    ptr::null_mut(),
                    &mut val,
                    obj,
                )
            };
            if TJS_FAILED(r) {
                return Err(TypeError("array").into());
            }
            let seq = TjsSeq {
                value: obj,
                count: val.as_integer(),
                current: 0,
            };
            visitor.visit_seq(seq)
        } else {
            Err(TypeError("array").into())
        }
    }
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_dict() {
            let obj = self.value.as_object_no_add_ref();
            let collector = DictKeyCollector::default();
            let dict = collector.0.clone();
            let collector = tTJSDispatch::new(collector);
            let mut val = tTJSVariant::new();
            val.assign(collector);
            unsafe { (*collector).release() };
            let mut collector = tTJSVariantClosure::new(collector, ptr::null_mut());
            let r = unsafe {
                (*obj).enum_members(TJS_IGNOREPROP as u32, &mut collector, ptr::null_mut())
            };
            if TJS_FAILED(r) {
                return Err(TypeError("dict").into());
            }
            let keys = dict.borrow().iter().map(|s| s.to_owned()).collect();
            let v = TjsMap {
                keys,
                key_current: 0,
                last_key: None,
                obj,
            };
            visitor.visit_map(v)
        } else {
            Err(TypeError("dict").into())
        }
    }
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }
    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let de = TjsEnum {
            obj: self.value,
            inner_value: tTJSVariant::new(),
        };
        visitor.visit_enum(de)
    }
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }
}

impl<'de> Visitor<'de> for tTJSVariant {
    type Value = Self;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "boolean/int/float/boolean/dict/array")
    }
    fn visit_bool<E>(mut self, v: bool) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.assign(if v { 1 } else { 0 });
        Ok(self)
    }
    fn visit_i64<E>(mut self, v: i64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.assign(v);
        Ok(self)
    }
    fn visit_u64<E>(mut self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.assign(v as i64);
        Ok(self)
    }
    fn visit_f32<E>(mut self, v: f32) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.assign(v as f64);
        Ok(self)
    }
    fn visit_f64<E>(mut self, v: f64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.assign(v);
        Ok(self)
    }
    fn visit_str<E>(mut self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.assign(v);
        Ok(self)
    }
    fn visit_bytes<E>(mut self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let mut v = tTJSVariantOctet::from(v);
        self.assign((&mut v) as *mut _);
        Ok(self)
    }
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Self::new())
    }
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Self::new())
    }
    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut s = ttstr::from(tjs_w!("return new Array();"));
        unsafe { TVPExecuteScript(&mut s, &mut self) };
        let obj = self.as_object_no_add_ref();
        if obj.is_null() {
            return Err(A::Error::custom("Failed to create array."));
        }
        while let Some(mut ele) = seq.next_element::<tTJSVariant>()? {
            let mut args = [&mut ele as *mut _];
            let r = unsafe {
                (*obj).func_call(
                    0,
                    tjs_w!("add"),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    1,
                    args.as_mut_ptr(),
                    obj,
                )
            };
            if TJS_FAILED(r) {
                return Err(A::Error::custom("Failed to add item to array."));
            }
        }
        Ok(self)
    }
    fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut s = ttstr::from(tjs_w!("return new Dictionary();"));
        unsafe { TVPExecuteScript(&mut s, &mut self) };
        let obj = self.as_object_no_add_ref();
        if obj.is_null() {
            return Err(A::Error::custom("Failed to create dictionary."));
        }
        while let Some((k, v)) = map.next_entry::<&str, tTJSVariant>()? {
            let key = ttstr::from(k);
            let r = unsafe {
                (*obj).prop_set(
                    TJS_MEMBERENSURE as u32,
                    key.c_str(),
                    ptr::null_mut(),
                    &v,
                    obj,
                )
            };
            if TJS_FAILED(r) {
                return Err(A::Error::custom(format!(
                    "Failed to set key {} to dictionary.",
                    k
                )));
            }
        }
        Ok(self)
    }
}

impl<'de> Deserialize<'de> for tTJSVariant {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(Self::new())
    }
}
