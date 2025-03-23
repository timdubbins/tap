use std::process::Command;

fn main() {
    if cfg!(target_os = "macos") {
        let output = Command::new("brew")
            .args(&["--prefix", "ncurses"])
            .output()
            .expect("Failed to run `brew --prefix ncurses`. Make sure Homebrew is installed.");

        if !output.status.success() {
            panic!("Failed to locate Homebrew ncurses. Please ensure ncurses is installed via Homebrew.");
        }

        let prefix = String::from_utf8_lossy(&output.stdout).trim().to_string();

        println!("cargo:rustc-link-search=native={}/lib", prefix);
        println!("cargo:rustc-link-lib=dylib=ncurses");
        println!("cargo:include={}/include", prefix);
    } else {
        println!("cargo:rustc-link-lib=ncurses");
    }
}
