#!/usr/bin/env bash
# Empacota `kd` + `knudge-mcp` da plataforma atual em `dist/`, no mesmo padrão de nome
# do GitHub Release: `knudge-<versão>-<target>.{tar.gz,zip}`.
#
# Uso:
#   ./scripts/package.sh              # compila release (otimizado) e empacota
#   ./scripts/package.sh --no-build   # empacota o que já existe em target/release
#
# O `target` é o host do rustc (ex.: x86_64-unknown-linux-musl quando se compila com
# `--target`); para bater com o release, defina RUSTFLAGS/target como preferir.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

build=1
if [ "${1:-}" = "--no-build" ]; then
    build=0
fi

version="$(grep -m1 -E '^version = "[0-9]' Cargo.toml | cut -d'"' -f2)"
host="$(rustc -vV | sed -n 's/^host: //p')"

if [ "$build" -eq 1 ]; then
    cargo build --release --locked -p knudge-cli -p knudge-mcp
fi

name="knudge-${version}-${host}"
staging="dist/${name}"
mkdir -p "$staging"

case "$host" in
    *-pc-windows-*)
        cp target/release/kd.exe "$staging/kd.exe"
        cp target/release/knudge-mcp.exe "$staging/knudge-mcp.exe"
        if command -v 7z >/dev/null 2>&1; then
            (cd "$staging" && 7z a -tzip "../${name}.zip" ./* >/dev/null)
        elif command -v zip >/dev/null 2>&1; then
            (cd "$staging" && zip -q "../${name}.zip" ./*)
        else
            printf 'ERRO: preciso de 7z ou zip para empacotar no Windows\n' >&2
            exit 1
        fi
        printf 'OK: dist/%s.zip\n' "$name"
        ;;
    *)
        cp target/release/kd "$staging/kd"
        cp target/release/knudge-mcp "$staging/knudge-mcp"
        chmod +x "$staging/kd" "$staging/knudge-mcp"
        (cd "$staging" && tar -czf "../${name}.tar.gz" kd knudge-mcp)
        printf 'OK: dist/%s.tar.gz\n' "$name"
        ;;
esac
