$ErrorActionPreference = "Stop"

git submodule update --init --recursive
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Set-Location wireguard-go

$version = git describe --tags --always --dirty
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
"package main`n`nconst Version = `"$version`"" | Set-Content -Path version.go

make libwg
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Move-Item -Path "libwg.*" -Destination ".."
