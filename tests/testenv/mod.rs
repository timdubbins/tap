#[path = "../../src/utils.rs"]
#[allow(dead_code)]
mod utils;

use std::path::{Path, PathBuf};
use std::{env, process};

use tempfile::TempDir;

use utils::create_working_dir;

// Environment for the integration tests.
pub struct TestEnv {
    // Temporary working directory.
    temp_dir: TempDir,
    // Path to the tap executable.
    tap_exe: PathBuf,
}

impl TestEnv {
    pub fn new(
        dirs: &[&'static str],
        audio_files: &[&'static str],
        dummy_files: &[&'static str],
    ) -> TestEnv {
        let temp_dir =
            create_working_dir(dirs, audio_files, dummy_files).expect("working directory");
        let tap_exe = find_exe();

        TestEnv { temp_dir, tap_exe }
    }

    // Assert that calling tap with the specified arguments produces the expected error.
    pub fn assert_failure_with_error(&self, args: &[&str], expected: &str) {
        let output = self.run_command(".".as_ref(), args);
        let stderr = String::from_utf8(output.stderr).expect("error message should be utf8");

        assert!(
            stderr.contains(expected),
            "\n\
            ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~\n\
            The error message:\n\
            {:?}\n\
            does not contain the expected message:\n\
            {:?}\n\
            ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~\n",
            stderr,
            expected
        );
    }

    fn run_command(&self, path: &Path, args: &[&str]) -> process::Output {
        let mut cmd = process::Command::new(&self.tap_exe);
        cmd.current_dir(self.temp_dir.path().join(path));
        cmd.args(args);

        cmd.output().expect("tap output")
    }
}

// Find the tap executable.
fn find_exe() -> PathBuf {
    // Tests exe is in target/debug/deps, the tap exe is in target/debug
    let root = env::current_exe()
        .expect("tests executable")
        .parent()
        .expect("tests executable directory")
        .parent()
        .expect("tap executable directory")
        .to_path_buf();

    root.join("tap")
}
