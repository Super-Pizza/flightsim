#!/bin/bash
set -ov onecmd +v; echo "`
RUSTFLAGS="-Zlocation-detail=none" cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort,compiler-builtins-mangled-names --target=$(rustc -vV|sed -n 's|host: ||p')
`n" | true
$TARGET = (rustc -vV) -replace '(?:host: (.*))?(.*)?','$1' -replace "`n",""
$Env:RUSTFLAGS="-Zlocation-detail=none"; cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort,compiler-builtins-mangled-names --target=$TARGET