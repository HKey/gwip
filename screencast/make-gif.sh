#!/bin/sh

set -e

if [ $# -ne 2 ]; then
  echo 'Usage: ./make-gif.sh <input-video> <output-gif>'
  exit 1
fi

input=$1
tmpdir=$(mktemp --directory --suffix=-gwip-make-gif)
palette=${tmpdir}/pallete.png
output=$2

ffmpeg -i ${input} -vf palettegen=max_colors=32 ${palette}
ffmpeg -i ${input} \
       -i ${palette} \
       -filter_complex 'paletteuse=diff_mode=1:dither=1' \
       ${output}

rm -r ${tmpdir}
