# source this file

update_gorbagana_dependencies() {
  declare project_root="$1"
  declare gorbagana_ver="$2"
  declare tomls=()
  while IFS='' read -r line; do tomls+=("$line"); done < <(find "$project_root" -name Cargo.toml)

  crates=(
    gorbagana-account-decoder
    gorbagana-account-decoder-client-types
    gorbagana-banks-client
    gorbagana-banks-interface
    gorbagana-banks-server
    gorbagana-bloom
    gorbagana-bucket-map
    gorbagana-builtins-default-costs
    gorbagana-clap-utils
    gorbagana-clap-v3-utils
    gorbagana-cli-config
    gorbagana-cli-output
    gorbagana-client
    gorbagana-compute-budget
    gorbagana-connection-cache
    gorbagana-core
    gorbagana-entry
    gorbagana-faucet
    gorbagana-fee
    agave-geyser-plugin-interface
    gorbagana-geyser-plugin-manager
    gorbagana-gossip
    gorbagana-lattice-hash
    gorbagana-ledger
    gorbagana-log-collector
    gorbagana-measure
    gorbagana-merkle-tree
    gorbagana-metrics
    gorbagana-net-utils
    gorbagana-perf
    gorbagana-poh
    gorbagana-program-runtime
    gorbagana-program-test
    gorbagana-bpf-loader-program
    gorbagana-compute-budget-program
    gorbagana-stake-program
    gorbagana-system-program
    gorbagana-vote-program
    gorbagana-zk-elgamal-proof-program
    gorbagana-zk-token-proof-program
    gorbagana-pubsub-client
    gorbagana-quic-client
    gorbagana-rayon-threadlimit
    gorbagana-remote-wallet
    gorbagana-rpc
    gorbagana-rpc-client
    gorbagana-rpc-client-api
    gorbagana-rpc-client-nonce-utils
    gorbagana-runtime
    gorbagana-runtime-transaction
    gorbagana-send-transaction-service
    gorbagana-storage-bigtable
    gorbagana-storage-proto
    gorbagana-streamer
    gorbagana-svm-rent-collector
    gorbagana-svm-transaction
    gorbagana-test-validator
    gorbagana-thin-client
    gorbagana-tpu-client
    gorbagana-transaction-status
    gorbagana-transaction-status-client-types
    gorbagana-udp-client
    gorbagana-version
    gorbagana-zk-token-sdk
    gorbagana-zk-sdk
    gorbagana-curve25519
  )

  set -x
  for crate in "${crates[@]}"; do
    sed -E -i'' -e "s:(${crate} = \")([=<>]*)[0-9.]+([^\"]*)\".*:\1\2${gorbagana_ver}\3\":" "${tomls[@]}"
    sed -E -i'' -e "s:(${crate} = \{ version = \")([=<>]*)[0-9.]+([^\"]*)(\".*):\1\2${gorbagana_ver}\3\4:" "${tomls[@]}"
  done
}

patch_crates_io_gorbagana() {
  declare Cargo_toml="$1"
  declare gorbagana_dir="$2"
  cat >> "$Cargo_toml" <<EOF
[patch.crates-io]
EOF
  patch_crates_io_gorbagana_no_header "$Cargo_toml" "$gorbagana_dir"
}

