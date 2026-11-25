use super::*;

const LOG: &str = "   Compiling daedyx-tasks-core v0.2.0 (D:\\x\\crates\\daedyx-tasks-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 3.0s
     Running unittests src\\lib.rs (D:\\b\\debug\\deps\\daedyx_tasks_core-0f1e2d3c4b5a6978.exe)

running 136 tests
test result: ok. 136 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.81s

     Running unittests src/main.rs (/b/debug/deps/xtask-1234567890abcdef)
test result: ok. 54 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s

     Running tests\\e2e.rs (D:\\b\\debug\\deps\\e2e-aaaaaaaaaaaaaaaa.exe)
test result: ok. 3 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 5.3s

   Doc-tests daedyx_tasks_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
";

const FLOOR: &str = "# Mindestzahl bestandener Tests je Testprogramm
136 unittests src/lib.rs (daedyx_tasks_core)
54 unittests src/main.rs (xtask)
3 tests/e2e.rs (e2e)
";

#[test]
fn counts_are_read_per_test_program() {
    let c = counts(LOG);
    assert_eq!(
        c.get("unittests src/lib.rs (daedyx_tasks_core)")
            .map(|r| r.passed),
        Some(136)
    );
    assert_eq!(
        c.get("unittests src/main.rs (xtask)").map(|r| r.passed),
        Some(54)
    );
    assert_eq!(c.get("tests/e2e.rs (e2e)").map(|r| r.passed), Some(3));
    assert_eq!(
        c.get("doc-tests (daedyx_tasks_core)").map(|r| r.passed),
        Some(0)
    );
}

#[test]
fn the_floor_holds_only_if_no_program_shrinks_vanishes_or_appears_unlisted() {
    assert!(check(LOG, FLOOR).is_empty(), "{:?}", check(LOG, FLOOR));
    let fewer = LOG.replace("136 passed", "100 passed");
    assert_eq!(check(&fewer, FLOOR).len(), 1, "{:?}", check(&fewer, FLOOR));
    let renamed = LOG
        .replace("Running tests\\e2e.rs", "Running tests\\anders.rs")
        .replace("e2e-aaaa", "anders-aaaa");
    assert_eq!(
        check(&renamed, FLOOR).len(),
        2,
        "{:?}",
        check(&renamed, FLOOR)
    );
    let failed = LOG.replace("54 passed; 0 failed", "53 passed; 1 failed");
    assert!(
        check(&failed, FLOOR).len() >= 2,
        "{:?}",
        check(&failed, FLOOR)
    );
    assert!(!check("", FLOOR).is_empty(), "leeres Protokoll");
    assert!(!check(LOG, "zahl unittests x").is_empty(), "kaputte Zeile");
}
