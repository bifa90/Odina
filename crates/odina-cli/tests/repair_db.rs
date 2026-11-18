// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! The component can be removed/installed without replacing the program.
//! Resolving it from the working directory would be an unsafe substitution.

use odina_core::repair_db::COMPONENT_PATH;
use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn only_load_matching_external_component_beside_the_binary() {
    let source = Path::new(env!("CARGO_BIN_EXE_odina"));
    let root = std::env::temp_dir().join(format!("odina-repair-test-{}", std::process::id()));
    let dir = root.join("app");
    fs::create_dir_all(&dir).unwrap();
    let binary = dir.join(source.file_name().unwrap());
    fs::copy(source, &binary).unwrap();
    let data = include_bytes!("../../../components/repair/odina-repair-v1.tsv");
    let elsewhere = root.join(COMPONENT_PATH);
    fs::create_dir_all(elsewhere.parent().unwrap()).unwrap();
    fs::write(elsewhere, data).unwrap();
    let run = || {
        Command::new(&binary)
            .current_dir(&root)
            .args(["repair-db", "--demo"])
            .output()
            .unwrap()
    };
    let missing = run();
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("No recipe will run"));
    let component = dir.join(COMPONENT_PATH);
    fs::create_dir_all(component.parent().unwrap()).unwrap();
    fs::write(&component, b"untrusted").unwrap();
    assert!(!run().status.success());
    fs::write(&component, data).unwrap();
    let result = run();
    assert!(result.status.success(), "{:?}", result.stderr);
    let text = String::from_utf8(result.stdout).unwrap();
    for outcome in [
        "2 training recipes",
        "note: disinfected",
        "settings: disinfected",
        "standalone: quarantine",
        "unsupported: quarantine",
    ] {
        assert!(text.contains(outcome), "{text}");
    }
    assert_eq!(fs::read(component).unwrap(), data);
    fs::remove_dir_all(root).unwrap();
}
