use krkrz_plugin_base::{tp_stub::*, *};
use std::cell::RefCell;
use std::fmt::Write as _;
use std::ptr;
use std::rc::Rc;

struct DM2 {}

impl TJSDispatch for DM2 {
    fn func_call(
        &mut self,
        _flag: tjs_uint32,
        membername: *const tjs_char,
        _hint: *mut tjs_uint32,
        result: *mut tTJSVariant,
        numparams: tjs_int,
        param: *mut *mut tTJSVariant,
        _objthis: *mut iTJSDispatch2,
    ) -> tjs_error {
        if !membername.is_null() {
            return TJS_E_MEMBERNOTFOUND;
        }

        let mut message = String::new();
        if numparams > 0 {
            if param.is_null() {
                // Some callers pass `dm2(void)` as a null parameter array.
                for index in 0..numparams {
                    if index > 0 {
                        message.push(' ');
                    }
                    message.push_str("void");
                }
            } else {
                for index in 0..numparams {
                    if index > 0 {
                        message.push(' ');
                    }
                    let value = unsafe { *param.add(index as usize) };
                    format_param(&mut message, value);
                }
            }
        }

        if !message.is_empty() {
            log!("{message}");
        }

        if let Some(result) = unsafe { result.as_mut() } {
            result.assign(());
        }

        0
    }
}

struct Stack {}

impl TJSDispatch for Stack {
    fn func_call(
        &mut self,
        _flag: tjs_uint32,
        membername: *const tjs_char,
        _hint: *mut tjs_uint32,
        result: *mut tTJSVariant,
        _numparams: tjs_int,
        _param: *mut *mut tTJSVariant,
        _objthis: *mut iTJSDispatch2,
    ) -> tjs_error {
        if !membername.is_null() {
            return TJS_E_MEMBERNOTFOUND;
        }

        if let Some(result) = unsafe { result.as_mut() } {
            result.assign(stack_trace());
        }

        TJS_S_OK
    }
}

/// Returns the current TJS stack trace, the same text as `Scripts.getTraceString()`.
fn stack_trace() -> String {
    unsafe {
        let global = TVPGetScriptDispatch();
        if global.is_null() {
            return String::new();
        }

        let trace = stack_trace_from_global(global);
        (*global).release();
        trace
    }
}

fn stack_trace_from_global(global: *mut iTJSDispatch2) -> String {
    unsafe {
        let scripts_name = ttstr::from("Scripts");
        let mut scripts = tTJSVariant::new();
        let error = (*global).prop_get(
            TJS_MEMBERMUSTEXIST as u32,
            scripts_name.c_str(),
            ptr::null_mut(),
            &mut scripts,
            global,
        );
        if TJS_FAILED(error) {
            return String::new();
        }

        let scripts_object = scripts.as_object_no_add_ref();
        if scripts_object.is_null() {
            return String::new();
        }

        let method_name = ttstr::from("getTraceString");
        let mut trace = tTJSVariant::new();
        let error = (*scripts_object).func_call(
            0,
            method_name.c_str(),
            ptr::null_mut(),
            &mut trace,
            0,
            ptr::null_mut(),
            scripts_object,
        );
        if TJS_FAILED(error) {
            return String::new();
        }

        let trace_string = trace.as_string_no_add_ref();
        if trace_string.is_null() {
            String::new()
        } else {
            ttstr::from(trace_string).to_string()
        }
    }
}

/// Formats one argument pointer. A null pointer represents `void`.
fn format_param(out: &mut String, value: *mut tTJSVariant) {
    match unsafe { value.as_mut() } {
        Some(value) => format_value(out, value),
        None => out.push_str("void"),
    }
}

