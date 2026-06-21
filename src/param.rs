//! Convert tTJSVariant to other types
use super::*;
use std::fmt::Display;

/// Convert tTJSVariant to other types
pub trait TjsParam<'a>: 'a + Sized {
    type Error: Display;
    /// Convert tTJSVariant to other types
    fn to_param(param: &'a mut tTJSVariant) -> Result<Self, Self::Error>;
}

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
