#! /bin/sh

set -e

function call() {
  notify-send "$*"
  $*
  sleep 1
}

call gwip move --grid=2x1 --place=0,0 # top
call gwip move --grid=2x1 --place=1,0 # bottom
call gwip move --grid=1x2 --place=0,0 # left
call gwip move --grid=1x2 --place=0,1 # right
call gwip move --grid=2x2 --place=0,0 # top left
call gwip move --grid=2x2 --place=0,1 # top right
call gwip move --grid=2x2 --place=1,0 # bottom left
call gwip move --grid=2x2 --place=1,1 # bottom right
call gwip move --grid=1x1 --place=0,0 # center
