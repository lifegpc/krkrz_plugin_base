//! Convert tTJSVariant to other types
use super::*;
use std::fmt::Display;

/// Convert tTJSVariant to other types
pub trait TjsParam: Sized {
    type Error: Display;
    /// Convert tTJSVariant to other types
    fn to_param(param: &mut tTJSVariant) -> Result<Self, Self::Error>;
}

pub struct TypeError(&'static str);

impl Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "data is not a {}.", self.0)
    }
}

impl TjsParam for String {
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

impl TjsParam for i64 {
    type Error = TypeError;
    fn to_param(param: &mut tTJSVariant) -> Result<Self, Self::Error> {
        if param.can_as_integer() {
            Ok(param.as_integer())
        } else {
            Err(TypeError("integer"))
        }
    }
}