fn format_value(out: &mut String, value: &mut tTJSVariant) {
    if value.is_void() {
        out.push_str("void");
    } else if value.is_integer() {
        let _ = write!(out, "(int){}", value.as_integer());
    } else if value.is_real() {
        let _ = write!(out, "(real){}", value.as_real());
    } else if value.is_string() {
        format_string(out, value);
    } else if value.is_octet() {
        format_octet(out, value);
    } else if value.is_array() {
        format_array(out, value);
    } else if value.is_dict() {
        format_dict(out, value);
    } else {
        format_object(out, value);
    }
}

fn format_object(out: &mut String, value: &tTJSVariant) {
    out.push_str("(object");
    let object = value.as_object_no_add_ref();
    if !object.is_null() {
        let mut classes = Vec::new();
        // ClassInstanceInfo lists the concrete class followed by its ancestors.
        for index in 0..64 {
            let mut class_name = tTJSVariant::new();
            let error = unsafe {
                (*object).class_instance_info(TJS_CII_GET as u32, index, &mut class_name)
            };
            if TJS_FAILED(error) || !class_name.is_string() {
                break;
            }
            let name = class_name.as_string_no_add_ref();
            if name.is_null() {
                break;
            }
            classes.push(ttstr::from(name).to_string());
        }
        if let Some((class_name, parents)) = classes.split_first() {
            let _ = write!(out, ": {class_name}");
            if !parents.is_empty() {
                let _ = write!(out, " extends {}", parents.join(" extends "));
            }
        }
    }
    out.push(')');
}

fn format_string(out: &mut String, value: &tTJSVariant) {
    out.push_str("(string)\"");
    let string = value.as_string_no_add_ref();
    if !string.is_null() {
        let string = ttstr::from(string);
        escape_c(out, &string.to_string());
    }
    out.push('"');
}

fn format_octet(out: &mut String, value: &tTJSVariant) {
    out.push_str("(octet)<% ");
    let octet = value.as_octet_no_add_ref();
    if let Some(octet) = unsafe { octet.as_ref() } {
        let data = octet.get_data();
        let length = octet.get_length() as usize;
        if length > 0 && !data.is_null() {
            let bytes = unsafe { std::slice::from_raw_parts(data, length) };
            for (index, byte) in bytes.iter().enumerate() {
                if index > 0 {
                    out.push(' ');
                }
                let _ = write!(out, "{byte:02X}");
            }
        }
    }
    out.push_str(" %>");
}

fn format_array(out: &mut String, value: &tTJSVariant) {
    out.push_str("(array)[");
    let object = value.as_object_no_add_ref();
    if !object.is_null() {
        let mut count = tTJSVariant::new();
        let error = unsafe {
            (*object).prop_get(
                TJS_MEMBERMUSTEXIST as u32,
                tjs_w!("count"),
                ptr::null_mut(),
                &mut count,
                object,
            )
        };
        if !TJS_FAILED(error) {
            let count = count.as_integer();
            for index in 0..count {
                if index > 0 {
                    out.push_str(", ");
                }
                let mut item = tTJSVariant::new();
                let error = unsafe {
                    (*object).prop_get_by_num(
                        TJS_MEMBERMUSTEXIST as u32,
                        index as tjs_int,
                        &mut item,
                        object,
                    )
                };
                if TJS_FAILED(error) {
                    out.push_str("void");
                } else {
                    format_value(out, &mut item);
                }
            }
        }
    }
    out.push(']');
}

fn format_dict(out: &mut String, value: &tTJSVariant) {
    out.push_str("(dict){");
    let object = value.as_object_no_add_ref();
    if !object.is_null() {
        let keys = collect_dict_keys(object);
        for (index, key) in keys.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            out.push_str("(string)\"");
            escape_c(out, key);
            out.push_str("\": ");

            let key = ttstr::from(key.as_str());
            let mut item = tTJSVariant::new();
            let error = unsafe {
                (*object).prop_get(
                    TJS_MEMBERMUSTEXIST as u32,
                    key.c_str(),
                    ptr::null_mut(),
                    &mut item,
                    object,
                )
            };
            if TJS_FAILED(error) {
                out.push_str("void");
            } else {
                format_value(out, &mut item);
            }
        }
    }
    out.push('}');
}

