# PNGme

Hide secret messages inside your PNGs.

This project is from [this excellent
guide](https://picklenerd.github.io/pngme_book/).

## Install

Locally:

    cargo install --path .

Remotely:

    cargo install --git https://github.com/gabebw/pngme

## Running

Add a secret message to a PNG in a "RuST" chunk:

    pngme encode ./something.png RuST "Secret message here"

Add a secret message without overwriting the original file:

    pngme encode ./input.png RuST "Secret message here" ./output.png

Show your secret message:

    pngme decode ./something.png RuST

Remove the secret message:

    pngme remove ./something.png RuST

Print out every chunk in a PNG:

    pngme print ./something.png
