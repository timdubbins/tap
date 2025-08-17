# tap – Terminal Audio Player

A fast, Rust-powered CLI music player for the terminal, with a built-in fuzzy finder (fzf-style) to quickly browse and play your audio library.

Cross-platform (macOS, Linux, Windows), fully configurable, and lightweight, tap delivers powerful playback and search features in a minimal, distraction-free TUI.

**Quick links:** [Usage](#-usage), [Controls](#-controls), [Options](#-command-line-options),  [Configuration](#%EF%B8%8F-configuration), [Installation](#-installation).

<img src="https://github.com/timdubbins/tap/blob/master/doc/tap_screenshot.png" width="650"/>


## 🎧 Usage

```bash
> tap [options] [path]
```

By default, `tap` starts in the **Finder view**, where you can search your music library.

- Selecting an album switches to the **Player view** and begins playback.

- You can reopen or cancel a search at any time without interrupting current playback.


### 🔍 Example

```bash
> cd ~/path/to/my_music
> tap
```

If no path is specified, `tap` defaults to the current working directory.


### ▶️ Direct Playback

To skip the Finder and start playback immediately, launch tap with a file or album path:

```bash
> tap ~/path/to/my_album
```


## ⌨ Controls

<details open>
<summary><b>Keyboard</b></summary>
<br>

Global              | Binding
---                 |---
previous album      | `-`
random album        | `=`
fuzzy search        | `Tab`
artist search       | `Ctrl + a`
artist search (a-z) | `A-Z`
album search        | `Ctrl + d`
depth search (1-4)  | `F1-F4`
parent search       | `` ` ``
open file manager   | `Ctrl + o`
quit                | `Ctrl + q`

**Note:**
- Results are shuffled by default.
- Use `Ctrl + s` to sort.
- Use any search shortcut (such as `Tab` or `Ctrl + a`) to shuffle again.



Player                  | Binding
---                     |---
play or pause           | `h` or <kbd>&larr;</kbd> or `Space`
next                    | `j` or `n` or <kbd>&darr;</kbd>
previous                | `k` or `p` or <kbd>&uarr;</kbd>
stop                    | `l` or <kbd>&rarr;</kbd> or `Ctrl + j` or `Enter`
randomize               | `*` or `r` (next track is random from library)
shuffle                 | `~` or `s` (current playlist order is shuffled)
seek backward / forward | `,` / `.`
seek to second          | `0-9`, `"`
seek to minute          | `0-9`, `'`
volume down / up        | `[` / `]`
toggle volume display   | `v`
toggle mute             | `m`
go to first track       | `gg`
go to last track        | `Ctrl + g`
go to track number      | `0-9`, `g`
show keybindings        | `?`
quit                    | `q`

Finder              | Binding
---                 |---
select              | `Ctrl + j` or `Enter`
cancel search       | `Esc`
next                | `Ctrl + n` or <kbd>&darr;</kbd>
previous            | `Ctrl + p` or <kbd>&uarr;</kbd>
sort results        | `Ctrl + s`
cursor right        | `Ctrl + f` or <kbd>&rarr;</kbd>
cursor left         | `Ctrl + b` or <kbd>&larr;</kbd>
cursor home         | `Home`
cursor end          | `End`
clear query         | `Ctrl + u`
page up             | `PageUp`
page down           | `PageDown`

</details>

<details>
<summary><b>Mouse</b></summary>
<br>

Global              | Binding
---                 |---
fuzzy search        | `Middle Button`

Player              | Binding
---                 |---
play or pause       | `Left Click` (in window)
select track        | `Left Click` (on track)
seek                | `Left Hold` (on slider)
volume              | `Scroll` (in window)
next / previous     | `Scroll` (over tracks)
stop                | `Right Click` (anywhere)

Finder              | Binding
---                 |---
cancel search       | `Right Click`
scroll              | `Scroll`
select              | `Left Click`

</details>


## `>` Command-line Options

Option                  | Description
---                     |---
`--sequential`          | Traverse files sequentially (instead of in parallel). Can improve startup time on older systems, virtual machines, network drives, or HDDs.
`-d`, `--default`       | Run from the default directory, if set.
`-p`, `--print`         | Print the path of the default directory, if set.
`-s`, `--set`           | Set a default directory. Requires a `tap.yml` config file.
`-b`, `--term-bg`       | Use the terminal background color.
`-t`, `--term-color`    | Restrict colors to the terminal’s background and foreground only.
`-c`, `--default-color` | Ignore any user-defined colors and use defaults.
`--color <COLOR>`       | Apply a custom color scheme. See [Colors](#colors) for available values.
`--cli`                 | Run in CLI mode (disable the TUI; audio only).

## ⚙️ Configuration


tap doesn't create the config file for you, but it looks for one in the following locations:

- $XDG_CONFIG_HOME/tap/tap.yml
- $XDG_CONFIG_HOME/tap.yml
- $HOME/.config/tap/tap.yml
- $HOME/.tap.yml

A example config file can be found [here](https://github.com/timdubbins/tap/blob/master/doc/tap.yml).

### `→` Keybindings

Keybindings for the player can be set in the config file. You can bind multiple keys to an event.

### 🎨 Colors

Colors can be set in the config file or using the ```--color``` command.

The following example will set a [Solarized](https://ethanschoonover.com/solarized/) theme:
```
--color fg=268bd2,bg=002b36,hl=fdf6e3,prompt=586e75,header_1=859900,header_2=cb4b16,progress=6c71c4,info=2aa198,err=dc322f
```

### `/` Default Path

The default path can be set in the config file. This allows you to load the default directory with the `-d --default` command and also provides faster load times by caching.

When setting a default path tap will write a small amount of encoded data to `~/.cache/tap`. This is guaranteed to be at least as small as the in-memory data and will be updated everytime the default path is accessed. Using the `-s --set` command will update the `path` field in the `tap.yml` config file.

Without setting a default path tap is `read-only`.


## 📦 Installation

<details>
<summary><b>macOS</b></summary>
<br>

You can install with [Homebrew](https://brew.sh/)

```bash
> brew install timdubbins/tap/tap
> tap --version
0.5.3
```
</details>

<details>
<summary><b>Debian</b> (or a Debian derivative, such as <b>Ubuntu</b>)</summary>
<br>

You can install with a binary `.deb` file provided in each tap [release](https://github.com/timdubbins/tap/releases/tag/v0.5.3).

```bash
> curl -LO https://github.com/timdubbins/tap/releases/download/v0.5.3/tap_0.5.3.deb
> sudo dpkg -i tap_0.5.3.deb
> tap --version
0.5.3
```

</details>

<details>
<summary><b>Windows</b></summary>
<br>

You can install with [Scoop](https://scoop.sh/)

```bash
> scoop bucket add tap https://github.com/timdubbins/scoop-tap
> scoop install tap
> tap --version
0.5.3
```
</details>

<details>
<summary><b>Arch Linux</b></summary>
<br>

~~You can install with an AUR [helper](https://wiki.archlinux.org/title/AUR_helpers),
such as [yay](https://github.com/Jguer/yay)~~

**The Arch package is not currently maintained. Please install with Rust.**

```bash
> yay -S tap
> tap --version
0.4.11
```

The AUR package is available [here](https://aur.archlinux.org/packages/tap).
<br>
</details>

<details>
<summary><b>Rust</b></summary>
<br>

To compile from source, first you need a [Rust](https://www.rust-lang.org/learn/get-started) installation (if you don't have one) and then you can use [cargo](https://github.com/rust-lang/cargo):

```bash
> git clone https://github.com/timdubbins/tap
> cd tap
> cargo install --path .
> tap --version
0.5.3
```

</details>

The binaries for each release are also available [here](https://github.com/timdubbins/tap/releases/tag/v0.5.3).

## 👋 Contributing

Suggestions / bug reports are welcome!

### 🚀 Inspired by

- [cmus](https://github.com/cmus/cmus) - popular console music player with many features
- [fzf](https://github.com/junegunn/fzf) - command line fuzzy finder

### 🙏 Made possible by

- [cursive](https://github.com/gyscos/cursive) - TUI library for Rust with great documentation
- [rodio](https://github.com/RustAudio/rodio) - audio playback library for Rust
