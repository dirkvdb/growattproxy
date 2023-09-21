fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(target_os = "windows")]
    return;

    #[cfg(not(target_os = "windows"))]
    {
        use cmake::Config;
        use std::env;

        if env::var_os("CARGO_FEATURE_SNIFFER").is_some() {
            let mut cfg = Config::new("libpcap");
            cfg.define("BUILD_SHARED_LIBS", "OFF")
                .define("DISABLE_NETMAP", "ON")
                .define("DISABLE_BLUETOOTH", "ON")
                .define("DISABLE_DBUS", "ON")
                .define("DISABLE_RDMA", "ON")
                .define("DISABLE_DAG", "ON")
                .define("DISABLE_SEPTEL", "ON")
                .define("DISABLE_SNF", "ON")
                .define("DISABLE_TC", "ON")
                .define("DISABLE_PACKET", "ON")
                .define("DISABLE_AIRPCAP", "ON")
                .define("DISABLE_DPDK ", "ON")
                .define("ENABLE_REMOTE", "OFF")
                .define("USE_STATIC_RT", "ON");

            if let Some(lex_path) = env::var_os("LEX_PATH") {
                cfg.define("LEX_EXECUTABLE", lex_path);
            }

            if let Some(yacc_path) = env::var_os("YACC_PATH") {
                cfg.define("YACC_EXECUTABLE", yacc_path);
            }

            #[allow(unused_mut)]
            let mut dst = cfg.build().join("lib");

            println!("cargo:rustc-link-search=native={}", dst.display());
            println!("cargo:rustc-link-lib=static=pcap");
        }
    }
}
