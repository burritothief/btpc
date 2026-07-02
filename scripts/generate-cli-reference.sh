#!/bin/sh
set -eu

output=${1:-docs}

if [ "${BTPC_BIN+x}" = x ]; then
    binary=$BTPC_BIN
elif [ -z "${BTPC_BIN:-}" ]; then
    cargo build -p btpc-cli
    binary=target/debug/btpc
fi
if [ ! -x "$binary" ]; then
    echo "BTPC binary is not executable: $binary" >&2
    exit 1
fi
mkdir -p "$output/reference" "$output/completions"
"$binary" __generate-markdown "$output/cli/reference"
"$binary" --help > "$output/reference/btpc.txt"
for command in create inspect validate verify edit magnet completion; do
    "$binary" "$command" --help > "$output/reference/btpc-$command.txt"
done
for command in generate install uninstall; do
    "$binary" completion "$command" --help > "$output/reference/btpc-completion-$command.txt"
done
"$binary" completions --help > "$output/reference/btpc-completions.txt"
"$binary" manpage --help > "$output/reference/btpc-manpage.txt"
for command in path init show check explain tracker preset; do
    "$binary" config "$command" --help > "$output/reference/btpc-config-$command.txt"
done
"$binary" config --help > "$output/reference/btpc-config.txt"
"$binary" config explain create --help > "$output/reference/btpc-config-explain-create.txt"
for command in list add remove; do
    "$binary" config tracker "$command" --help > "$output/reference/btpc-config-tracker-$command.txt"
done
for command in list show save remove; do
    "$binary" config preset "$command" --help > "$output/reference/btpc-config-preset-$command.txt"
done
"$binary" manpage > "$output/reference/btpc.1"
for shell in bash zsh fish powershell elvish; do
    "$binary" completion generate "$shell" > "$output/completions/btpc.$shell"
done
