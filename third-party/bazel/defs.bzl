###############################################################################
# @generated
# DO NOT MODIFY: This file is auto-generated by a crate_universe tool. To
# regenerate this file, run the following:
#
#     bazel run @//third-party:vendor
###############################################################################
"""
# `crates_repository` API

- [aliases](#aliases)
- [crate_deps](#crate_deps)
- [all_crate_deps](#all_crate_deps)
- [crate_repositories](#crate_repositories)

"""

load("@bazel_skylib//lib:selects.bzl", "selects")
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
load("@bazel_tools//tools/build_defs/repo:utils.bzl", "maybe")

###############################################################################
# MACROS API
###############################################################################

# An identifier that represent common dependencies (unconditional).
_COMMON_CONDITION = ""

def _flatten_dependency_maps(all_dependency_maps):
    """Flatten a list of dependency maps into one dictionary.

    Dependency maps have the following structure:

    ```python
    DEPENDENCIES_MAP = {
        # The first key in the map is a Bazel package
        # name of the workspace this file is defined in.
        "workspace_member_package": {

            # Not all dependnecies are supported for all platforms.
            # the condition key is the condition required to be true
            # on the host platform.
            "condition": {

                # An alias to a crate target.     # The label of the crate target the
                # Aliases are only crate names.   # package name refers to.
                "package_name":                   "@full//:label",
            }
        }
    }
    ```

    Args:
        all_dependency_maps (list): A list of dicts as described above

    Returns:
        dict: A dictionary as described above
    """
    dependencies = {}

    for workspace_deps_map in all_dependency_maps:
        for pkg_name, conditional_deps_map in workspace_deps_map.items():
            if pkg_name not in dependencies:
                non_frozen_map = dict()
                for key, values in conditional_deps_map.items():
                    non_frozen_map.update({key: dict(values.items())})
                dependencies.setdefault(pkg_name, non_frozen_map)
                continue

            for condition, deps_map in conditional_deps_map.items():
                # If the condition has not been recorded, do so and continue
                if condition not in dependencies[pkg_name]:
                    dependencies[pkg_name].setdefault(condition, dict(deps_map.items()))
                    continue

                # Alert on any miss-matched dependencies
                inconsistent_entries = []
                for crate_name, crate_label in deps_map.items():
                    existing = dependencies[pkg_name][condition].get(crate_name)
                    if existing and existing != crate_label:
                        inconsistent_entries.append((crate_name, existing, crate_label))
                    dependencies[pkg_name][condition].update({crate_name: crate_label})

    return dependencies

def crate_deps(deps, package_name = None):
    """Finds the fully qualified label of the requested crates for the package where this macro is called.

    Args:
        deps (list): The desired list of crate targets.
        package_name (str, optional): The package name of the set of dependencies to look up.
            Defaults to `native.package_name()`.

    Returns:
        list: A list of labels to generated rust targets (str)
    """

    if not deps:
        return []

    if package_name == None:
        package_name = native.package_name()

    # Join both sets of dependencies
    dependencies = _flatten_dependency_maps([
        _NORMAL_DEPENDENCIES,
        _NORMAL_DEV_DEPENDENCIES,
        _PROC_MACRO_DEPENDENCIES,
        _PROC_MACRO_DEV_DEPENDENCIES,
        _BUILD_DEPENDENCIES,
        _BUILD_PROC_MACRO_DEPENDENCIES,
    ]).pop(package_name, {})

    # Combine all conditional packages so we can easily index over a flat list
    # TODO: Perhaps this should actually return select statements and maintain
    # the conditionals of the dependencies
    flat_deps = {}
    for deps_set in dependencies.values():
        for crate_name, crate_label in deps_set.items():
            flat_deps.update({crate_name: crate_label})

    missing_crates = []
    crate_targets = []
    for crate_target in deps:
        if crate_target not in flat_deps:
            missing_crates.append(crate_target)
        else:
            crate_targets.append(flat_deps[crate_target])

    if missing_crates:
        fail("Could not find crates `{}` among dependencies of `{}`. Available dependencies were `{}`".format(
            missing_crates,
            package_name,
            dependencies,
        ))

    return crate_targets

