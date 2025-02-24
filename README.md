# tap

tap is an audio player for the terminal. Jump to any album in your library with fuzzy-finder shortcuts!

**Quick links:** [Options](#options), [Controls](#controls), [Configuration](#configuration), [Installation](#installation).

<img src="https://github.com/timdubbins/tap/blob/master/doc/tap_screenshot.png" width="650"/>


## Usage

```bash
> tap [options] [path]
```

- Run `tap` in a directory containing music folders to launch the fuzzy-finder. Simply select an album to begin playback.
- Press `Tab` at any time to return to the fuzzy-finder.

**Example:**

```bash
> cd ~/path/to/my_music
> tap
```

If no `path` is specified, `tap` defaults to the current working directory.

### Direct Playback

To skip the fuzzy-finder and directly open an audio file or album:

```bash
> tap ~/path/to/my_album
```


## Options
Option                  | Description
---                     |---
`-d` `--default`        | Run from the default directory, if set.
`-p` `--print`          | Print the path of the default directory, if set.
`-s` `--set`            | Set a default directory. Requires a `tap.yml` config file.
`-b` `--term-bg`        | Use the terminal background color.
`-t` `--term-color`     | Use the terminal background and foreground colors only.
`-c` `--default-color`  | Ignore any user-defined colors.
`--color <COLOR>`       | Set your own color scheme. See [Notes](#notes) for available names.
`--cli`                 | Play audio in CLI-mode (without the TUI).


## Controls

<details open>
<summary><b>Keyboard</b></summary>
<br>

Global              | Binding
---                 |---
previous album      | `-`
random album        | `=`
fuzzy search        | `Tab`
artist search       | `Ctrl` + `a`
artist search (a-z) | `A-Z`
album search        | `Ctrl` + `d`
depth search (1-4)  | `F1-F4`
parent search       | `` ` ``
open file manager   | `Ctrl` + `o`
quit                | `Ctrl` + `q`

**Note:** Search results are shuffled by default. Sort with `Ctrl` + `s`.



Player                  | Binding
---                     |---
play or pause           | `h` or <kbd>&larr;</kbd> or `Space`
next                    | `j` or `n` or <kbd>&darr;</kbd>
previous                | `k` or `p` or <kbd>&uarr;</kbd>
stop                    | `l` or <kbd>&rarr;</kbd> or `Enter`
randomize               | `*` or `r` (next track is random from library)
shuffle                 | `~` or `s` (current playlist order is shuffled)
seek << / >>            | `,` / `.`
seek to second          | `0-9`, `"`
seek to minute          | `0-9`, `'`
volume down / up        | `[` / `]`
toggle volume display   | `v`
toggle mute             | `m`
go to first track       | `gg`
go to last track        | `Ctrl` + `g`
go to track number      | `0-9`, `g`
show keybindings        | `?`
quit                    | `q`

Finder              | Binding
---                 |---
select              | `Ctrl` + `j` or `Enter`
next                | `Ctrl` + `n` or <kbd>&darr;</kbd>
previous            | `Ctrl` + `p` or <kbd>&uarr;</kbd>
sort results        | `Ctrl` + `s`
cursor right        | `Ctrl` + `f` or <kbd>&rarr;</kbd>
cursor left         | `Ctrl` + `b` or <kbd>&larr;</kbd>
cursor home         | `Home`
cursor end          | `End`
clear query         | `Ctrl` + `u`
cancel search       | `Esc`
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


## Configuration


tap doesn't create the config file for you, but it looks for one in the following locations:

- $XDG_CONFIG_HOME/tap/tap.yml
- $XDG_CONFIG_HOME/tap.yml
- $HOME/.config/tap/tap.yml
- $HOME/.tap.yml

A example config file can be found [here]("https://github.com/timdubbins/tap/blob/master/doc/tap.yml).

**Colors:**

Colors can be set in the [config](#configuration) file or using the ```--color``` command.

The following example will set a [Solarized](https://ethanschoonover.com/solarized/) theme:
```
--color fg=268bd2,bg=002b36,hl=fdf6e3,prompt=586e75,header_1=859900,header_2=cb4b16,progress=6c71c4,info=2aa198,err=dc322f
```



**Default Path:**

The default path can be set in the tap.yml [config](#configuration) file. This allows you to load the default directory with the `-d --default` command and also provides faster load times by caching.

When setting a default path tap will write a small amount of encoded data to `~/.cache/tap`. This is guaranteed to be at least as small as the in-memory data and will be updated everytime the default path is accessed. Using the `-s --set` command will update the `path` field in the `tap.yml` config file.

Without setting a default path tap is `read-only`.


## Installation
You will need an `ncurses` distribution (with development headers) to compile tap. Installation instructions for each supported platform are below:


<details>
<summary><b>macOS</b></summary>
<br>
You can install with <a href="https://brew.sh/">Homebrew</a>:

```bash
> brew install timdubbins/tap/tap
> tap --version
0.5.0
```

`ncurses` can be installed with:
```bash
> brew install ncurses
```


</details>


<details>
<summary><b>Arch Linux</b></summary>
<br>

~~You can install with an <a href="https://wiki.archlinux.org/title/AUR_helpers">AUR helper</a>,
such as <a href="https://github.com/Jguer/yay">yay</a>:~~

**The Arch package is not currently maintained. Please install with Rust.**

```bash
> yay -S tap
> tap --version
0.5.0
```

`ncurses` can be installed with:
```bash
> yay -S ncurses
```

The AUR package is available <a href="https://aur.archlinux.org/packages/tap">here</a>.~~
<br>
</details>


<details>
<summary><b>Debian</b> (or a Debian derivative, such as <b>Ubuntu</b>)</summary>
<br>

You can install with a binary <code>.deb</code> file provided in each <a href="https://github.com/timdubbins/tap/releases/tag/v0.5.0">tap release</a>:

```bash
> curl -LO https://github.com/timdubbins/tap/releases/download/v0.5.0/tap_0.5.0.deb
> sudo dpkg -i tap_0.5.0.deb
> tap --version
0.5.0
```

`ncurses` can be installed with:
```bash
> sudo apt install libncurses5-dev libncursesw5-dev
```

</details>

<details>
<summary><b>Rust</b></summary>
<br>

To compile from source, first you need a <a href="https://www.rust-lang.org/learn/get-started">Rust installation</a> (if you don't have one) and then you can use <a href="https://github.com/rust-lang/cargo">cargo</a>:

```bash
> git clone https://github.com/timdubbins/tap
> cd tap
> cargo install --path .
> tap --version
0.5.0
```

</details>

The binaries for each release are also available [here](https://github.com/timdubbins/tap/releases/tag/v0.5.0).


## Notes

**Supports:**
- Gapless playback.
- `aac`, `flac`, `mp3`, `m4a`, `ogg` and `wav`.

## Contributing

Suggestions / bug reports are welcome!

### Inspired by

- [cmus](https://github.com/cmus/cmus) - popular console music player with many features
- [fzf](https://github.com/junegunn/fzf) - command line fuzzy finder

### Made possible by

- [cursive](https://github.com/gyscos/cursive) - TUI library for Rust with great documentation
- [rodio](https://github.com/RustAudio/rodio) - audio playback library for Rust
