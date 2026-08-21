#!/bin/bash

set -euo pipefail

git submodule update --init --recursive
cd wireguard-go

version="$(git describe --tags --always --dirty 2>/dev/null || git rev-parse --short HEAD)"
printf 'package main\n\nconst Version = "%s"\n' "$version" > version.go

make libwg
mv libwg.* ../