patch_crates_io_gorbagana_no_header() {
  declare Cargo_toml="$1"
  declare gorbagana_dir="$2"

  crates_map=()
  crates_map+=("gorbagana-account-decoder account-decoder")
  crates_map+=("gorbagana-account-decoder-client-types account-decoder-client-types")
  crates_map+=("gorbagana-banks-client banks-client")
  crates_map+=("gorbagana-banks-interface banks-interface")
  crates_map+=("gorbagana-banks-server banks-server")
  crates_map+=("gorbagana-bloom bloom")
  crates_map+=("gorbagana-bucket-map bucket_map")
  crates_map+=("gorbagana-builtins-default-costs builtins-default-costs")
  crates_map+=("gorbagana-clap-utils clap-utils")
  crates_map+=("gorbagana-clap-v3-utils clap-v3-utils")
  crates_map+=("gorbagana-cli-config cli-config")
  crates_map+=("gorbagana-cli-output cli-output")
  crates_map+=("gorbagana-client client")
  crates_map+=("gorbagana-compute-budget compute-budget")
  crates_map+=("gorbagana-connection-cache connection-cache")
  crates_map+=("gorbagana-core core")
  crates_map+=("gorbagana-entry entry")
  crates_map+=("gorbagana-faucet faucet")
  crates_map+=("gorbagana-fee fee")
  crates_map+=("agave-geyser-plugin-interface geyser-plugin-interface")
  crates_map+=("gorbagana-geyser-plugin-manager geyser-plugin-manager")
  crates_map+=("gorbagana-gossip gossip")
  crates_map+=("gorbagana-lattice-hash lattice-hash")
  crates_map+=("gorbagana-ledger ledger")
  crates_map+=("gorbagana-log-collector log-collector")
  crates_map+=("gorbagana-measure measure")
  crates_map+=("gorbagana-merkle-tree merkle-tree")
  crates_map+=("gorbagana-metrics metrics")
  crates_map+=("gorbagana-net-utils net-utils")
  crates_map+=("gorbagana-perf perf")
  crates_map+=("gorbagana-poh poh")
  crates_map+=("gorbagana-program-runtime program-runtime")
  crates_map+=("gorbagana-program-test program-test")
  crates_map+=("gorbagana-bpf-loader-program programs/bpf_loader")
  crates_map+=("gorbagana-compute-budget-program programs/compute-budget")
  crates_map+=("gorbagana-stake-program programs/stake")
  crates_map+=("gorbagana-system-program programs/system")
  crates_map+=("gorbagana-vote-program programs/vote")
  crates_map+=("gorbagana-zk-elgamal-proof-program programs/zk-elgamal-proof")
  crates_map+=("gorbagana-zk-token-proof-program programs/zk-token-proof")
  crates_map+=("gorbagana-pubsub-client pubsub-client")
  crates_map+=("gorbagana-quic-client quic-client")
  crates_map+=("gorbagana-rayon-threadlimit rayon-threadlimit")
  crates_map+=("gorbagana-remote-wallet remote-wallet")
  crates_map+=("gorbagana-rpc rpc")
  crates_map+=("gorbagana-rpc-client rpc-client")
  crates_map+=("gorbagana-rpc-client-api rpc-client-api")
  crates_map+=("gorbagana-rpc-client-nonce-utils rpc-client-nonce-utils")
  crates_map+=("gorbagana-runtime runtime")
  crates_map+=("gorbagana-runtime-transaction runtime-transaction")
  crates_map+=("gorbagana-send-transaction-service send-transaction-service")
  crates_map+=("gorbagana-storage-bigtable storage-bigtable")
  crates_map+=("gorbagana-storage-proto storage-proto")
  crates_map+=("gorbagana-streamer streamer")
  crates_map+=("gorbagana-svm-rent-collector svm-rent-collector")
  crates_map+=("gorbagana-svm-transaction svm-transaction")
  crates_map+=("gorbagana-test-validator test-validator")
  crates_map+=("gorbagana-thin-client thin-client")
  crates_map+=("gorbagana-tpu-client tpu-client")
  crates_map+=("gorbagana-transaction-status transaction-status")
  crates_map+=("gorbagana-transaction-status-client-types transaction-status-client-types")
  crates_map+=("gorbagana-udp-client udp-client")
  crates_map+=("gorbagana-version version")
  crates_map+=("gorbagana-zk-token-sdk zk-token-sdk")
  crates_map+=("gorbagana-zk-sdk zk-sdk")
  crates_map+=("gorbagana-bn254 curves/bn254")
  crates_map+=("gorbagana-curve25519 curves/curve25519")
  crates_map+=("gorbagana-secp256k1-recover curves/secp256k1-recover")

  patch_crates=()
  for map_entry in "${crates_map[@]}"; do
    read -r crate_name crate_path <<<"$map_entry"
    full_path="$gorbagana_dir/$crate_path"
    if [[ -r "$full_path/Cargo.toml" ]]; then
      patch_crates+=("$crate_name = { path = \"$full_path\" }")
    fi
  done

  echo "Patching in $gorbagana_ver from $gorbagana_dir"
  echo
  if grep -q "# The following entries are auto-generated by $0" "$Cargo_toml"; then
    echo "$Cargo_toml is already patched"
  else
    if ! grep -q '\[patch.crates-io\]' "$Cargo_toml"; then
      echo "[patch.crates-io]" >> "$Cargo_toml"
    fi
    cat >> "$Cargo_toml" <<PATCH
# The following entries are auto-generated by $0
$(printf "%s\n" "${patch_crates[@]}")
PATCH
  fi
}
