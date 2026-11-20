// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
mod protocol;
use std::{env, io, process::ExitCode};
fn main() -> ExitCode {
    let args: Vec<_> = env::args_os().skip(1).collect();
    // Browsers enforce the allowlist in their registered manifest. Also reject unexpected launchers.
    if args.first().and_then(|s| s.to_str()) != Some(CHROME_ORIGIN)
        && args.get(1).and_then(|s| s.to_str()) != Some("context-menu@odina.local")
    {
        return ExitCode::FAILURE;
    }
    match protocol::once(&mut io::stdin().lock(), &mut io::stdout().lock()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
const CHROME_ORIGIN: &str = "chrome-extension://nlbkfeimaaaoemglhmgmjmiefpnlhdlp/";
