# probe output
clippy exit: 101
```
[1m[92m    Updating[0m crates.io index
[1m[92m Downloading[0m crates ...
[1m[92m  Downloaded[0m typenum v1.20.1
[1m[92m  Downloaded[0m block-buffer v0.10.4
[1m[92m  Downloaded[0m generic-array v0.14.7
[1m[92m  Downloaded[0m cfg-if v1.0.5
[1m[92m  Downloaded[0m cpufeatures v0.2.17
[1m[92m  Downloaded[0m crypto-common v0.1.7
[1m[92m  Downloaded[0m itoa v1.0.11
[1m[92m  Downloaded[0m version_check v0.9.5
[1m[92m  Downloaded[0m digest v0.10.7
[1m[92m  Downloaded[0m sha2 v0.10.9
[1m[92m  Downloaded[0m quote v1.0.36
[1m[92m  Downloaded[0m proc-macro2 v1.0.85
[1m[92m  Downloaded[0m unicode-ident v1.0.12
[1m[92m  Downloaded[0m serde v1.0.203
[1m[92m  Downloaded[0m serde_derive v1.0.203
[1m[92m  Downloaded[0m ryu v1.0.18
[1m[92m  Downloaded[0m serde_json v1.0.117
[1m[92m  Downloaded[0m syn v2.0.66
[1m[92m  Downloaded[0m libc v0.2.191
[1m[92m   Compiling[0m version_check v0.9.5
[1m[92m    Checking[0m typenum v1.20.1
[1m[92m    Checking[0m cfg-if v1.0.5
[1m[92m    Checking[0m cpufeatures v0.2.17
[1m[92m   Compiling[0m libc v0.2.191
[1m[92m   Compiling[0m proc-macro2 v1.0.85
[1m[92m   Compiling[0m unicode-ident v1.0.12
[1m[92m    Checking[0m odina-report v0.1.0 (/home/runner/work/Odina/Odina/crates/odina-report)
[1m[92m   Compiling[0m generic-array v0.14.7
[1m[92m   Compiling[0m serde v1.0.203
[1m[92m   Compiling[0m serde_json v1.0.117
[1m[92m    Checking[0m ryu v1.0.18
[1m[92m    Checking[0m itoa v1.0.11
[1m[92m    Checking[0m odina-instrument v0.1.0 (/home/runner/work/Odina/Odina/crates/odina-instrument)
[1m[92m    Checking[0m block-buffer v0.10.4
[1m[92m    Checking[0m crypto-common v0.1.7
[1m[92m    Checking[0m digest v0.10.7
[1m[92m    Checking[0m sha2 v0.10.9
[1m[92m   Compiling[0m quote v1.0.36
[1m[92m   Compiling[0m syn v2.0.66
[1m[92m    Checking[0m odina-resident v0.1.0 (/home/runner/work/Odina/Odina/crates/odina-resident)
[1m[92m    Checking[0m odina-core v0.1.0 (/home/runner/work/Odina/Odina/crates/odina-core)
[1m[92m    Checking[0m odina-scan v0.1.0 (/home/runner/work/Odina/Odina/crates/odina-scan)
[1m[92m    Checking[0m odina-cache v0.1.0 (/home/runner/work/Odina/Odina/crates/odina-cache)
[1m[92m    Checking[0m odina-cli v0.1.0 (/home/runner/work/Odina/Odina/crates/odina-cli)
[1m[91merror[0m[1m: useless use of `format!`[0m
   [1m[94m--> [0mcrates/odina-core/src/path_privacy.rs:356:32
    [1m[94m|[0m
[1m[94m356[0m [1m[94m|[0m         assert!(!is_normalized(&format!("{HOME}")));
    [1m[94m|[0m                                [1m[91m^^^^^^^^^^^^^^^^^^[0m [1m[91mhelp: consider using the argument directly: `HOME`[0m
    [1m[94m|[0m
    [1m[94m= [0m[1mhelp[0m: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.100.0/index.html#useless_format
    [1m[94m= [0m[1mnote[0m: `-D clippy::useless-format` implied by `-D warnings`
    [1m[94m= [0m[1mhelp[0m: to override `-D warnings` add `#[allow(clippy::useless_format)]`

[1m[91merror[0m: could not compile `odina-core` (lib test) due to 1 previous error
[1m[33mwarning[0m: build failed, waiting for other jobs to finish...
```
test exit: 101
```
test path_privacy::tests::every_environment_token_is_reachable_from_a_real_windows_path ... FAILED
test path_privacy::tests::the_location_is_kept_wherever_the_file_hides ... FAILED
thread 'path_privacy::tests::every_environment_token_is_reachable_from_a_real_windows_path' (3349) panicked at crates/odina-core/src/path_privacy.rs:376:13:
thread 'path_privacy::tests::the_location_is_kept_wherever_the_file_hides' (3352) panicked at crates/odina-core/src/path_privacy.rs:280:9:
assertion `left == right` failed
test result: FAILED. 143 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
