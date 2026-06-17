use convert_case::{Case, Casing, ccase};
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Attribute, Error, FnArg, Ident, ImplItem, ImplItemFn, ItemImpl, LitStr,
    parse::{Parse, ParseStream, discouraged::Speculative},
    parse_macro_input,
    spanned::Spanned,
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

#[derive(Default)]
struct GlobalTjsAttributes {
    class_name: Option<LitStr>,
}

impl GlobalTjsAttributes {
    fn parse(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut data = Self::default();
        for attr in attrs {
            if attr.path().is_ident("tjs") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("class_name") {
                        let value = meta.value()?;
                        let s: LitStr = value.parse()?;
                        data.class_name = Some(s);
                        Ok(())
                    } else {
                        Err(meta.error("unsupported tjs attribute for impl block"))
                    }
                })?;
            }
        }
        Ok(data)
    }
}

#[derive(Default)]
struct FnTjsAttributes {
    constructor: bool,
    skip: bool,
}

impl FnTjsAttributes {
    fn parse(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut data = Self::default();
        for attr in attrs {
            if attr.path().is_ident("tjs") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("constructor") {
                        data.constructor = true;
                        Ok(())
                    } else if meta.path.is_ident("skip") {
                        data.skip = true;
                        Ok(())
                    } else {
                        Err(meta.error("unsupported tjs attribute for impl block"))
                    }
                })?;
            }
        }
        Ok(data)
    }
}

fn get_class_name(ty: &syn::Type) -> syn::Result<String> {
    match ty {
        syn::Type::Path(p) => Ok(p
            .path
            .segments
            .last()
            .ok_or_else(|| syn::Error::new(ty.span(), "Failed to get class name from self type."))?
            .ident
            .to_string()),
        _ => Err(syn::Error::new(
            ty.span(),
            "Failed to get class name from self type.",
        )),
    }
}

fn find_constructor(items: &[ImplItem]) -> Option<&ImplItemFn> {
    let mut new_class = None;
    let mut first_class = None;
    for item in items {
        match item {
            ImplItem::Fn(f) => {
                let attr = FnTjsAttributes::parse(&f.attrs).unwrap();
                if attr.skip {
                    continue;
                }
                if attr.constructor {
                    return Some(f);
                }
                if f.sig
                    .inputs
                    .first()
                    .is_some_and(|f| matches!(f, FnArg::Receiver(_)))
                {
                    continue;
                }
                if f.sig.ident == "new" {
                    new_class = Some(f);
                }
                first_class.get_or_insert(f);
            }
            _ => {}
        }
    }
    new_class.or(first_class)
}

fn peel_type(ty: &syn::Type) -> &syn::Type {
    match ty {
        syn::Type::Group(group) => peel_type(&group.elem),
        syn::Type::Paren(paren) => peel_type(&paren.elem),
        other => other,
    }
}

fn extract_option_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    use syn::{GenericArgument, PathArguments, Type, TypePath};

    let inner = peel_type(ty);
    if let Type::Path(TypePath { qself: None, path }) = inner {
        if let Some(segment) = path.segments.last() {
            if segment.ident == "Option" {
                if let PathArguments::AngleBracketed(ref args) = segment.arguments {
                    if args.args.len() == 1 {
                        if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                            return Some(inner_ty);
                        }
                    }
                }
            }
        }
    }
    None
}

fn is_option_type(ty: &syn::Type) -> bool {
    extract_option_inner_type(ty).is_some()
}

fn extract_result_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    use syn::{GenericArgument, PathArguments, Type, TypePath};

    let inner = peel_type(ty);
    if let Type::Path(TypePath { qself: None, path }) = inner {
        if let Some(segment) = path.segments.last() {
            if segment.ident == "Result" {
                if let PathArguments::AngleBracketed(ref args) = segment.arguments {
                    if args.args.len() == 1 {
                        if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                            return Some(inner_ty);
                        }
                    }
                }
            }
        }
    }
    None
}

fn is_result_type(ty: &syn::Type) -> bool {
    extract_result_inner_type(ty).is_some()
}

