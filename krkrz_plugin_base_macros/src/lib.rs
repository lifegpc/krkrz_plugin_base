use convert_case::{Case, Casing, ccase};
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Error, Ident, LitStr,
    parse::{Parse, ParseStream, discouraged::Speculative},
    parse_macro_input,
    token::{Comma, Eq},
};

fn parse_case(case: &str) -> Case<'static> {
    let case = ccase!(snake, case);
    match case.as_str() {
        "snake" => Case::Snake,
        "constant" => Case::Constant,
        "upper_snake" => Case::UpperSnake,
        "ada" => Case::Ada,
        "kebab" => Case::Kebab,
        "cobol" => Case::Cobol,
        "upper_kebab" => Case::UpperKebab,
        "train" => Case::Train,
        "flat" => Case::Flat,
        "upper_flat" => Case::UpperFlat,
        "pascal" => Case::Pascal,
        "upper_camel" => Case::UpperCamel,
        "camel" => Case::Camel,
        "lower" => Case::Lower,
        "upper" => Case::Upper,
        "title" => Case::Title,
        "sentence" => Case::Sentence,
        _ => panic!("Unsupported case: {}", case),
    }
}

struct OriginStaticBlockVars(Vec<Ident>);

impl Parse for OriginStaticBlockVars {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut data = Vec::new();
        loop {
            if input.cursor().eof() {
                break;
            }
            data.push(input.parse()?);
            if input.cursor().eof() {
                break;
            }
            input.parse::<Comma>()?;
        }
        Ok(Self(data))
    }
}

struct RegisterVar {
    ident: Ident,
    name: String,
}

struct RegisterVars(Vec<RegisterVar>);

impl Parse for RegisterVars {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut case = Case::Pascal;
        let mut data = Vec::new();
        loop {
            if input.cursor().eof() {
                break;
            }
            let key: Ident = input.parse()?;
            let fork = input.fork();
            if let Ok(_) = fork.parse::<Eq>() {
                let ckey = key.to_string().to_lowercase();
                if ckey == "case" {
                    let value: Ident = fork.parse()?;
                    input.advance_to(&fork);
                    case = parse_case(&value.to_string());
                } else {
                    return Err(Error::new(key.span(), "Unknown key."));
                }
            } else {
                let name = key.to_string().to_case(case);
                data.push(RegisterVar { ident: key, name });
            }
            if input.cursor().eof() {
                break;
            }
            input.parse::<Comma>()?;
        }
        Ok(Self(data))
    }
}

#[proc_macro]
/// Generate a list static mut global variable to store original TJS variant
pub fn generate_origin_static_block(input: TokenStream) -> TokenStream {
    let vars = parse_macro_input!(input as OriginStaticBlockVars).0;
    let streams: Vec<_> = vars.iter().map(|s| {
        let name = format!("ORIGIN_{}", ccase!(constant, s.to_string()));
        let name = Ident::new(&name, s.span());
        quote!(static mut #name: *mut krkrz_plugin_base::tp_stub::iTJSDispatch2 = std::ptr::null_mut();)
    }).collect();
    let stream = quote! {
        #(#streams)*
    };
    stream.into()
}

#[proc_macro]
/// Regsister variables to TVP global scope.
///
/// Accept a list of variables.
/// Use case = xxx to specify case of variable name. Default case is [`PascalCase`](Case::Pascal). See [`Case`] for more information.
///
/// ```ignore
/// use krkrz_plugin_base::tp_stub::*;
/// static mut GLOBAL_REF_COUNT_AT_INIT: tjs_int = 0;
/// generate_origin_static_block!(func_a, class_a);
/// #[unsafe(export_name = "V2Link")]
/// unsafe extern "system" fn v2_link(exporter: *mut iTVPFunctionExporter) -> i32 {
///     unsafe { TVPInitImportStub(exporter) };
///     let func_a = create_func_a();
///     let class_a = create_class_a();
///     register_var!(case = camel, func_a, case = pascal, class_a); // Register func_a as funcA, class_a as ClassA
///     unsafe { GLOBAL_REF_COUNT_AT_INIT = TVPPluginGlobalRefCount };
///     0
/// }
/// #[unsafe(export_name = "V2Unlink")]
/// {
///     if unsafe { TVPPluginGlobalRefCount > GLOBAL_REF_COUNT_AT_INIT } {
///         return TJS_E_FAIL;
///     }
///     unregister_var!(case = camel, func_a, case = pascal, class_a);
///     unsafe { TVPUninitImportStub() };
///     0
/// }
/// ```
pub fn register_var(input: TokenStream) -> TokenStream {
    let vars = parse_macro_input!(input as RegisterVars).0;
    let ok_streams: Vec<_> = vars
        .iter()
        .map(|s| {
            let name = LitStr::new(&s.name, s.ident.span());
            let tname = format!("ORIGIN_{}", ccase!(constant, s.ident.to_string()));
            let origin = Ident::new(&tname, s.ident.span());
            let ident = s.ident.clone();
            quote! {
                let name = ttstr::from(#name);
                let n = name.c_str();
                let mut val = tTJSVariant::new();
                if TJS_SUCCEEDED(unsafe {
                    (*global).prop_get(0, n, std::ptr::null_mut(), &mut val, global)
                }) {
                    unsafe {
                        #origin = val.as_object();
                    }
                    val.clear();
                }
                let val = tTJSVariant::from(#ident);
                unsafe { (*#ident).release() };
                unsafe {
                    (*global).prop_set(
                        TJS_MEMBERENSURE as u32,
                        n,
                        std::ptr::null_mut(),
                        &val,
                        global,
                    )
                };
            }
        })
        .collect();
    let err_streams: Vec<_> = vars
        .iter()
        .map(|s| {
            let ident = s.ident.clone();
            quote!(unsafe { (*#ident).release() };)
        })
        .collect();
    let stream = quote! {{
        use krkrz_plugin_base::tp_stub::*;
        let global = unsafe { TVPGetScriptDispatch() };
        if !global.is_null() {
            #(#ok_streams)*
            unsafe { (*global).release() };
        } else {
            #(#err_streams)*
        }
    }};
    stream.into()
}

#[proc_macro]
/// Unregsister variables to TVP global scope.
///
/// Accept same arguments for [register_var].
/// See [register_var] for more information.
pub fn unregister_var(input: TokenStream) -> TokenStream {
    let vars = parse_macro_input!(input as RegisterVars).0;
    let streams: Vec<_> = vars
        .iter()
        .map(|s| {
            let name = LitStr::new(&s.name, s.ident.span());
            let tname = format!("ORIGIN_{}", ccase!(constant, s.ident.to_string()));
            let origin = Ident::new(&tname, s.ident.span());
            quote! {
                let name = ttstr::from(#name);
                let n = name.c_str();
                unsafe { (*global).delete_member(0, n, std::ptr::null_mut(), global) };
                unsafe {
                    if !#origin.is_null() {
                        let val: tTJSVariant = tTJSVariant::from(#origin);
                        (*#origin).release();
                        #origin = std::ptr::null_mut();
                        (*global).prop_set(
                            TJS_MEMBERENSURE as u32,
                            n,
                            std::ptr::null_mut(),
                            &val,
                            global,
                        );
                    }
                }
            }
        })
        .collect();
    let stream = quote! {{
        use krkrz_plugin_base::tp_stub::*;
        let global = unsafe { TVPGetScriptDispatch() };
        if !global.is_null() {
            #(#streams)*
            unsafe { (*global).release() };
        }
    }};
    stream.into()
}
