# tap

tap is an audio player for the terminal. It gives you access to the albums in your library through fuzzy-finding shortcuts.

<img src="https://github.com/timdubbins/tap/blob/master/doc/tap_screenshot.png" width="650"/>

## How to use

```bash
> tap [path]
```
Run tap in a directory that contains music folders to open a fuzzy-finder, allowing you to select an album to play. Playback starts on selection and you can return to the fuzzy-finder by pressing `Tab`.

Provide a path to an audio file or album to open and play the file(s) without the fuzzy-finder.

**NB:** If path is omitted, the current directory is used. If a second path is provided it will override the first.

## Bindings

<details open>
<summary><b>Keyboard</b></summary>
<br>

Global              | Keybinding    | Includes
---                 |---            |---
fuzzy search        | `Tab`         | <i>all folders</i>
depth search        | `F1...F4`     | <i>folders at depth 1...4</i>
filtered search     | `A...Z`       | <i>artists beginning with A...Z</i>
artist search       | `Ctrl` + `s`  | <i>all artists, sorted alphabetically</i>
album search        | `Ctrl` + `a`  | <i>all albums, sorted alphabetically</i>
load previous album | `-`           |
load random album   | `=`           |

Player              | Keybinding
---                 |---
play or pause       | `h` or <kbd>&larr;</kbd> or `Space`
next                | `j` or <kbd>&darr;</kbd>
previous            | `k` or <kbd>&uarr;</kbd>
stop                | `l` or <kbd>&rarr;</kbd> or `Enter`
go to first track   | `gg`
go to last track    | `Ctrl` + `g`
go to track number  | `0...9` + `g`
toggle mute         | `m`
help                | `?`
quit                | `q`

Fuzzy               | Keybinding
---                 |---
clear search        | `Ctrl` + `u`
cancel search       | `Esc`
page up             | `Ctrl` + `h` or `PgUp`
page down           | `Ctrl` + `l` or `PgDn`
random page         | `Ctrl` + `z`

</details>

<details>
<summary><b>Mouse</b></summary>
<br>

Global              | Keybinding
---                 |---
fuzzy search        | `Middle Button`

Player              | Keybinding
---                 |---
play or pause       | `Left Button`
next / previous     | `Scroll`
stop                | `Right Button`
select              | `Left Button`

Fuzzy               | Keybinding
---                 |---
cancel search       | `Right Button`
scroll              | `Scroll`
select              | `Left Button`

</details>

## Installation

<details>
<summary><b>macOS</b></summary>
<br>
You can install with <a href="https://brew.sh/">Homebrew</a>:

```bash
> brew install timdubbins/tap/tap
> tap --version
0.4.1
```

</details>


<details>
<summary><b>Arch Linux</b></summary>
<br>

You can install with an <a href="https://wiki.archlinux.org/title/AUR_helpers">AUR helper</a>,
such as <a href="https://github.com/Jguer/yay">yay</a>:

```bash
> yay -S tap
> tap --version
0.4.1
```
The AUR package is available <a href="https://aur.archlinux.org/packages/tap">here</a>.
<br>
</details>


<details>
<summary><b>Debian</b> (or a Debian derivative, such as <b>Ubuntu</b>)</summary>
<br>

You can install with a binary <code>.deb</code> file provided in each <a href="https://github.com/timdubbins/tap/releases/tag/v0.4.1">tap release</a>:

```bash
> curl -LO https://github.com/timdubbins/tap/releases/download/v0.4.1/tap_v0.4.1_amd64.deb
> sudo dpkg -i tap_v0.4.1_amd64.deb
> tap --version
0.4.1
```

</details>

<details>
<summary><b>Rust</b></summary>
<br>

To compile from source, first you need a <a href="https://www.rust-lang.org/">Rust installation</a> (if you don't have one) and then you can use <a href="https://github.com/rust-lang/cargo">cargo</a>:

```bash
> git clone https://github.com/timdubbins/tap
> cd tap
> cargo install --path .
> tap --version
0.4.1
```

</details>

The binaries for each release are also available [here](https://github.com/timdubbins/tap/releases/tag/v0.4.1).

## Notes

- The supported formats are: `aac`, `flac`, `mp3`, `m4a`, `ogg` and `wav`.
- If there is an issue with playback of a file it is possible that this is due to incorrect audio tags on the file.

## Building

For Rust programmers, tap can be built in the usual manner with:
```bash
> cargo build [options]
```

### Inspired by

- [cmus](https://github.com/cmus/cmus) - popular console music player with many features
- [fzf](https://github.com/junegunn/fzf) - command line fuzzy finder

### Made possible by

- [cursive](https://github.com/gyscos/cursive) - TUI library for Rust with great documentation
- [rodio](https://github.com/RustAudio/rodio) - audio playback library for Rust
