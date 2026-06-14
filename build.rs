fn main() {
    // let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let bindings = bindgen::Builder::default()
        .clang_args(&["-x", "c++", "--std=c++17"])
        // .generate_inline_functions(true)
        // .wrap_static_fns(true)
        // .wrap_static_fns_path(&out_dir.join("a"))
        .header("src/tp_stub/tp_stub.h") // 包含宏的头文件
        .generate_comments(true)
        .allowlist_file(".*tp_stub\\.h")
        .blocklist_type("tTJSNativeInstance")
        .derive_default(true)
        .no_default("tTJS(String|Variant|VariantString)")
        .vtable_generation(true)
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
        .generate()
        .expect("bindgen failed");
    bindings
        .write_to_file(std::env::var("OUT_DIR").unwrap() + "/generated.rs")
        .expect("write failed");
    cc::Build::new()
        .cpp(true)
        .flag_if_supported("-std=c++17")
        .flag_if_supported("/std:c++17")
        .flag_if_supported("/utf-8")
        // .file(&cpp_path)
        .include("src/tp_stub")
        .file("src/tp_stub/tp_stub.cpp")
        .file("src/tp_stub/tp_stub_wrapping.cpp")
        .compile("tp_stub");
    println!("cargo:rerun-if-changed=src/tp_stub/tp_stub.h");
}
