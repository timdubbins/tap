# tap

tap is an audio player for the terminal. Jump to any album in your library with fuzzy-finder shortcuts!

**Quick links:** [Bindings](#bindings), [Installation](#installation).

<img src="https://github.com/timdubbins/tap/blob/master/doc/tap_screenshot.png" width="650"/>

## How to use
```bash
> tap [options] [path]
```
Run `tap` in a directory that contains music folders to open a `fuzzy-finder`, allowing you to select an album to play. Playback starts on selection and you can return to the fuzzy-finder by pressing `Tab`:
```bash
> cd ~/path/to/my_music
> tap
``` 


To open a player without the fuzzy-finder provide a `path` to an audio file or album:
```bash
> tap ~/path/to/my_album
```

`path` can be a file or directory. If it is omitted the current directory is used.

Option                  | Description
---                     |---
`-a` `--automate`       | Run an automated player without the TUI. Quit with `Enter`.
`-d` `--default`        | Run from the default directory, if set.
`-p` `--print`          | Print the path of the default directory, if set.
`-s` `--set-default`    | Set `path` as the default directory. This can significantly reduce the time it takes to load this directory. See [Notes](#notes).
`-e` `--exclude`        | Exclude all directories that don't contain audio files. 
`-b` `--term-bg`        | Use the terminal background color.
`-c` `--term-color`     | Use the terminal background and foreground colors only.
`--color <COLOR>`       | Set colors using \<NAME>=\<HEX>. See [Notes](#notes) for available names.


## Bindings

<details open>
<summary><b>Keyboard</b></summary>
<br>

Global              | Keybinding    | Includes
---                 |---            |---
fuzzy search        | `Tab`         | <i>all folders</i>
depth search        | `F1...F4`     | <i>folders at depth 1...4</i>
filtered search     | `A...Z`       | <i>artists beginning with A...Z</i>
artist search       | `Ctrl` + `a`  | <i>all artists, sorted alphabetically</i>
album search        | `Ctrl` + `s`  | <i>all albums, sorted alphabetically</i>
parent search       | `Ctrl` + `p`  | <i>folders up one level</i>
previous album      | `-`           |
random album        | `=`           |
open file manager   | `Ctrl` + `o`  | See [Notes](#notes).

Player              | Keybinding
---                 |---
play or pause       | `h` or <kbd>&larr;</kbd> or `Space`
next                | `j` or <kbd>&darr;</kbd>
previous            | `k` or <kbd>&uarr;</kbd>
stop                | `l` or <kbd>&rarr;</kbd> or `Enter`
step forward        | `}`
step backward       | `{`
seek to sec         | `0...9` + `"`
seek to min         | `0...9` + `'`
random              | `r`
volume up           | `]`
volume down         | `[`
show volume         | `v`
mute                | `m`
go to first track   | `gg`
go to last track    | `Ctrl` + `g`
go to track number  | `0...9` + `g`
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

Player              | Keybinding             | Where
---                 |---                     |---
play or pause       | `Left Button`          | <i>Outside playlist</i>
select track        | `Left Button`          | <i>Inside playlist</i>
seek                | `Left Button Hold`     | <i>Inside progress bar<i>
volume              | `Scroll`               | <i>Outside playlist</i>
next / previous     | `Scroll`               | <i>Inside playlist</i>
stop                | `Right Button`         | <i>Anywhere</i>

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
0.4.11
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
0.4.11
```
The AUR package is available <a href="https://aur.archlinux.org/packages/tap">here</a>.
<br>
</details>


<details>
<summary><b>Debian</b> (or a Debian derivative, such as <b>Ubuntu</b>)</summary>
<br>

You can install with a binary <code>.deb</code> file provided in each <a href="https://github.com/timdubbins/tap/releases/tag/v0.4.11">tap release</a>:

```bash
> curl -LO https://github.com/timdubbins/tap/releases/download/v0.4.11/tap_0.4.11.deb
> sudo dpkg -i tap_0.4.11.deb
> tap --version
0.4.11
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
0.4.11
```

</details>

The binaries for each release are also available [here](https://github.com/timdubbins/tap/releases/tag/v0.4.11).

## Notes

**Supports:**
- Gapless playback.
- `aac`, `flac`, `mp3`, `m4a`, `ogg` and `wav`.


**Setting colors:**

The following `--color` example will set a [Solarized](https://ethanschoonover.com/solarized/) theme:
```
--color fg=268bd2,bg=002b36,hl=fdf6e3,prompt=586e75,header=859900,header+=cb4b16,progress=6c71c4,info=2aa198,err=dc322f 
```

**Setting an alias:**

It can be useful to create an `alias` if you set a default directory or want to persist your color scheme. Put something like the following in your shell config (for `zsh` users this would be your `.zshrc`):

```bash
alias tap="tap -db --color fg=268bd2,bg=002b36,hl=fdf6e3,prompt=586e75,header=859900,header+=cb4b16,progress=6c71c4,info=2aa198,err=dc322f"
```

Running `tap` from any directory will now load the cached default path and set the colors to those defined in the alias (as well as setting the background color to use the terminal background). We can still use commands like `tap .` and `tap <PATH> --color fg=ff9999` with this alias. 

**Setting the default directory:**

This will write a small amount of encoded data to `~/.cache/tap`. This is the only place that `tap` will write to and the data is guaranteed to be at least as small as the in-memory data. Changes in the default directory will be updated in ~/.cache/tap the next time it is accessed by tap.

As a benchmark, setting a directory that is 200GB as the default produces a ~/.cache/tap  that has size 350KB (equivalent to an mp3 that is 2 seconds long) and decreases the load time by ~6x.

**Opening your file manager:**

You can open your preferred file manager from within tap with `Ctrl` + `o` Requires `xdg-open` on linux. From the fuzzy-finder this opens the currently selected directory. From the player it opens the parent of the loaded audio file. 

## Contributing

Suggestions / bug reports are welcome!

### Inspired by

- [cmus](https://github.com/cmus/cmus) - popular console music player with many features
- [fzf](https://github.com/junegunn/fzf) - command line fuzzy finder

### Made possible by

- [cursive](https://github.com/gyscos/cursive) - TUI library for Rust with great documentation
- [rodio](https://github.com/RustAudio/rodio) - audio playback library for Rust
