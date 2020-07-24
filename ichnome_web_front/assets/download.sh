#!/bin/sh
set -eu

PROGRAM="$(realpath "$0")"
PROGRAM_DIR="$(dirname "$PROGRAM")"

cd "$PROGRAM_DIR"
exec curl -LO https://raw.githubusercontent.com/highlightjs/highlight.js/master/src/styles/solarized-light.css
