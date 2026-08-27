//! Convert tTJSVariant to other types
use super::*;
use crate::types::*;
use std::fmt::Display;
use std::ptr;

/// Convert tTJSVariant to other types
pub trait TjsParam<'a>: 'a + Sized {
    type Error: Display;
    /// Convert tTJSVariant to other types
    fn to_param(param: &'a mut tTJSVariant) -> Result<Self, Self::Error>;
}

#[derive(Debug)]
pub struct TypeError(pub &'static str);

impl Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "data is not a {}.", self.0)
    }
}

impl<'a> TjsParam<'a> for String {
    type Error = TypeError;
    fn to_param(param: &mut tTJSVariant) -> Result<Self, Self::Error> {
        if param.is_string() {
            let s = param.as_string_no_add_ref();
            if s.is_null() {
                return Err(TypeError("string"));
            }
            let ss = ttstr::from(s);
            Ok(ss.to_string())
        } else {
            Err(TypeError("string"))
        }
    }
}

impl<'a> TjsParam<'a> for i64 {
    type Error = TypeError;
    fn to_param(param: &mut tTJSVariant) -> Result<Self, Self::Error> {
        if param.can_as_integer() {
            Ok(param.as_integer())
        } else {
            Err(TypeError("integer"))
        }
    }
}

impl<'a> TjsParam<'a> for f64 {
    type Error = TypeError;
    fn to_param(param: &mut tTJSVariant) -> Result<Self, Self::Error> {
        if param.can_as_real() {
            Ok(param.as_real())
        } else {
            Err(TypeError("real"))
        }
    }
}

impl<'a> TjsParam<'a> for Octet {
    type Error = TypeError;
    fn to_param(param: &'a mut tTJSVariant) -> Result<Self, Self::Error> {
        if param.is_octet() {
            let val = param.as_octet();
            Ok(unsafe { Octet::new_owned(val) })
        } else {
            Err(TypeError("octet"))
        }
    }
}

impl<'a> TjsParam<'a> for i128 {
    type Error = TypeError;
    #[allow(non_upper_case_globals)]
    fn to_param(param: &mut tTJSVariant) -> Result<Self, Self::Error> {
        match param.typ() {
            tTJSVariantType_tvtInteger | tTJSVariantType_tvtReal => Ok(param.as_integer() as i128),
            tTJSVariantType_tvtString => {
                let s = param.as_string_no_add_ref();
                if s.is_null() {
                    return Err(TypeError("string"));
                }
                let ss = ttstr::from(s).to_string();
                ss.parse().map_err(|_| TypeError("big integer"))
            }
            _ => Err(TypeError("big integer")),
        }
    }
}

impl<'a> TjsParam<'a> for &'a mut tTJSVariant {
    type Error = TypeError;
    fn to_param(param: &'a mut tTJSVariant) -> Result<Self, Self::Error> {
        Ok(param)
    }
}

impl<'a> TjsParam<'a> for *mut tTJSVariant {
    type Error = TypeError;
    fn to_param(param: &'a mut tTJSVariant) -> Result<Self, Self::Error> {
        Ok(param)
    }
}

impl<'a> TjsParam<'a> for ttstr {
    type Error = TypeError;
    fn to_param(param: &mut tTJSVariant) -> Result<Self, Self::Error> {
        if param.is_string() {
            let s = param.as_string_no_add_ref();
            if s.is_null() {
                return Err(TypeError("string"));
            }
            let ss = ttstr::from(s);
            Ok(ss)
        } else {
            Err(TypeError("string"))
        }
    }
}

pub enum VecError<E: Display> {
    Type(TypeError),
    GetItem(i64),
    Item((i64, E)),
}

impl<E: Display> VecError<E> {
    pub fn new_item(index: i64, err: E) -> Self {
        Self::Item((index, err))
    }
}

impl<E: Display> From<TypeError> for VecError<E> {
    fn from(value: TypeError) -> Self {
        Self::Type(value)
    }
}

impl<E: Display> Display for VecError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Type(typ) => typ.fmt(f),
            Self::GetItem(ind) => write!(f, "Failed to get item {}", ind),
            Self::Item((ind, err)) => write!(f, "Failed to decode item {}: {}", ind, err),
        }
    }
}

impl<'a, T, E> TjsParam<'a> for Vec<T>
where
    T: for<'b> TjsParam<'b, Error = E> + 'static,
    E: Display,
{
    type Error = VecError<E>;
    fn to_param(param: &'a mut tTJSVariant) -> Result<Self, Self::Error> {
        if param.is_array() {
            let mut count = tTJSVariant::new();
            let obj = param.as_object_no_add_ref();
            let err = unsafe {
                (*obj).prop_get(
                    TJS_MEMBERMUSTEXIST as u32,
                    tjs_w!("count"),
                    ptr::null_mut(),
                    &mut count,
                    obj,
                )
            };
            if TJS_FAILED(err) {
                return Err(TypeError("array").into());
            }
            let count = count.as_integer();
            let mut result = Vec::with_capacity(count as usize);
            for ind in 0..count {
                let mut val = tTJSVariant::new();
                let err = unsafe {
                    (*obj).prop_get_by_num(
                        TJS_MEMBERMUSTEXIST as u32,
                        ind as tjs_int,
                        &mut val,
                        obj,
                    )
                };
                if TJS_FAILED(err) {
                    return Err(VecError::GetItem(ind));
                }
                let re = match T::to_param(&mut val) {
                    Ok(t) => t,
                    Err(e) => {
                        return Err(VecError::new_item(ind, e));
                    }
                };
                result.push(re);
            }
            Ok(result)
        } else {
            Err(TypeError("array").into())
        }
    }
}
