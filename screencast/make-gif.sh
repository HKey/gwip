#!/bin/sh

set -e

input=rec.mp4
palette=$(mktemp --suffix=-palette.png)
output=screencast.gif

ffmpeg -i ${input} -vf palettegen=max_colors=32 ${palette}
ffmpeg -i ${input} \
       -i ${palette} \
       -filter_complex 'paletteuse=diff_mode=1:dither=1' \
       ${output}

rm ${palette}
