fn main() {
    // Register rustc cfg for switching between mount implementations.
    println!(
        "cargo::rustc-check-cfg=cfg(fuser_mount_impl, values(\"pure-rust\", \"libfuse2\", \"libfuse3\", \"macos-no-mount\"))"
    );

    let target_os =
        std::env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS should be set");

    if matches!(
        target_os.as_str(),
        "linux" | "freebsd" | "dragonfly" | "openbsd" | "netbsd"
    ) && cfg!(not(feature = "libfuse"))
    {
        println!("cargo::rustc-cfg=fuser_mount_impl=\"pure-rust\"");
    } else if target_os == "macos" {
        if cfg!(feature = "macos-no-mount") {
            println!("cargo::rustc-cfg=fuser_mount_impl=\"macos-no-mount\"");
        } else {
            // macFUSE 5.3+ makes libfuse2's fuse_mount_compat25() always fail on macOS, so
            // mount through libfuse3, which macFUSE also ships.
            // See https://github.com/cberner/fuser/issues/752
            configure_libfuse3().map_err(|e| eprintln!("{e}")).unwrap();
            println!("cargo::rustc-cfg=feature=\"macfuse-4-compat\"");
        }
    } else if cfg!(feature = "libfuse3") {
        configure_libfuse3().unwrap();
    } else if cfg!(feature = "libfuse2") {
        configure_libfuse2().unwrap();
    } else {
        // First try to link with libfuse3
        match configure_libfuse3() {
            Ok(()) => {}
            Err(e3) => {
                // Fallback to libfuse
                match configure_libfuse2() {
                    Ok(()) => {}
                    Err(e2) => {
                        panic!("Failed to configure libfuse3 or libfuse2: {e3}; {e2}");
                    }
                }
            }
        }
    }
}

fn configure_libfuse3() -> Result<(), pkg_config::Error> {
    pkg_config::Config::new()
        .atleast_version("3.0.0")
        .probe("fuse3")?;
    println!("cargo::rustc-cfg=fuser_mount_impl=\"libfuse3\"");
    Ok(())
}

fn configure_libfuse2() -> Result<(), pkg_config::Error> {
    pkg_config::Config::new()
        .atleast_version("2.6.0")
        .probe("fuse")?;
    println!("cargo::rustc-cfg=fuser_mount_impl=\"libfuse2\"");
    Ok(())
}