fn gen_constructor(f: &ImplItemFn, self_ty: &syn::Type) -> proc_macro2::TokenStream {
    use syn::{Pat, ReturnType};
    let mut min_args = 0;
    for (i, arg) in f.sig.inputs.iter().enumerate() {
        let t = match arg {
            FnArg::Receiver(_) => panic!("constructor can not have a self argument."),
            FnArg::Typed(t) => t,
        };
        if !is_option_type(&t.ty) {
            min_args = i + 1;
        }
    }
    let streams: Vec<_> = f
        .sig
        .inputs
        .iter()
        .enumerate()
        .map(|(i, arg)| {
            let t = match arg {
                FnArg::Receiver(_) => panic!("constructor can not have a self argument."),
                FnArg::Typed(t) => t,
            };
            let is_option = is_option_type(&t.ty);
            let name = match t.pat.as_ref() {
                Pat::Ident(idt) => idt.ident.clone(),
                _ => panic!("Unsupported name decalre of argument."),
            };
            if is_option {
                quote! {
                    let #name = if numparams <= (#i as tjs_int) {
                        None
                    } else {
                        let p = unsafe { *param.add(#i) };
                        if p.is_null() {
                            None
                        } else {
                            let p = unsafe { &mut *p };
                            if p.is_void() {
                                None
                            } else {
                                match TjsParam::to_param(p) {
                                    Ok(t) => Some(t),
                                    Err(e) => {
                                        log!("Failed to convert param to specify type: {}", e);
                                        return Err(TJS_E_INVALIDPARAM);
                                    }
                                }
                            }
                        }
                    };
                }
            } else {
                quote! {
                    let p = unsafe { *param.add(#i) };
                    let #name = if p.is_null() {
                        throw_null_access();
                    } else {
                        let p = unsafe { &mut *p };
                        match TjsParam::to_param(p) {
                            Ok(t) => t,
                            Err(e) => {
                                log!("Failed to convert param to specify type: {}", e);
                                return Err(TJS_E_INVALIDPARAM);
                            }
                        }
                    };
                }
            }
        })
        .collect();
    let args: Vec<_> = f
        .sig
        .inputs
        .iter()
        .map(|arg| {
            let t = match arg {
                FnArg::Receiver(_) => panic!("constructor can not have a self argument."),
                FnArg::Typed(t) => t,
            };
            let name = match t.pat.as_ref() {
                Pat::Ident(idt) => idt.ident.clone(),
                _ => panic!("Unsupported name decalre of argument."),
            };
            quote!(#name,)
        })
        .collect();
    let name = f.sig.ident.clone();
    let output_type = match &f.sig.output {
        ReturnType::Default => panic!("constructor mut return something."),
        ReturnType::Type(_, ty) => ty,
    };
    let result = if is_result_type(&output_type) {
        quote!(
            let re = match re {
                Ok(re) => re,
                Err(e) => {
                    log!("Failed to construct object: {}", e);
                    return Err(TJS_E_FAIL);
                }
            };
        )
    } else {
        quote!()
    };
    let min_args = min_args;
    quote! {
        if numparams < (#min_args as tjs_int) {
            return Err(TJS_E_BADPARAMCOUNT);
        }
        if numparams > 0 && param.is_null() {
            throw_null_access();
        }
        #(#streams)*
        let re = #self_ty::#name(#(#args)*);
        #result
        Ok(Box::new(re))
    }
}

#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Tjs2Class(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemImpl);
    let attrs = GlobalTjsAttributes::parse(&input.attrs).unwrap();
    let class_name = attrs.class_name.clone().unwrap_or_else(|| {
        LitStr::new(
            &get_class_name(&input.self_ty).unwrap(),
            input.self_ty.span(),
        )
    });
    let self_ty = input.self_ty.clone();
    let constructor = find_constructor(&input.items).expect("No constructor function was found.");
    let main_constructor = gen_constructor(constructor, &self_ty);
    let invalidate = input
        .items
        .iter()
        .filter_map(|s| match s {
            ImplItem::Fn(s) => {
                if s.sig.ident == "invalidate" {
                    let attr = FnTjsAttributes::parse(&s.attrs).unwrap();
                    if attr.skip {
                        return None;
                    }
                    Some(quote! { s.invalidate() })
                } else {
                    None
                }
            }
            _ => None,
        })
        .next()
        .unwrap_or_default();
    let classid_name = Ident::new(
        &format!("CID_{}", ccase!(constant, &class_name.value())),
        class_name.span(),
    );
    let stream = quote! {
        #input
        static mut #classid_name: i32 = 0;
        impl krkrz_plugin_base::Tjs2Class for #self_ty {
            fn create_native_class() -> (i32, *mut krkrz_plugin_base::tp_stub::iTJSDispatch2) {
                use krkrz_plugin_base::{tp_stub::*, *};
                #[repr(C)]
                struct NativeInstatce {
                    base: iTJSNativeInstance,
                    inner: Option<Box<#self_ty>>,
                }
                impl NativeInstatce {
                    unsafe extern "C" fn new() -> *mut iTJSNativeInstance {
                        static VTABLE: iTJSNativeInstance__bindgen_vtable = iTJSNativeInstance__bindgen_vtable {
                            iTJSNativeInstance_Construct: NativeInstatce::construct,
                            iTJSNativeInstance_Invalidate: NativeInstatce::invalidate,
                            iTJSNativeInstance_Destruct: NativeInstatce::destruct,
                        };
                        let boxed = Box::new(Self {
                            base: iTJSNativeInstance { vtable_: &VTABLE },
                            inner: None,
                        });
                        Box::into_raw(boxed) as *mut iTJSNativeInstance
                    }
                    fn constructor(numparams: tjs_int, param: *mut *mut tTJSVariant, tjs_obj: *mut iTJSDispatch2) -> Result<Box<#self_ty>, i32> {
                        #main_constructor
                    }
                    unsafe extern "C" fn construct(
                        this: *mut iTJSNativeInstance,
                        numparams: tjs_int,
                        param: *mut *mut tTJSVariant,
                        tjs_obj: *mut iTJSDispatch2,
                    ) -> tjs_error {
                        let self_ = unsafe { &mut *(this as *mut NativeInstatce) };
                        let hr = match Self::constructor(numparams, param, tjs_obj) {
                            Ok(data) => {
                                self_.inner = Some(data);
                                0
                            }
                            Err(err) => {
                                err
                            }
                        };
                        if TJS_SUCCEEDED(hr) {
                            unsafe { TVPPluginGlobalRefCount += 1 };
                        } else {
                            // Workround fix leak when error returned from construct
                            let _boxed = unsafe { Box::from_raw(this as *mut tTJSNativeInstance) };
                        }
                        hr
                    }
                    unsafe extern "C" fn invalidate(this: *mut iTJSNativeInstance) {
                        let self_ = unsafe { &mut *(this as *mut NativeInstatce) };
                        if let Some(s) = self_.inner.as_mut() {
                            #invalidate
                        }
                    }
                    unsafe extern "C" fn destruct(this: *mut iTJSNativeInstance) {
                        let _box = unsafe { Box::from_raw(this as *mut NativeInstatce) };
                        unsafe { TVPPluginGlobalRefCount -= 1 };
                    }
                }
                let classname = ttstr::from(#class_name);
                let classobj =
                    unsafe { TJSCreateNativeClassForPlugin(&classname as *const _, Some(NativeInstatce::new)) }
                        as *mut tTJSNativeClass;
                let name = classname.c_str();
                let classid = unsafe { TJSRegisterNativeClass(name) };
                unsafe { #classid_name = classid };
                unsafe { TJSNativeClassSetClassID(classobj, classid); }
                unsafe extern "C" fn ncm_construct(
                    _result: *mut tTJSVariant,
                    numparams: tjs_int,
                    param: *mut *mut tTJSVariant,
                    tjs_obj: *mut iTJSDispatch2,
                ) -> tjs_error {
                    let mut _this: *mut iTJSNativeInstance = std::ptr::null_mut();
                    let hr =
                        unsafe { (*tjs_obj).native_instance_support(0x00000002, #classid_name, &mut _this) };
                    if TJS_FAILED(hr) {
                        return TJS_E_NATIVECLASSCRASH;
                    }
                    if _this.is_null() {
                        return TJS_E_NATIVECLASSCRASH;
                    }
                    unsafe { (*_this).construct(numparams, param, tjs_obj) }
                }
                unsafe extern "C" fn ncm_finalize(
                    _result: *mut tTJSVariant,
                    _numparams: tjs_int,
                    _param: *mut *mut tTJSVariant,
                    _tjs_obj: *mut iTJSDispatch2,
                ) -> tjs_error {
                    TJS_S_OK
                }
                let fnname = ttstr::from("finalize");
                let fname = fnname.c_str();
                unsafe {
                    TJSNativeClassRegisterNCM(
                        classobj,
                        fname,
                        TJSCreateNativeClassMethod(Some(ncm_finalize)) as *mut _,
                        name,
                        tTJSNativeInstanceType_nitMethod,
                        0,
                    );
                    TJSNativeClassRegisterNCM(
                        classobj,
                        name,
                        TJSCreateNativeClassConstructor(Some(ncm_construct)) as *mut _,
                        name,
                        tTJSNativeInstanceType_nitClass,
                        0,
                    );
                }
                (classid, classobj as *mut iTJSDispatch2)
            }
        }
    };
    stream.into()
}

#[proc_macro_attribute]
/// This macro attribute do nothing. Check [Tjs2Class] instead.
pub fn tjs(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    input
}
