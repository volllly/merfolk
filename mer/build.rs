fn main() {
    #[cfg(any(feature = "c_ffi", feature = "cpp_ffi"))]
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    #[cfg(feature = "c_ffi")]
    cbindgen::Builder::new()
      .with_crate(&crate_dir)
      .with_config(cbindgen::Config::from_file(".cbindgen.toml").expect("Unable to read config"))
      .generate()
      .expect("Unable to generate bindings")
      .write_to_file("ffi/c/mer.h");

    #[cfg(feature = "cpp_ffi")]
    cbindgen::Builder::new()
      .with_crate(&crate_dir)
      .with_config(cbindgen::Config::from_file(".cppbindgen.toml").expect("Unable to read config"))
      .generate()
      .expect("Unable to generate bindings")
      .write_to_file("ffi/cpp/mer.h");
}
