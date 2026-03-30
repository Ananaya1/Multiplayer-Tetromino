#!/usr/bin/env zsh

setopt -eu

cd site/
python -m http.server "$@"
