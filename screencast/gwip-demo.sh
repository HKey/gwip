#! /bin/sh

set -e

function demo() {
  notify-send "$*"
  $*
  sleep 1
}

# move only
demo gwip move --grid=2x1 --place=0,0 # top
demo gwip move --grid=2x1 --place=1,0 # bottom
demo gwip move --grid=1x2 --place=0,0 # left
demo gwip move --grid=1x2 --place=0,1 # right
demo gwip move --grid=2x2 --place=0,0 # top left
demo gwip move --grid=2x2 --place=0,1 # top right
demo gwip move --grid=2x2 --place=1,0 # bottom left
demo gwip move --grid=2x2 --place=1,1 # bottom right
demo gwip move --grid=1x1 --place=0,0 # center

# move and fill
demo gwip move --grid=1x2 --place=0,0 --fill --gap=2%x2% # left
demo gwip move --grid=1x2 --place=0,1 --fill --gap=2%x2% # right
demo gwip move --grid=1x1 --place=0,0 --fill --gap=2%x2% # center wide
demo gwip move --grid=1x1 --place=0,0 --fill --gap=26%x2% # center tall
