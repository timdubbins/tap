# tap

tap is an audio player for the terminal, with fuzzy file selection.

<img src="https://github.com/timdubbins/tap/blob/master/doc/tap_screenshot.png" width="650"/>

### Who's it for?

If you want a fast, minimal player for the terminal that provides quick access to your entire library, tap may be for you.

## How to use

```bash
> tap [path]
```

`[path]` is optional and can be a file or directory. If omitted, tap uses the path of the current directory.

tap will then start in one of two states:

1. if the supplied path is an audio file, or a directory containing audio files (i.e. an album), tap will open and play the file(s).

2. if path is a directory that contains other directories (such as your root music folder), tap will open a fuzzy search, allowing you to select an album to play. Playback starts on selection and you can return to search mode by pressing `TAB`.

**Tip**: Create an `alias` that provides a default path to your root music folder:

``` bash
# 1. put this somewhere in your shell config (i.e. in your .zshrc for zsh users)
# 2. source the file or restart your shell

alias tap="tap ~/path/to/my_music"
```

Since we can overide this default path by passing in a second path argument, we now how the following behaviour:
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
play or pause | `p` or `SPACE`
stop | `.` or `s`
next | `j` or `DOWN` or `l` or `RIGHT`
previous | `k` or `UP` or `h` or `LEFT`
go to first track | `gg`
go to last track | `G`
go to track number | `0...9` + `g` or `ENTER`
quit | `q`

## Notes

- The currently supported formats are: `aac`, `flac`, `mp3`, `mp4`, `ogg` and `wav`.
- tap currently relies on metadata. If there is an issue with playback of a file it is possible that this is due to incorrect audio tags on the file.

## Installation

1. Install tap.

If you're on **macOS** you can use [Homebrew]():

```bash
> brew install timdubbins/tap/tap
> tap --version
0.1.1
```
If you're on **Arch** you can grab the [AUR package](https://aur.archlinux.org/packages/tap). Alternatively, you can automate the install process with an [AUR helper](https://wiki.archlinux.org/title/AUR_helpers), such as [yay](https://github.com/Jguer/yay):
```bash
> yay -S tap
> tap --version
0.1.1
```

If you're a **Debian** user (or a user of a Debian derivative like **Ubuntu**) then tap can be installed using a binary `.deb` file provided in each [tap release](https://github.com/timdubbins/tap/releases).

```bash
> curl -LO https://github.com/timdubbins/tap/releases/download/v0.1.1/tap_v0.1.1_amd64.deb
> sudo dpkg -i tap_0.1.1_amd64.deb
> tap --version
0.1.1
```

To compile from source, first you need a [Rust installation](https://www.rust-lang.org/) (if you don't have one) and then you can use [cargo](https://github.com/rust-lang/cargo):

```bash
> git clone https://github.com/timdubbins/tap
> cd tap
> cargo install --path .
> tap --version
0.1.1
```

Alternatively, the binaries are available [here](https://github.com/timdubbins/tap/releases/tag/v0.1.1).

2. Install [`fzf`](https://github.com/junegunn/fzf) or [`skim`](https://github.com/lotabout/skim). fzf is a very popular (and useful!) fuzzy finder, and skim is a Rust alternative to fzf. Installing either program will enable fuzzy-finding in tap.

3. Install [`fd`]() (optional). fd is a fast alternative to find and tap will use fd if it's installed on your machine. This is recommended if you want to search large directories.

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
