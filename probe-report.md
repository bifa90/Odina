# probe output

clippy exit: 101
```
[1m[92m    Updating[0m crates.io index
[1m[92m Downloading[0m crates ...
[1m[92m  Downloaded[0m block-buffer v0.10.4
[1m[92m  Downloaded[0m itoa v1.0.11
[1m[92m  Downloaded[0m crypto-common v0.1.7
[1m[92m  Downloaded[0m cpufeatures v0.2.17
[1m[92m  Downloaded[0m version_check v0.9.5
[1m[92m  Downloaded[0m cfg-if v1.0.5
[1m[92m  Downloaded[0m generic-array v0.14.7
[1m[92m  Downloaded[0m digest v0.10.7
[1m[92m  Downloaded[0m sha2 v0.10.9
[1m[92m  Downloaded[0m quote v1.0.36
[1m[92m  Downloaded[0m ryu v1.0.18
[1m[92m  Downloaded[0m proc-macro2 v1.0.85
[1m[92m  Downloaded[0m unicode-ident v1.0.12
[1m[92m  Downloaded[0m serde_derive v1.0.203
[1m[92m  Downloaded[0m serde v1.0.203
[1m[92m  Downloaded[0m typenum v1.20.1
[1m[92m  Downloaded[0m serde_json v1.0.117
[1m[92m  Downloaded[0m syn v2.0.66
[1m[92m  Downloaded[0m libc v0.2.191
[1m[92m   Compiling[0m version_check v0.9.5
[1m[92m    Checking[0m typenum v1.20.1
[1m[92m    Checking[0m cpufeatures v0.2.17
[1m[92m    Checking[0m cfg-if v1.0.5
[1m[92m   Compiling[0m libc v0.2.191
[1m[92m   Compiling[0m proc-macro2 v1.0.85
[1m[92m   Compiling[0m unicode-ident v1.0.12
[1m[92m    Checking[0m odina-report v0.1.0 (/home/runner/work/Odina/Odina/crates/odina-report)
[1m[92m   Compiling[0m generic-array v0.14.7
[1m[92m   Compiling[0m serde v1.0.203
[1m[92m   Compiling[0m serde_json v1.0.117
[1m[92m    Checking[0m itoa v1.0.11
[1m[92m    Checking[0m ryu v1.0.18
[1m[92m    Checking[0m odina-instrument v0.1.0 (/home/runner/work/Odina/Odina/crates/odina-instrument)
[1m[92m    Checking[0m block-buffer v0.10.4
[1m[92m    Checking[0m crypto-common v0.1.7
[1m[92m    Checking[0m digest v0.10.7
[1m[92m    Checking[0m sha2 v0.10.9
[1m[92m   Compiling[0m quote v1.0.36
[1m[92m   Compiling[0m syn v2.0.66
[1m[92m    Checking[0m odina-resident v0.1.0 (/home/runner/work/Odina/Odina/crates/odina-resident)
[1m[92m    Checking[0m odina-core v0.1.0 (/home/runner/work/Odina/Odina/crates/odina-core)
[1m[91merror[0m[1m: consider using `sort_by_key`[0m
   [1m[94m--> [0mcrates/odina-core/src/update_check.rs:156:9
    [1m[94m|[0m
[1m[94m156[0m [1m[94m|[0m         rows.sort_by(|a, b| b.version.cmp(&a.version));
    [1m[94m|[0m         [1m[91m^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m
    [1m[94m|[0m
    [1m[94m= [0m[1mhelp[0m: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.100.0/index.html#unnecessary_sort_by
    [1m[94m= [0m[1mnote[0m: `-D clippy::unnecessary-sort-by` implied by `-D warnings`
    [1m[94m= [0m[1mhelp[0m: to override `-D warnings` add `#[allow(clippy::unnecessary_sort_by)]`
[1m[96mhelp[0m: try
    [1m[94m|[0m
[1m[94m156[0m [91m- [0m        rows.[91msort_by(|a, b| b.version.cmp(&a.version))[0m;
[1m[94m156[0m [92m+ [0m        rows.[92msort_by_key(|a| std::cmp::Reverse(a.version))[0m;
    [1m[94m|[0m

[1m[91merror[0m: could not compile `odina-core` (lib) due to 1 previous error
[1m[33mwarning[0m: build failed, waiting for other jobs to finish...
[1m[91merror[0m[1m: this assertion has a constant value[0m
   [1m[94m--> [0mcrates/odina-core/src/report_submission.rs:429:9
    [1m[94m|[0m
[1m[94m429[0m [1m[94m|[0m         assert!(!USER_REPORT_IS_INDEPENDENT_ORIGIN);
    [1m[94m|[0m         [1m[91m^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m
    [1m[94m|[0m
    [1m[94m= [0m[1mhelp[0m: consider moving this to an anonymous constant: `const _: () = { assert!(..); }`
    [1m[94m= [0m[1mhelp[0m: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.100.0/index.html#assertions_on_constants
    [1m[94m= [0m[1mnote[0m: `-D clippy::assertions-on-constants` implied by `-D warnings`
    [1m[94m= [0m[1mhelp[0m: to override `-D warnings` add `#[allow(clippy::assertions_on_constants)]`

[1m[91merror[0m: could not compile `odina-core` (lib test) due to 2 previous errors
```
test exit: 0
```
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 139 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test pe::tests::a_truncated_header_is_rejected_not_panicked_on ... ok
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
