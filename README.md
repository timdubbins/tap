tap
------------
tap is a TUI-audio-player. It's a fast, minimal player that utilizes `fzf` and `fd` commands for rapid file selecting.

<img src="https://github.com/timdubbins/tap/blob/master/doc/tap_screenshot.png" width="650"/>

### How to use
1. Run `$ tap [PATH]` (or just `$ tap` to use the current directory as the argument).

2. If `[PATH]` can be played it is loaded into the player. If `[PATH]` contains directories (such as a root music folder) then tap opens in `Fuzzy` mode, allowing you to find and select the folder to load into the player.

3. Playback starts as soon as the player loads.

### Bindings

- `TAB` - start new fuzzy search in `[PATH]`

- `0...9` + `ENTER` - select track by number

- `p` - play / pause

- `s` - stop

- `j` - next

- `k` - previous

- `q` - quit

### Note

- `Fuzzy` mode depends on [`fzf`](https://github.com/junegunn/fzf) and [`fd`](https://github.com/sharkdp/fd) being installed (tap looks for `fzf` and `fd` in `$PATH`).

- If you used `cargo build` instead of `cargo install` you will need to replace `tap` with the path to the executable.
 Alternatively, create an alias with `$ alias tap='/path/to/executable'`.


### Installation
If you're a Rust programmer, tap can be installed with `cargo`.

- Note that the binary may be bigger than expected because it contains debug symbols.
To remove debug symbols run strip on the binary.

Run this command from inside the `tap` directory.

`$ cargo install --path .`


### Building

tap is written in Rust, so you'll need to grab a
[Rust installation](https://www.rust-lang.org/) in order to compile it.

To build tap:

```
$ git clone https://github.com/timdubbins/tap
$ cd tap
$ cargo build --release
$ ./target/release/tap --version
0.1.0
```

### Inspired by
- [cmus](https://github.com/cmus/cmus) - popular console music player with many features

### Made possible by
- [fzf](https://github.com/junegunn/fzf) - general-purpose command-line fuzzy finder
- [fd](https://github.com/sharkdp/fd) - very fast alternative to 'find'
- [cursive](https://github.com/gyscos/cursive) - TUI library for Rust with great documentation
- [rodio](https://github.com/RustAudio/rodio) - audio playback library for Rust
