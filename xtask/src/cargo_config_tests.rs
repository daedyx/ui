use super::*;

const BASE: &str = "# Kommentar\n[alias]\nxtask = \"run --quiet --package xtask --\"\n";

#[test]
fn only_the_xtask_alias_is_allowed() {
    assert!(config_problems(".cargo/config.toml", "", BASE).is_empty());
    let hostile = [
        "[target.x86_64-pc-windows-msvc]\nrunner = \"tools/ok.cmd\"\n",
        "[target.'cfg(windows)']\nrunner = \"tools/ok.cmd\"\n",
        "[build]\nrustc-wrapper = \"tools/w.cmd\"\n",
        "[build]\nrustc-workspace-wrapper = \"tools/w.cmd\"\n",
        "[build]\nrustc = \"tools/rustc.cmd\"\n",
        "[build]\nrustflags = [\"--cap-lints\", \"allow\"]\n",
        "[target.x86_64-pc-windows-msvc]\nlinker = \"tools/l.cmd\"\n",
        "[env]\nRUSTC_BOOTSTRAP = \"1\"\n",
        "[alias]\nclippy = \"check\"\n",
        "[profile.dev]\noverflow-checks = false\n",
    ];
    for h in hostile {
        assert!(
            !config_problems(".cargo/config.toml", "", &format!("{BASE}{h}")).is_empty(),
            "{h}"
        );
    }
    assert!(
        !config_problems(".cargo/config.toml", "", "[alias]\nxtask = \"version\"\n").is_empty()
    );
}

#[test]
fn config_files_elsewhere_and_toolchain_files_are_red() {
    for p in [
        "crates/a/.cargo/config.toml",
        ".cargo/config",
        "xtask/.cargo/config",
    ] {
        assert!(!config_problems(p, "", BASE).is_empty(), "{p}");
    }
    for p in [
        "rust-toolchain",
        "rust-toolchain.toml",
        "crates/a/rust-toolchain.toml",
    ] {
        assert!(is_toolchain_file(p), "{p}");
    }
    assert!(!is_toolchain_file("docs/rust-toolchain.md"));
    assert!(
        is_config_file(".CARGO/Config.toml") && is_toolchain_file("Rust-Toolchain.TOML"),
        "Windows: Schreibweise egal"
    );
    assert!(
        is_config_file(".cargo/config.toml")
            && is_config_file("a/.cargo/config")
            && !is_config_file(".cargo/audit.toml")
    );
}

#[test]
fn workflows_may_not_change_the_compiler_or_its_settings() {
    let ok = "    steps:\n      - run: cargo test --workspace --locked --offline\n      - run: ./target/debug/xtask.exe scan\n";
    assert!(workflow_sites(ok).is_empty());
    let cases = [
        ("env:\n  RUSTFLAGS: \"-A warnings\"\n", "workflow:rustflags"),
        (
            "      env:\n        RUSTC_WRAPPER: tools/w.cmd\n",
            "workflow:rustc_wrapper",
        ),
        (
            "        CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUNNER: ok.cmd\n",
            "workflow:cargo_target_",
        ),
        (
            "      - run: $env:CARGO_BUILD_RUSTC_WRAPPER = 'w'\n",
            "workflow:cargo_build_",
        ),
        (
            "      - run: cargo clippy --config build.jobs=1 -- -D warnings\n",
            "workflow:--config",
        ),
        ("      - run: cargo +nightly test\n", "workflow:cargo +"),
        ("        CARGO_HOME: ./eigen\n", "workflow:cargo_home"),
    ];
    for (text, want) in cases {
        assert_eq!(workflow_sites(text), vec![want.to_string()], "{text}");
    }
    // Nur erwähnt (z. B. in einer Liste verbotener Namen), nicht gesetzt: keine Stelle.
    assert!(workflow_sites("        $verboten = 'RUSTC_WRAPPER', 'RUSTFLAGS'\n").is_empty());
}
