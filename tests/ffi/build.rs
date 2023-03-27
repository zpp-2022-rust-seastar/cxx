use cxx_build::CFG;

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

    if cfg!(trybuild) {
        return;
    }

    CFG.include_prefix = "tests/ffi";
    let sources = vec!["lib.rs", "module.rs"];
    let mut build = cxx_build::bridges(sources);
    build.file("tests.cc");
    build.flag_if_supported(cxxbridge_flags::STD);
    build.flag_if_supported("-std=c++20");
    build.includes(&seastar.include_paths);
    build.warnings_into_errors(cfg!(deny_warnings));
    build.flag_if_supported("-std=c++20");
    build.includes(&seastar.include_paths);
    if cfg!(not(target_env = "msvc")) {
        build.define("CXX_TEST_INSTANTIATIONS", None);
    }
    build.compile("cxx-test-suite");

    println!("cargo:rerun-if-changed=tests.cc");
    println!("cargo:rerun-if-changed=tests.h");
}
