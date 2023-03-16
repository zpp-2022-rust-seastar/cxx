use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let seastar = pkg_config::Config::new()
        .statik(true)
        .probe("seastar")
        .unwrap();

    // Workaround for the fact that seastar's pkg-config file
    // specifies the fmt dependency in a weird way. `pkg-config seastar --libs`
    // prints a path to a particular version of fmt (e.g. libfmt.so.8.1.1)
    // and the pkg_config crate can't parse this name as it expects to end
    // with just ".so". pkg_config crate prints a warning and does not
    // tell cargo to link with that library, so we have to do it manually.
    // Unfortunately, this workaround doesn't prevent a warning from being
    // printed by the previous command which prevents us from enforcing
    // a no-warning policy in the CI.
    // TODO: Remove this after seastar.pc or the pkg-config crate is fixed
    pkg_config::Config::new().statik(true).probe("fmt").unwrap();

    cc::Build::new()
        .file("src/cxx.cc")
        .cpp(true)
        .cpp_link_stdlib(None) // linked via link-cplusplus crate
        .flag_if_supported(cxxbridge_flags::STD)
        .flag_if_supported("-std=c++20")
        .includes(&seastar.include_paths)
        .warnings_into_errors(cfg!(deny_warnings))
        .compile("cxxbridge1");

    println!("cargo:rerun-if-changed=src/cxx.cc");
    println!("cargo:rerun-if-changed=include/cxx.h");
    println!("cargo:rustc-cfg=built_with_cargo");

    if let Some(manifest_dir) = env::var_os("CARGO_MANIFEST_DIR") {
        let cxx_h = Path::new(&manifest_dir).join("include").join("cxx.h");
        println!("cargo:HEADER={}", cxx_h.to_string_lossy());
    }

    if let Some(rustc) = rustc_version() {
        if rustc.minor < 60 {
            println!("cargo:warning=The cxx crate requires a rustc version 1.60.0 or newer.");
            println!(
                "cargo:warning=You appear to be building with: {}",
                rustc.version,
            );
        }
    }
}

struct RustVersion {
    version: String,
    minor: u32,
}

fn rustc_version() -> Option<RustVersion> {
    let rustc = env::var_os("RUSTC")?;
    let output = Command::new(rustc).arg("--version").output().ok()?;
    let version = String::from_utf8(output.stdout).ok()?;
    let mut pieces = version.split('.');
    if pieces.next() != Some("rustc 1") {
        return None;
    }
    let minor = pieces.next()?.parse().ok()?;
    Some(RustVersion { version, minor })
}
