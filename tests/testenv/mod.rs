use std::path::{Path, PathBuf};
use std::process::Output;
use std::{env, process};

use tempfile::TempDir;

// Environment for the integration tests.
pub struct TestEnv {
    // Temporary working directory.
    pub temp_dir: TempDir,
    // Path to the tap executable.
    tap_exe: PathBuf,
}

impl TestEnv {
    pub fn new(
        dirs: &[&'static str],
        audio_files: &[(&'static str, &'static str)],
        dummy_files: &[&'static str],
    ) -> TestEnv {
        let temp_dir =
            create_working_dir(dirs, audio_files, dummy_files).expect("working directory");
        let tap_exe = find_exe();
        TestEnv { temp_dir, tap_exe }
    }

    // Assert that calling tap with the specified arguments produces the expected error.
    pub fn assert_success(&self, args: &[&str]) {
        let output = self.run_command(".".as_ref(), args);
        assert!(output.status.success())
    }

    // Assert that calling tap with the specified arguments produces the expected error.
    pub fn assert_error_msg(&self, args: &[&str], expected: &str) {
        let output = self.run_command(".".as_ref(), args);
        let stderr = String::from_utf8(output.stderr).expect("error message should be utf8");

        assert!(
            stderr.contains(expected),
            "\nThe error message:\n`{}`\n\
            does not contain the expected message:\n`{}`\n",
            stderr,
            expected
        );
    }

    pub fn assert_normalized_paths(&self, expected: &[&str]) {
        let output = self.run_command(".".as_ref(), &[]);
        let stderr = normalize(output);

        for path in expected.iter() {
            assert!(
                stderr.contains(&path.to_string()),
                "\nThe list of paths:\n{:?}\n\
                does not contain the expected path:\n`{}`\n",
                stderr,
                path
            );
        }

        assert_eq!(stderr.len(), expected.len());
    }

    fn run_command(&self, path: &Path, args: &[&str]) -> process::Output {
        let mut cmd = process::Command::new(&self.tap_exe);
        cmd.current_dir(self.temp_dir.path().join(path));
        cmd.args(args);

        cmd.output().expect("tap output")
    }
}

pub fn create_working_dir(
    dirs: &[&'static str],
    audio_data: &[(&'static str, &'static str)],
    dummy_data: &[&'static str],
) -> Result<tempfile::TempDir, std::io::Error> {
    let temp_dir = tempfile::Builder::new()
        .prefix("tap-tests")
        .tempdir()
        .expect("failed to create temporary directory");

    let assets_dir = find_assets_dir();

    for path in dirs {
        let path = temp_dir.path().join(path);
        std::fs::create_dir_all(path).expect("failed to create subdirectories")
    }

    for (temp_path, asset_path) in audio_data {
        let src = assets_dir.join(asset_path);
        let dest = temp_dir.path().join(temp_path);
        std::fs::copy(src, dest).expect("failed to copy audio data");
    }

    for path in dummy_data {
        let path = temp_dir.path().join(path);
        std::fs::File::create(path).expect("failed to create dummy data");
    }

    Ok(temp_dir)
}

// Find the test assets.
pub fn find_assets_dir() -> std::path::PathBuf {
    // Tests exe is in target/debug/deps, test assets are in tests/assets

    let root = std::env::current_exe()
        .expect("tests executable")
        .parent()
        .expect("tests executable directory")
        .parent()
        .expect("tap executable directory")
        .parent()
        .expect("target directory")
        .parent()
        .expect("project root")
        .to_path_buf();

    root.join("tests").join("assets")
}

fn normalize(output: Output) -> Vec<String> {
    let stderr = String::from_utf8(output.stderr).unwrap();
    let slice = &stderr[38..];
    let end = slice.find("]").unwrap();

    slice[..end]
        .split(",")
        .map(|s| {
            let s = String::from(s);
            s[75..s.len() - 1].to_string()
        })
        .collect::<Vec<_>>()
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
