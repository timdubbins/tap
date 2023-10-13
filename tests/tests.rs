mod testenv;
#[path = "../src/utils.rs"]
#[allow(dead_code)]
mod utils;

use crate::testenv::TestEnv;

static DEFAULT_DIRS: &[&str] = &[
    "one/two_a",
    "one/two_b",
    "one/two_c",
    "one/.hidden_dir",
    "one/two_c/empty_dir",
];

static DEFAULT_FILES: &[&str] = &[
    "a.foo",
    "one/b.foo",
    "one/two/c.foo",
    "one/two/C.Foo2",
    "one/two/three/d.foo",
    "fdignored.foo",
    "gitignored.foo",
    ".hidden.foo",
    "e1 e2",
];

#[test]
fn test_empty_dir_error() {
    let te = TestEnv::new(&["one", "one/two"], &[], &[]);
    te.assert_failure_with_error(&[], "is empty")
}

#[test]
fn test_no_audio_error() {
    let te = TestEnv::new(DEFAULT_DIRS, &[], &["invalid_audio.mp3", "one/foo.txt"]);
    te.assert_failure_with_error(&[], "no audio")
}

#[test]
fn test_no_metadata_error() {
    let te = TestEnv::new(&["one"], &["one/a.mp3"], &[]);
    te.assert_failure_with_error(&[], "")
}
