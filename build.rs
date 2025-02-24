use std::env;

fn main() {
    if cfg!(target_os = "macos") {
        let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
        let brew_ncurses = match arch.as_str() {
            "aarch64" => "/opt/homebrew/opt/ncurses", // Apple Silicon
            "x86_64" => "/usr/local/opt/ncurses",     // Intel
            _ => panic!("Unsupported architecture: {}", arch),
        };

        println!("cargo:rustc-link-search=native={}/lib", brew_ncurses);
        println!("cargo:rustc-link-lib=ncurses");
    } else if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=ncurses");
    } else {
        panic!("Unsupported operating system");
    }
}
