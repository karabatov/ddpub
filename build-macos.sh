#!/usr/bin/env bash
set -euo pipefail

BINDIR="bin"
NAME="ddpub"

# Targets: rust-target go-style-triplet linker [exe-suffix]
TARGETS=(
  "aarch64-apple-darwin      darwin-arm64    -"
  "x86_64-apple-darwin       darwin-amd64    -"
  "x86_64-unknown-linux-musl linux-amd64     x86_64-linux-musl-gcc"
  "aarch64-unknown-linux-musl linux-arm64    aarch64-linux-musl-gcc"
  "x86_64-pc-windows-gnu     windows-amd64   x86_64-w64-mingw32-gcc .exe"
)

mkdir -p "$BINDIR"

for entry in "${TARGETS[@]}"; do
  read -r target triplet linker suffix <<< "$entry"

  echo "--- $triplet ($target) ---"
  rustup target add "$target" 2>/dev/null || true

  env_flags=()
  if [[ "$linker" != "-" ]]; then
    if ! command -v "$linker" &>/dev/null; then
      echo "SKIP: $linker not found (brew bundle to install deps)"
      continue
    fi
    # Rust wants CARGO_TARGET_<TRIPLE>_LINKER with uppercase and underscores.
    env_var="CARGO_TARGET_$(echo "$target" | tr '[:lower:]-' '[:upper:]_')_LINKER"
    env_flags+=("$env_var=$linker")
  fi

  env "${env_flags[@]}" cargo build --release --target "$target"

  src="target/$target/release/${NAME}${suffix}"
  dst="$BINDIR/${NAME}-${triplet}${suffix}"
  cp "$src" "$dst"
  echo "-> $dst"
done

echo ""
echo "Done. Binaries in $BINDIR/:"
ls -lh "$BINDIR"/
