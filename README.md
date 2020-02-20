# gwip

Gridded Window Placer

A wrapper command to place the focused window to the center of
the specified cell of gridded display.

## Demo

TODO

## Installation

```
$ cargo install --git 'https://github.com/HKey/gwip.git'
```

## Usage

```
Gridded Window Placer

Usage:
    gwip move --grid=<NLINESxNCOLUMNS> --place=<LINE,COLUMN>
              [--xdotool=<cmd>] [--xwininfo=<cmd>]
    gwip -h | --help

Options:
    -h, --help                Show this screen.
    --grid=<NLINESxNCOLUMNS>  How to divide screen.
                              Example: "--grid=2x1"
    --place=<LINE,COLUMN>     Where move the focused window to.
                              Example: "--place=0,0"
    --xdotool=<cmd>           Command of xdotool. [default: xdotool]
    --xwininfo=<cmd>          Command of xwininfo. [default: xwininfo]
```

## Dependency

This command depends on following commands:
- [xdotool](https://www.semicomplete.com/projects/xdotool/)
- [xwininfo](https://gitlab.freedesktop.org/xorg/app/xwininfo)

## TODO

- Support multiple display
