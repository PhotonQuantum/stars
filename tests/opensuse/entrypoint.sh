#!/bin/bash
set -euo pipefail
IFS=$'\n\t'

cargo test
cargo run -- --dry-run