def all_crate_deps(
        normal = False,
        normal_dev = False,
        proc_macro = False,
        proc_macro_dev = False,
        build = False,
        build_proc_macro = False,
        package_name = None):
    """Finds the fully qualified label of all requested direct crate dependencies \
    for the package where this macro is called.

    If no parameters are set, all normal dependencies are returned. Setting any one flag will
    otherwise impact the contents of the returned list.

    Args:
        normal (bool, optional): If True, normal dependencies are included in the
            output list.
        normal_dev (bool, optional): If True, normal dev dependencies will be
            included in the output list..
        proc_macro (bool, optional): If True, proc_macro dependencies are included
            in the output list.
        proc_macro_dev (bool, optional): If True, dev proc_macro dependencies are
            included in the output list.
        build (bool, optional): If True, build dependencies are included
            in the output list.
        build_proc_macro (bool, optional): If True, build proc_macro dependencies are
            included in the output list.
        package_name (str, optional): The package name of the set of dependencies to look up.
            Defaults to `native.package_name()` when unset.

    Returns:
        list: A list of labels to generated rust targets (str)
    """

    if package_name == None:
        package_name = native.package_name()

    # Determine the relevant maps to use
    all_dependency_maps = []
    if normal:
        all_dependency_maps.append(_NORMAL_DEPENDENCIES)
    if normal_dev:
        all_dependency_maps.append(_NORMAL_DEV_DEPENDENCIES)
    if proc_macro:
        all_dependency_maps.append(_PROC_MACRO_DEPENDENCIES)
    if proc_macro_dev:
        all_dependency_maps.append(_PROC_MACRO_DEV_DEPENDENCIES)
    if build:
        all_dependency_maps.append(_BUILD_DEPENDENCIES)
    if build_proc_macro:
        all_dependency_maps.append(_BUILD_PROC_MACRO_DEPENDENCIES)

    # Default to always using normal dependencies
    if not all_dependency_maps:
        all_dependency_maps.append(_NORMAL_DEPENDENCIES)

    dependencies = _flatten_dependency_maps(all_dependency_maps).pop(package_name, None)

    if not dependencies:
        if dependencies == None:
            fail("Tried to get all_crate_deps for package " + package_name + " but that package had no Cargo.toml file")
        else:
            return []

    crate_deps = list(dependencies.pop(_COMMON_CONDITION, {}).values())
    for condition, deps in dependencies.items():
        crate_deps += selects.with_or({_CONDITIONS[condition]: deps.values()})

    return crate_deps

def aliases(
        normal = False,
        normal_dev = False,
        proc_macro = False,
        proc_macro_dev = False,
        build = False,
        build_proc_macro = False,
        package_name = None):
    """Produces a map of Crate alias names to their original label

    If no dependency kinds are specified, `normal` and `proc_macro` are used by default.
    Setting any one flag will otherwise determine the contents of the returned dict.

    Args:
        normal (bool, optional): If True, normal dependencies are included in the
            output list.
        normal_dev (bool, optional): If True, normal dev dependencies will be
            included in the output list..
        proc_macro (bool, optional): If True, proc_macro dependencies are included
            in the output list.
        proc_macro_dev (bool, optional): If True, dev proc_macro dependencies are
            included in the output list.
        build (bool, optional): If True, build dependencies are included
            in the output list.
        build_proc_macro (bool, optional): If True, build proc_macro dependencies are
            included in the output list.
        package_name (str, optional): The package name of the set of dependencies to look up.
            Defaults to `native.package_name()` when unset.

    Returns:
        dict: The aliases of all associated packages
    """
    if package_name == None:
        package_name = native.package_name()

    # Determine the relevant maps to use
    all_aliases_maps = []
    if normal:
        all_aliases_maps.append(_NORMAL_ALIASES)
    if normal_dev:
        all_aliases_maps.append(_NORMAL_DEV_ALIASES)
    if proc_macro:
        all_aliases_maps.append(_PROC_MACRO_ALIASES)
    if proc_macro_dev:
        all_aliases_maps.append(_PROC_MACRO_DEV_ALIASES)
    if build:
        all_aliases_maps.append(_BUILD_ALIASES)
    if build_proc_macro:
        all_aliases_maps.append(_BUILD_PROC_MACRO_ALIASES)

    # Default to always using normal aliases
    if not all_aliases_maps:
        all_aliases_maps.append(_NORMAL_ALIASES)
        all_aliases_maps.append(_PROC_MACRO_ALIASES)

    aliases = _flatten_dependency_maps(all_aliases_maps).pop(package_name, None)

    if not aliases:
        return dict()

    common_items = aliases.pop(_COMMON_CONDITION, {}).items()

    # If there are only common items in the dictionary, immediately return them
    if not len(aliases.keys()) == 1:
        return dict(common_items)

    # Build a single select statement where each conditional has accounted for the
    # common set of aliases.
    crate_aliases = {"//conditions:default": common_items}
    for condition, deps in aliases.items():
        condition_triples = _CONDITIONS[condition]
        if condition_triples in crate_aliases:
            crate_aliases[condition_triples].update(deps)
        else:
            crate_aliases.update({_CONDITIONS[condition]: dict(deps.items() + common_items)})

    return selects.with_or(crate_aliases)

