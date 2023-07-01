# tap

tap is an audio player for the terminal, written in Rust. It's a fast, minimal player that gives you access to the albums in your library through fuzzy finding shortcuts.

<img src="https://github.com/timdubbins/tap/blob/master/doc/tap_screenshot.png" width="650"/>

## How to use

```bash
> tap [path]
```

The path argument is optional. If it is omitted the current directory is used. Running the above command will:

- open and play the file(s) if path is an audio file or a directory containing audio files (i.e. an album).

- open a fuzzy search, allowing you to select an album to play, if path is a directory that contains subdirectories (such as your root music folder). Playback starts on selection and you can return to search mode by pressing `TAB`.

**Tip**: Create an `alias` that provides a default path to your root music folder. Do this by putting something like the following in your shell config (for zsh users this could be in your .zshrc) and then source or restart your shell:

``` bash
alias tap="tap ~/path/to/my_music"
```

Passing in a second path argument will overide the path provided in the alias, so we now have the following behaviour:
``` bash
> tap                       # start by searching albums in `.../my_music`
> tap .                     # runs tap in the current directory
> tap ~/path/to/album       # starts playback of `.../album` files
> tap ~/path/to/album/file  # starts playback of `.../file`
```

## Bindings

Command | Keybinding
---|---
new fuzzy search | `TAB`
random selection | `r`
previous selection | `R`
play or pause | `p` or `SPACE`
stop | `.` or `s`
next | `j` or `DOWN` or `l` or `RIGHT`
previous | `k` or `UP` or `h` or `LEFT`
go to first track | `gg`
go to last track | `G`
go to track number | `0...9` + `g` or `ENTER`
toggle mute | `m`
quit | `q`

## Notes

- The currently supported formats are: `aac`, `flac`, `mp3`, `m4a`, `ogg` and `wav`.
- tap currently relies on metadata. If there is an issue with playback of a file it is possible that this is due to incorrect audio tags on the file.

## Installation

**1. Install tap.**

<details>
<summary>If you're on <b>macOS</b> you can use <a href="https://brew.sh/">Homebrew</a>:</summary>
<br>

```bash
> brew install timdubbins/tap/tap
> tap --version
0.1.1
```
</details>

<details>
<summary>If you're on <b>Arch</b> you can grab the <a href="https://aur.archlinux.org/packages/tap">AUR package</a>.
Or you can automate the install process with an <a href="https://wiki.archlinux.org/title/AUR_helpers">AUR helper</a>,
such as <a href="https://github.com/Jguer/yay">yay</a>:</summary>
<br>

```bash
> yay -S tap
> tap --version
0.1.1
```
</details>

<details>
<summary>If you're a <b>Debian</b> user (or a user of a Debian derivative like <b>Ubuntu</b> then tap can be installed using a binary <code>.deb</code> file provided in each <a href="https://github.com/timdubbins/tap/releases">tap release</a>.</summary>
<br>

```bash
> curl -LO https://github.com/timdubbins/tap/releases/download/v0.1.1/tap_v0.1.1_amd64.deb
> sudo dpkg -i tap_0.1.1_amd64.deb
> tap --version
0.1.1
```
</details>

<details>
<summary>To compile from source, first you need a <a href="https://www.rust-lang.org/">Rust installation</a> (if you don't have one) and then you can use <a href="https://github.com/rust-lang/cargo">cargo</a>:</summary>
<br>

```bash
> git clone https://github.com/timdubbins/tap
> cd tap
> cargo install --path .
> tap --version
0.1.1
```
</details>

The binaries for each release are also available [here](https://github.com/timdubbins/tap/releases/tag/v0.1.1).

**2. Install [fzf](https://github.com/junegunn/fzf) or [skim](https://github.com/lotabout/skim) (optional).** fzf is a very popular (and useful!) fuzzy finder, and skim is a Rust alternative to fzf. Installing either program will enable fuzzy-finding in tap.

**3. Install [fd]() (optional).** fd is a fast alternative to find and tap will use fd if it's installed on your machine.

### Building

For Rust programmers, tap can be built in the usual manner with:
```bash
> cargo build [options]
```

### Inspired by

- [cmus](https://github.com/cmus/cmus) - popular console music player with many features

### Made possible by

- [cursive](https://github.com/gyscos/cursive) - TUI library for Rust with great documentation
- [rodio](https://github.com/RustAudio/rodio) - audio playback library for Rust
