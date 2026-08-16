#!/usr/bin/env bash
set -euo pipefail
dir="$(cd "$(dirname "$0")" && pwd)"
cd "$dir"
if [ ! -f target/release/present ]; then
  cargo build --release 2>&1 | tail -1
fi
PRESENT_AUTO_PICK=1 target/release/present --ask "pick one" --options '["a","b"]' --json
