mod testenv;
#[path = "../src/utils.rs"]
#[allow(dead_code)]
mod utils;

use crate::testenv::TestEnv;

#[test]
fn test_empty_dir_error() {
    let te = TestEnv::new(&["one", "one/two"], &[], &[]);
    te.assert_error_msg(&[], "is empty");
}

#[test]
fn test_no_audio_error() {
    let te = TestEnv::new(
        &["one/two_a", "one/two_b"],
        &[],
        &["invalid_audio.mp3", "one/foo.txt"],
    );
    te.assert_error_msg(&[], "no audio");
}

#[test]
fn test_decode_error() {
    let te = TestEnv::new(&["one"], &[("one/a.ogg", "test_ogg_audio.ogg")], &[]);
    te.assert_error_msg(&[], "could not decode");
}

#[test]
fn test_multiple_audio_files_success() {
    let te = TestEnv::new(
        &["one"],
        &[
            ("one/a.mp3", "test_mp3_audio.mp3"),
            ("one/b.flac", "test_flac_audio.flac"),
            ("one/c.wav", "test_wav_audio.wav"),
        ],
        &[],
    );
    te.assert_error_msg(&[], "success");
}

#[test]
fn test_automate_success() {
    let te = TestEnv::new(
        &["one", "one/two", "one/three"],
        &[
            ("one/two/a.mp3", "test_mp3_audio.mp3"),
            ("one/three/b.mp3", "test_mp3_audio.mp3"),
        ],
        &[],
    );
    te.assert_error_msg(&["--automate"], "success");
}

#[test]
fn test_arg_conflict_errors() {
    let te = TestEnv::new(&[], &[], &[]);

    te.assert_error_msg(&["-ad"], "cannot be used");
    te.assert_error_msg(&["-ap"], "cannot be used");
    te.assert_error_msg(&["-as"], "cannot be used");
    te.assert_error_msg(&["-dp"], "cannot be used");
    te.assert_error_msg(&["-ds"], "cannot be used");
    te.assert_error_msg(&["-ps"], "cannot be used");
}

#[test]
fn test_default_is_not_set_error() {
    let te = TestEnv::new(
        &["one", "two"],
        &[
            ("one/a.mp3", "test_mp3_audio.mp3"),
            ("two/b.mp3", "test_mp3_audio.mp3"),
        ],
        &[],
    );

    te.assert_error_msg(&["-d"], "set a default");
}

#[test]
fn test_find_two_audio_dirs() {
    let te = TestEnv::new(
        &["one", "two"],
        &[
            ("one/a.mp3", "test_mp3_audio.mp3"),
            ("one/b.mp3", "test_mp3_audio.mp3"),
            ("two/c.mp3", "test_mp3_audio.mp3"),
            ("two/d.mp3", "test_mp3_audio.mp3"),
        ],
        &[],
    );
    te.assert_normalized_paths(&["one", "two"]);
}

#[test]
fn test_exclude_empty_dir() {
    let te = TestEnv::new(
        &["one", "one/two", "one/empty_dir"],
        &[
            ("one/two/a.mp3", "test_mp3_audio.mp3"),
            ("one/two/b.mp3", "test_mp3_audio.mp3"),
        ],
        &[],
    );
    te.assert_normalized_paths(&["one/two"]);
}

#[test]
fn test_exclude_empty_leaf_but_include_audio_parent() {
    let te = TestEnv::new(
        &["one", "one/two", "one/two/empty_leaf"],
        &[
            ("one/two/a.mp3", "test_mp3_audio.mp3"),
            ("one/two/b.mp3", "test_mp3_audio.mp3"),
        ],
        &[],
    );
    te.assert_normalized_paths(&["one/two"]);
}

#[test]
fn test_exclude_non_audio_leaf_but_include_audio_parent() {
    let te = TestEnv::new(
        &["one", "one/two", "one/two/three"],
        &[
            ("one/two/a.mp3", "test_mp3_audio.mp3"),
            ("one/two/b.mp3", "test_mp3_audio.mp3"),
        ],
        &["one/two/three/c.foo"],
    );
    te.assert_normalized_paths(&["one/two"]);
}

#[test]
fn test_find_audio_in_root_dir() {
    let te = TestEnv::new(
        &["one", "one/two", "one/three", "one/empty_dir"],
        &[
            ("one/a.mp3", "test_mp3_audio.mp3"),
            ("one/two/b.mp3", "test_mp3_audio.mp3"),
            ("one/three/c.mp3", "test_mp3_audio.mp3"),
            ("one/three/d.mp3", "test_mp3_audio.mp3"),
        ],
        &[],
    );
    te.assert_normalized_paths(&["one", "one/two", "one/three"]);
}

#[test]
fn test_single_audio_dir() {
    let te = TestEnv::new(
        &["one"],
        &[
            ("one/a.mp3", "test_mp3_audio.mp3"),
            ("one/b.mp3", "test_mp3_audio.mp3"),
        ],
        &[],
    );
    te.assert_normalized_paths(&["success"]);
}