###############################################################################
# WORKSPACE MEMBER DEPS AND ALIASES
###############################################################################

_NORMAL_DEPENDENCIES = {
    "third-party": {
        _COMMON_CONDITION: {
            "cc": "@vendor__cc-1.0.79//:cc",
            "clap": "@vendor__clap-4.1.8//:clap",
            "codespan-reporting": "@vendor__codespan-reporting-0.11.1//:codespan_reporting",
            "once_cell": "@vendor__once_cell-1.17.1//:once_cell",
            "proc-macro2": "@vendor__proc-macro2-1.0.51//:proc_macro2",
            "quote": "@vendor__quote-1.0.23//:quote",
            "scratch": "@vendor__scratch-1.0.5//:scratch",
            "syn": "@vendor__syn-1.0.109//:syn",
        },
    },
}

_NORMAL_ALIASES = {
    "third-party": {
        _COMMON_CONDITION: {
        },
    },
}

_NORMAL_DEV_DEPENDENCIES = {
    "third-party": {
    },
}

_NORMAL_DEV_ALIASES = {
    "third-party": {
    },
}

_PROC_MACRO_DEPENDENCIES = {
    "third-party": {
    },
}

_PROC_MACRO_ALIASES = {
    "third-party": {
    },
}

_PROC_MACRO_DEV_DEPENDENCIES = {
    "third-party": {
    },
}

_PROC_MACRO_DEV_ALIASES = {
    "third-party": {
    },
}

_BUILD_DEPENDENCIES = {
    "third-party": {
    },
}

_BUILD_ALIASES = {
    "third-party": {
    },
}

_BUILD_PROC_MACRO_DEPENDENCIES = {
    "third-party": {
    },
}

_BUILD_PROC_MACRO_ALIASES = {
    "third-party": {
    },
}

_CONDITIONS = {
    "cfg(windows)": ["aarch64-pc-windows-msvc", "i686-pc-windows-msvc", "x86_64-pc-windows-msvc"],
    "i686-pc-windows-gnu": [],
    "x86_64-pc-windows-gnu": [],
}

###############################################################################