struct DictKeyCollector(Rc<RefCell<Vec<String>>>);

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
        if numparams >= 2 && !param.is_null() {
            let key = unsafe { (*param).as_ref() };
            let flags = unsafe { (*param.add(1)).as_ref() };
            if let (Some(key), Some(flags)) = (key, flags) {
                if (flags.as_integer() & TJS_HIDDENMEMBER as i64) == 0 {
                    let string = key.as_string_no_add_ref();
                    if !string.is_null() {
                        let string = ttstr::from(string);
                        self.0.borrow_mut().push(string.to_string());
                    }
                }
            }
        }

        if let Some(result) = unsafe { result.as_mut() } {
            result.assign(true);
        }
        TJS_S_OK
    }
}

fn collect_dict_keys(object: *mut iTJSDispatch2) -> Vec<String> {
    let keys = Rc::new(RefCell::new(Vec::new()));
    let dispatch = tTJSDispatch::new(DictKeyCollector(keys.clone()));
    let mut callback = tTJSVariantClosure::new(dispatch, ptr::null_mut());
    let error =
        unsafe { (*object).enum_members(TJS_IGNOREPROP as u32, &mut callback, ptr::null_mut()) };
    unsafe { (*dispatch).release() };

    if TJS_FAILED(error) {
        Vec::new()
    } else {
        keys.borrow().clone()
    }
}

/// Escapes a string like TJS2's `tTJSString::EscapeC`.
fn escape_c(out: &mut String, value: &str) {
    let mut hex_escape = false;
    for character in value.chars() {
        match character {
            '\x07' => {
                out.push_str("\\a");
                hex_escape = false;
            }
            '\x08' => {
                out.push_str("\\b");
                hex_escape = false;
            }
            '\x0c' => {
                out.push_str("\\f");
                hex_escape = false;
            }
            '\n' => {
                out.push_str("\\n");
                hex_escape = false;
            }
            '\r' => {
                out.push_str("\\r");
                hex_escape = false;
            }
            '\t' => {
                out.push_str("\\t");
                hex_escape = false;
            }
            '\x0b' => {
                out.push_str("\\v");
                hex_escape = false;
            }
            '\\' => {
                out.push_str("\\\\");
                hex_escape = false;
            }
            '\'' => {
                out.push_str("\\'");
                hex_escape = false;
            }
            '"' => {
                out.push_str("\\\"");
                hex_escape = false;
            }
            character if (character as u32) < 0x20 => {
                let _ = write!(out, "\\x{:02x}", character as u32);
                hex_escape = true;
            }
            character => {
                if hex_escape && character.is_ascii_hexdigit() {
                    let _ = write!(out, "\\x{:02x}", character as u32);
                } else {
                    out.push(character);
                    hex_escape = false;
                }
            }
        }
    }
}

static mut GLOBAL_REF_COUNT_AT_INIT: tjs_int = 0;
generate_origin_static_block!(dm2, stack);

#[unsafe(export_name = "V2Link")]
unsafe extern "system" fn v2_link(exporter: *mut iTVPFunctionExporter) -> i32 {
    unsafe {
        TVPInitImportStub(exporter);
    }
    let dm2 = tTJSDispatch::new(DM2 {});
    let stack = tTJSDispatch::new(Stack {});
    register_var!(case = camel, dm2, stack);
    unsafe { GLOBAL_REF_COUNT_AT_INIT = TVPPluginGlobalRefCount };
    0
}

#[unsafe(export_name = "V2Unlink")]
unsafe extern "system" fn v2_unlink() -> i32 {
    if unsafe { TVPPluginGlobalRefCount > GLOBAL_REF_COUNT_AT_INIT } {
        log!("[debug]Can not unlink plugin");
        return TJS_E_FAIL;
    }
    unregister_var!(case = camel, dm2, stack);
    unsafe { TVPUninitImportStub() };
    0
}
