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

Commands:
    move
        Move the focused window to the specified place.
        The display is divided into grid by "--grid" parameter.
        The window will be moved to the center of the cell in the grid
        specified by "--place" parameter.
        The line and the column of a cell is counted from top left
        and zero based.

        Example:
            $ gwip move --grid=2x3 --place=0,1

                            ncolumns = 3
                      |-----------------------|

                   -  +-------+-------+-------+
                   |  |  0,0  |  0,1  |  0,2  |
                   |  |       | here! |       |
        nlines = 2 |  |-------+-------+-------|
                   |  |  1,0  |  1,1  |  1,2  |
                   |  |       |       |       |
                   -  +-------+-------+-------+
```

## Dependency

This command depends on following commands:
- [xdotool](https://www.semicomplete.com/projects/xdotool/)
- [xwininfo](https://gitlab.freedesktop.org/xorg/app/xwininfo)

## TODO

- Support multiple display
