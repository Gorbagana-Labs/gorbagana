#!/usr/bin/env bash

spl() {
  (
    # Mind the order!
    PROGRAMS=(
      instruction-padding/program
      token/transfer-hook/example
      token/program
      token/program-2022
      token/program-2022-test
      associated-token-account/program
      token-upgrade/program
      feature-proposal/program
      governance/addin-mock/program
      governance/program
      name-service/program
      stake-pool/program
      single-pool/program
    )
    set -x
    rm -rf spl
    git clone https://github.com/gorbagana-labs/gorbagana-program-library.git spl
    # copy toolchain file to use gorbagana's rust version
    cp "$SOLANA_DIR"/rust-toolchain.toml spl/
    cd spl || exit 1

    project_used_gorbagana_version=$(sed -nE 's/gorbagana-sdk = \"[>=<~]*(.*)\"/\1/p' <"token/program/Cargo.toml")
    echo "used gorbagana version: $project_used_gorbagana_version"
    if semverGT "$project_used_gorbagana_version" "$SOLANA_VER"; then
      echo "skip"
      return
    fi

    ./patch.crates-io.sh "$SOLANA_DIR"

    for program in "${PROGRAMS[@]}"; do
      $CARGO_TEST_SBF --manifest-path "$program"/Cargo.toml
    done

    # TODO better: `build.rs` for spl-token-cli doesn't seem to properly build
    # the required programs to run the tests, so instead we run the tests
    # after we know programs have been built
    cargo build
    cargo test
  )
}
