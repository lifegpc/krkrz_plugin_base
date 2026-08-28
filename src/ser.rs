use crate::de::DictKeyCollector;
use crate::{tp_stub::*, types::*};
use krkrz_plugin_base_macros::tjs_w;
use serde::ser::{
    Error, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant,
};
use serde::{Serialize, Serializer};
use std::ptr;

pub fn to<T: Serialize>(value: &T) -> Result<tTJSVariant, SerError> {
    let mut v = tTJSVariant::new();
    value.serialize(TjsSerializer::new(&mut v))?;
    Ok(v)
}

#[derive(Debug)]
pub enum SerError {
    Custom(String),
}

impl std::fmt::Display for SerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Custom(s) => f.write_str(&s),
        }
    }
}

impl std::error::Error for SerError {}

impl Error for SerError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Custom(format!("{}", msg))
    }
}

pub struct TjsSerializer<'a> {
    obj: &'a mut tTJSVariant,
}

impl<'a> TjsSerializer<'a> {
    pub fn new(value: &'a mut tTJSVariant) -> Self {
        Self { obj: value }
    }
}

pub struct TjsSeqSerializer {
    obj: *mut iTJSDispatch2,
}

impl SerializeSeq for TjsSeqSerializer {
    type Ok = ();
    type Error = SerError;
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let mut v = tTJSVariant::new();
        value.serialize(TjsSerializer::new(&mut v))?;
        let mut args = [&mut v as *mut _];
        let r = unsafe {
            (*self.obj).func_call(
                0,
                tjs_w!("add"),
                ptr::null_mut(),
                ptr::null_mut(),
                1,
                args.as_mut_ptr(),
                self.obj,
            )
        };
        if TJS_FAILED(r) {
            return Err(SerError::custom("Failed to add item to array."));
        }
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeTuple for TjsSeqSerializer {
    type Ok = ();
    type Error = SerError;
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeTupleStruct for TjsSeqSerializer {
    type Ok = ();
    type Error = SerError;
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeTupleVariant for TjsSeqSerializer {
    type Ok = ();
    type Error = SerError;
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

pub struct TjsDictSerializer {
    obj: *mut iTJSDispatch2,
    key: Option<ttstr>,
}

impl SerializeMap for TjsDictSerializer {
    type Ok = ();
    type Error = SerError;
    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let mut var = tTJSVariant::new();
        key.serialize(TjsSerializer::new(&mut var))?;
        if !var.can_as_string() {
            return Err(SerError::custom("Key can not cast as string"));
        }
        let s = var.as_string();
        if s.is_null() {
            return Err(SerError::custom("Key can not cast as string"));
        }
        let key = ttstr::from(s);
        unsafe { (*s).release() };
        self.key = Some(key);
        Ok(())
    }
    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        if let Some(key) = &self.key {
            let mut v = tTJSVariant::new();
            value.serialize(TjsSerializer::new(&mut v))?;
            let r = unsafe {
                (*self.obj).prop_set(
                    TJS_MEMBERENSURE as u32,
                    key.c_str(),
                    ptr::null_mut(),
                    &v,
                    self.obj,
                )
            };
            if TJS_FAILED(r) {
                return Err(SerError::custom(format!(
                    "Failed to set item {} for dict.",
                    key.to_string()
                )));
            }
            Ok(())
        } else {
            panic!("serialize_key must called before calling serialize_value");
        }
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeStruct for TjsSeqSerializer {
    type Ok = ();
    type Error = SerError;
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let k = ttstr::from(key);
        let mut v = tTJSVariant::new();
        value.serialize(TjsSerializer::new(&mut v))?;
        let r = unsafe {
            (*self.obj).prop_set(
                TJS_MEMBERENSURE as u32,
                k.c_str(),
                ptr::null_mut(),
                &v,
                self.obj,
            )
        };
        if TJS_FAILED(r) {
            return Err(SerError::custom(format!(
                "Failed to set item {} for dict.",
                key
            )));
        }
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeStructVariant for TjsSeqSerializer {
    type Ok = ();
    type Error = SerError;
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        SerializeStruct::serialize_field(self, key, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> Serializer for TjsSerializer<'a> {
    type Ok = ();
    type Error = SerError;
    type SerializeSeq = TjsSeqSerializer;
    type SerializeTuple = TjsSeqSerializer;
    type SerializeTupleStruct = TjsSeqSerializer;
    type SerializeTupleVariant = TjsSeqSerializer;
    type SerializeMap = TjsDictSerializer;
    type SerializeStruct = TjsSeqSerializer;
    type SerializeStructVariant = TjsSeqSerializer;
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.obj.assign(v);
        Ok(())
    }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.obj.assign(v as i64);
        Ok(())
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.obj.assign(v as i64);
        Ok(())
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.obj.assign(v as i64);
        Ok(())
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.obj.assign(v);
        Ok(())
    }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.obj.assign(v as i64);
        Ok(())
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.obj.assign(v as i64);
        Ok(())
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.obj.assign(v as i64);
        Ok(())
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.obj.assign(v as i64);
        Ok(())
    }
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.obj.assign(v as f64);
        Ok(())
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.obj.assign(v);
        Ok(())
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        // According to serde document, serialize it to u32
        self.serialize_u32(v as u32)
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.obj.assign(v);
        Ok(())
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let mut octet = tTJSVariantOctet::from(v);
        self.obj.assign(&mut octet as *mut _);
        Ok(())
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.obj.assign(());
        Ok(())
    }
    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.obj.assign(());
        Ok(())
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.obj.assign(());
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let mut s = ttstr::from(tjs_w!("return new Dictionary();"));
        unsafe { TVPExecuteScript(&mut s, self.obj) };
        let obj = self.obj.as_object_no_add_ref();
        if obj.is_null() {
            return Err(SerError::custom(
                "Failed to create dictionary for newtype variant.",
            ));
        }
        let mut inner_val = tTJSVariant::new();
        value.serialize(TjsSerializer::new(&mut inner_val))?;
        let key = ttstr::from(variant);
        let r = unsafe {
            (*obj).prop_set(
                TJS_MEMBERENSURE as u32,
                key.c_str(),
                ptr::null_mut(),
                &inner_val,
                obj,
            )
        };
        if TJS_FAILED(r) {
            return Err(SerError::custom(format!(
                "Failed to set variant {} to dictionary.",
                variant
            )));
        }
        Ok(())
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let mut s = ttstr::from(tjs_w!("return new Array();"));
        unsafe { TVPExecuteScript(&mut s, self.obj) };
        let obj = self.obj.as_object_no_add_ref();
        if obj.is_null() {
            return Err(SerError::custom("Failed to create array for seq."));
        }
        Ok(TjsSeqSerializer { obj })
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        let mut s = ttstr::from(tjs_w!("return new Array();"));
        unsafe { TVPExecuteScript(&mut s, self.obj) };
        let obj = self.obj.as_object_no_add_ref();
        if obj.is_null() {
            return Err(SerError::custom("Failed to create array for seq."));
        }
        Ok(TjsSeqSerializer { obj })
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        let mut s = ttstr::from(tjs_w!("return new Array();"));
        unsafe { TVPExecuteScript(&mut s, self.obj) };
        let obj = self.obj.as_object_no_add_ref();
        if obj.is_null() {
            return Err(SerError::custom("Failed to create array for seq."));
        }
        Ok(TjsSeqSerializer { obj })
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        let mut s = ttstr::from(tjs_w!("return new Dictionary();"));
        unsafe { TVPExecuteScript(&mut s, self.obj) };
        let obj = self.obj.as_object_no_add_ref();
        if obj.is_null() {
            return Err(SerError::custom(
                "Failed to create dictionary for tuple variant.",
            ));
        }
        let key = ttstr::from(variant);
        let mut inner = tTJSVariant::new();
        s = ttstr::from(tjs_w!("return new Array();"));
        unsafe { TVPExecuteScript(&mut s, &mut inner) };
        let inner_obj = inner.as_object_no_add_ref();
        if inner_obj.is_null() {
            return Err(SerError::custom(
                "Failed to create array for tuple variant.",
            ));
        }
        let r = unsafe {
            (*obj).prop_set(
                TJS_MEMBERENSURE as u32,
                key.c_str(),
                ptr::null_mut(),
                &inner,
                obj,
            )
        };
        if TJS_FAILED(r) {
            return Err(SerError::custom(format!(
                "Failed to set tuple variant {} to dictionary.",
                variant
            )));
        }
        Ok(TjsSeqSerializer { obj: inner_obj })
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let mut s = ttstr::from(tjs_w!("return new Dictionary();"));
        unsafe { TVPExecuteScript(&mut s, self.obj) };
        let obj = self.obj.as_object_no_add_ref();
        if obj.is_null() {
            return Err(SerError::custom("Failed to create dictionary for map."));
        }
        Ok(TjsDictSerializer { obj, key: None })
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        let mut s = ttstr::from(tjs_w!("return new Dictionary();"));
        unsafe { TVPExecuteScript(&mut s, self.obj) };
        let obj = self.obj.as_object_no_add_ref();
        if obj.is_null() {
            return Err(SerError::custom("Failed to create dictionary for struct."));
        }
        Ok(TjsSeqSerializer { obj })
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        let mut s = ttstr::from(tjs_w!("return new Dictionary();"));
        unsafe { TVPExecuteScript(&mut s, self.obj) };
        let obj = self.obj.as_object_no_add_ref();
        if obj.is_null() {
            return Err(SerError::custom(
                "Failed to create dictionary for struct variant.",
            ));
        }
        let key = ttstr::from(variant);
        let mut inner = tTJSVariant::new();
        unsafe { TVPExecuteScript(&mut s, &mut inner) };
        let inner_obj = inner.as_object_no_add_ref();
        if inner_obj.is_null() {
            return Err(SerError::custom(
                "Failed to create inner dictionary for tuple variant.",
            ));
        }
        let r = unsafe {
            (*obj).prop_set(
                TJS_MEMBERENSURE as u32,
                key.c_str(),
                ptr::null_mut(),
                &inner,
                obj,
            )
        };
        if TJS_FAILED(r) {
            return Err(SerError::custom(format!(
                "Failed to set tuple variant {} to dictionary.",
                variant
            )));
        }
        Ok(TjsSeqSerializer { obj: inner_obj })
    }
}

impl Serialize for tTJSVariant {
    #[allow(non_upper_case_globals)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.typ() {
            tTJSVariantType_tvtVoid => serializer.serialize_unit(),
            tTJSVariantType_tvtInteger => serializer.serialize_i64(self.as_integer()),
            tTJSVariantType_tvtReal => serializer.serialize_f64(self.as_real()),
            tTJSVariantType_tvtString => {
                let s = self.as_string_no_add_ref();
                if s.is_null() {
                    return Err(S::Error::custom("String is null"));
                }
                let s = ttstr::from(s);
                serializer.serialize_str(&s.to_string())
            }
            tTJSVariantType_tvtOctet => {
                let o = self.as_octet();
                if o.is_null() {
                    return Err(S::Error::custom("octet is null"));
                }
                let o = unsafe { Octet::new_owned(o) };
                serializer.serialize_bytes(&o)
            }
            tTJSVariantType_tvtObject => {
                if self.is_array() {
                    let obj = self.as_object_no_add_ref();
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
                        return Err(S::Error::custom("data is not a array"));
                    }
                    let count = val.as_integer();
                    let mut seq = serializer.serialize_seq(Some(count as usize))?;
                    for i in 0..count {
                        let mut value = tTJSVariant::new();
                        let r = unsafe {
                            (*obj).prop_get_by_num(
                                TJS_MEMBERMUSTEXIST as u32,
                                i as i32,
                                &mut value,
                                obj,
                            )
                        };
                        if TJS_FAILED(r) {
                            return Err(S::Error::custom(format!("Failed to get item {}", i)));
                        }
                        seq.serialize_element(&value)?;
                    }
                    seq.end()
                } else if self.is_dict() {
                    let obj = self.as_object_no_add_ref();
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
                        return Err(S::Error::custom("data is not a dict"));
                    }
                    let mut map = serializer.serialize_map(Some(dict.borrow().len()))?;
                    for key in dict.borrow().iter() {
                        let mut value = tTJSVariant::new();
                        let s = ttstr::from(key);
                        let r = unsafe {
                            (*obj).prop_get(
                                TJS_MEMBERMUSTEXIST as u32,
                                s.c_str(),
                                ptr::null_mut(),
                                &mut value,
                                obj,
                            )
                        };
                        if TJS_FAILED(r) {
                            return Err(S::Error::custom(format!("Failed to get item {}", key)));
                        }
                        map.serialize_entry(key, &value)?;
                    }
                    map.end()
                } else {
                    let s = self.as_string();
                    if s.is_null() {
                        return Err(S::Error::custom("data is not a string"));
                    }
                    let t = ttstr::from(s);
                    unsafe { (*s).release() };
                    serializer.serialize_str(&t.to_string())
                }
            }
            _ => Err(S::Error::custom("other types is not supported.")),
        }
    }
}