def crate_repositories():
    """A macro for defining repositories for all generated crates"""
    maybe(
        http_archive,
        name = "vendor__bitflags-1.3.2",
        sha256 = "bef38d45163c2f1dde094a7dfd33ccf595c92905c8f8f4fdc18d06fb1037718a",
        type = "tar.gz",
        urls = ["https://crates.io/api/v1/crates/bitflags/1.3.2/download"],
        strip_prefix = "bitflags-1.3.2",
        build_file = Label("@cxx.rs//third-party/bazel:BUILD.bitflags-1.3.2.bazel"),
    )

    maybe(
        http_archive,
        name = "vendor__cc-1.0.79",
        sha256 = "50d30906286121d95be3d479533b458f87493b30a4b5f79a607db8f5d11aa91f",
        type = "tar.gz",
        urls = ["https://crates.io/api/v1/crates/cc/1.0.79/download"],
        strip_prefix = "cc-1.0.79",
        build_file = Label("@cxx.rs//third-party/bazel:BUILD.cc-1.0.79.bazel"),
    )

    maybe(
        http_archive,
        name = "vendor__clap-4.1.8",
        sha256 = "c3d7ae14b20b94cb02149ed21a86c423859cbe18dc7ed69845cace50e52b40a5",
        type = "tar.gz",
        urls = ["https://crates.io/api/v1/crates/clap/4.1.8/download"],
        strip_prefix = "clap-4.1.8",
        build_file = Label("@cxx.rs//third-party/bazel:BUILD.clap-4.1.8.bazel"),
    )

    maybe(
        http_archive,
        name = "vendor__clap_lex-0.3.2",
        sha256 = "350b9cf31731f9957399229e9b2adc51eeabdfbe9d71d9a0552275fd12710d09",
        type = "tar.gz",
        urls = ["https://crates.io/api/v1/crates/clap_lex/0.3.2/download"],
        strip_prefix = "clap_lex-0.3.2",
        build_file = Label("@cxx.rs//third-party/bazel:BUILD.clap_lex-0.3.2.bazel"),
    )

    maybe(
        http_archive,
        name = "vendor__codespan-reporting-0.11.1",
        sha256 = "3538270d33cc669650c4b093848450d380def10c331d38c768e34cac80576e6e",
        type = "tar.gz",
        urls = ["https://crates.io/api/v1/crates/codespan-reporting/0.11.1/download"],
        strip_prefix = "codespan-reporting-0.11.1",
        build_file = Label("@cxx.rs//third-party/bazel:BUILD.codespan-reporting-0.11.1.bazel"),
    )

    maybe(
        http_archive,
        name = "vendor__once_cell-1.17.1",
        sha256 = "b7e5500299e16ebb147ae15a00a942af264cf3688f47923b8fc2cd5858f23ad3",
        type = "tar.gz",
        urls = ["https://crates.io/api/v1/crates/once_cell/1.17.1/download"],
        strip_prefix = "once_cell-1.17.1",
        build_file = Label("@cxx.rs//third-party/bazel:BUILD.once_cell-1.17.1.bazel"),
    )

    maybe(
        http_archive,
        name = "vendor__os_str_bytes-6.4.1",
        sha256 = "9b7820b9daea5457c9f21c69448905d723fbd21136ccf521748f23fd49e723ee",
        type = "tar.gz",
        urls = ["https://crates.io/api/v1/crates/os_str_bytes/6.4.1/download"],
        strip_prefix = "os_str_bytes-6.4.1",
        build_file = Label("@cxx.rs//third-party/bazel:BUILD.os_str_bytes-6.4.1.bazel"),
    )

    maybe(
        http_archive,
        name = "vendor__proc-macro2-1.0.51",
        sha256 = "5d727cae5b39d21da60fa540906919ad737832fe0b1c165da3a34d6548c849d6",
        type = "tar.gz",
        urls = ["https://crates.io/api/v1/crates/proc-macro2/1.0.51/download"],
        strip_prefix = "proc-macro2-1.0.51",
        build_file = Label("@cxx.rs//third-party/bazel:BUILD.proc-macro2-1.0.51.bazel"),
    )

    maybe(
        http_archive,
        name = "vendor__quote-1.0.23",
        sha256 = "8856d8364d252a14d474036ea1358d63c9e6965c8e5c1885c18f73d70bff9c7b",
        type = "tar.gz",
        urls = ["https://crates.io/api/v1/crates/quote/1.0.23/download"],
        strip_prefix = "quote-1.0.23",
        build_file = Label("@cxx.rs//third-party/bazel:BUILD.quote-1.0.23.bazel"),
    )

    maybe(
        http_archive,
        name = "vendor__scratch-1.0.5",
        sha256 = "1792db035ce95be60c3f8853017b3999209281c24e2ba5bc8e59bf97a0c590c1",
        type = "tar.gz",
        urls = ["https://crates.io/api/v1/crates/scratch/1.0.5/download"],
        strip_prefix = "scratch-1.0.5",
        build_file = Label("@cxx.rs//third-party/bazel:BUILD.scratch-1.0.5.bazel"),
    )

    maybe(
        http_archive,
        name = "vendor__syn-1.0.109",
        sha256 = "72b64191b275b66ffe2469e8af2c1cfe3bafa67b529ead792a6d0160888b4237",
        type = "tar.gz",
        urls = ["https://crates.io/api/v1/crates/syn/1.0.109/download"],
        strip_prefix = "syn-1.0.109",
        build_file = Label("@cxx.rs//third-party/bazel:BUILD.syn-1.0.109.bazel"),
    )

    maybe(
        http_archive,
        name = "vendor__termcolor-1.2.0",
        sha256 = "be55cf8942feac5c765c2c993422806843c9a9a45d4d5c407ad6dd2ea95eb9b6",
        type = "tar.gz",
        urls = ["https://crates.io/api/v1/crates/termcolor/1.2.0/download"],
        strip_prefix = "termcolor-1.2.0",
        build_file = Label("@cxx.rs//third-party/bazel:BUILD.termcolor-1.2.0.bazel"),
    )

    maybe(
        http_archive,
        name = "vendor__unicode-ident-1.0.8",
        sha256 = "e5464a87b239f13a63a501f2701565754bae92d243d4bb7eb12f6d57d2269bf4",
        type = "tar.gz",
        urls = ["https://crates.io/api/v1/crates/unicode-ident/1.0.8/download"],
        strip_prefix = "unicode-ident-1.0.8",
        build_file = Label("@cxx.rs//third-party/bazel:BUILD.unicode-ident-1.0.8.bazel"),
    )

    maybe(
        http_archive,
        name = "vendor__unicode-width-0.1.10",
        sha256 = "c0edd1e5b14653f783770bce4a4dabb4a5108a5370a5f5d8cfe8710c361f6c8b",
        type = "tar.gz",
        urls = ["https://crates.io/api/v1/crates/unicode-width/0.1.10/download"],
        strip_prefix = "unicode-width-0.1.10",
        build_file = Label("@cxx.rs//third-party/bazel:BUILD.unicode-width-0.1.10.bazel"),
    )

    maybe(
        http_archive,
        name = "vendor__winapi-0.3.9",
        sha256 = "5c839a674fcd7a98952e593242ea400abe93992746761e38641405d28b00f419",
        type = "tar.gz",
        urls = ["https://crates.io/api/v1/crates/winapi/0.3.9/download"],
        strip_prefix = "winapi-0.3.9",
        build_file = Label("@cxx.rs//third-party/bazel:BUILD.winapi-0.3.9.bazel"),
    )

    maybe(
        http_archive,
        name = "vendor__winapi-i686-pc-windows-gnu-0.4.0",
        sha256 = "ac3b87c63620426dd9b991e5ce0329eff545bccbbb34f3be09ff6fb6ab51b7b6",
        type = "tar.gz",
        urls = ["https://crates.io/api/v1/crates/winapi-i686-pc-windows-gnu/0.4.0/download"],
        strip_prefix = "winapi-i686-pc-windows-gnu-0.4.0",
        build_file = Label("@cxx.rs//third-party/bazel:BUILD.winapi-i686-pc-windows-gnu-0.4.0.bazel"),
    )

    maybe(
        http_archive,
        name = "vendor__winapi-util-0.1.5",
        sha256 = "70ec6ce85bb158151cae5e5c87f95a8e97d2c0c4b001223f33a334e3ce5de178",
        type = "tar.gz",
        urls = ["https://crates.io/api/v1/crates/winapi-util/0.1.5/download"],
        strip_prefix = "winapi-util-0.1.5",
        build_file = Label("@cxx.rs//third-party/bazel:BUILD.winapi-util-0.1.5.bazel"),
    )

    maybe(
        http_archive,
        name = "vendor__winapi-x86_64-pc-windows-gnu-0.4.0",
        sha256 = "712e227841d057c1ee1cd2fb22fa7e5a5461ae8e48fa2ca79ec42cfc1931183f",
        type = "tar.gz",
        urls = ["https://crates.io/api/v1/crates/winapi-x86_64-pc-windows-gnu/0.4.0/download"],
        strip_prefix = "winapi-x86_64-pc-windows-gnu-0.4.0",
        build_file = Label("@cxx.rs//third-party/bazel:BUILD.winapi-x86_64-pc-windows-gnu-0.4.0.bazel"),
    )
