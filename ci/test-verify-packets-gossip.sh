#!/usr/bin/env bash

set -e
here=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)

if ! git lfs --version &>/dev/null; then
  echo "Git LFS is not installed. Please install Git LFS to proceed."
  exit 1
fi

rm -rf "$here"/gorbagana-packets
git clone https://github.com/anza-xyz/gorbagana-packets.git "$here"/gorbagana-packets
GOSSIP_WIRE_FORMAT_PACKETS="$here/gorbagana-packets/GOSSIP_PACKETS" cargo test --package gorbagana-gossip -- wire_format_tests::tests::test_gossip_wire_format --exact --show-output
