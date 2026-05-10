fn main() {
    pyo3_build_config::use_pyo3_cfgs();
    pyo3_build_config::add_extension_module_link_args();
    let python_config = pyo3_build_config::get();

    println!("cargo::rustc-check-cfg=cfg(unicode_state)");
    if std::env::var("CARGO_CFG_TARGET_ENDIAN").unwrap() == "little"
        && !(std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows"
            && python_config.is_free_threaded())
    {
        println!("cargo::rustc-cfg=unicode_state");
    }
}
