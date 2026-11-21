```
running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 18 tests
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 2 tests
test resident::the_public_service_scans_and_reports_the_embedded_vector ... FAILED
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
TEST_EXIT=101
---- resident::the_public_service_scans_and_reports_the_embedded_vector stdout ----

thread 'resident::the_public_service_scans_and_reports_the_embedded_vector' (3177) panicked at crates/odina-cli/tests/public_build.rs:73:9:
assertion `left == right` failed: {"error":"busy"}
 
  left: Some(1)
 right: Some(0)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    resident::the_public_service_scans_and_reports_the_embedded_vector

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

[1m[91merror[0m: test failed, to rerun pass `-p odina-cli --test public_build`
TEST_EXIT=101
```
