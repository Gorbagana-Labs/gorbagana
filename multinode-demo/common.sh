# |source| this file
#
# Common utilities shared by other scripts in this directory
#
# The following directive disable complaints about unused variables in this
# file:
# shellcheck disable=2034
#

# shellcheck source=net/common.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. || exit 1; pwd)"/net/common.sh

prebuild=
if [[ $1 = "--prebuild" ]]; then
  prebuild=true
fi

if [[ $(uname) != Linux ]]; then
  # Protect against unsupported configurations to prevent non-obvious errors
  # later. Arguably these should be fatal errors but for now prefer tolerance.
  if [[ -n $SOLANA_CUDA ]]; then
    echo "Warning: CUDA is not supported on $(uname)"
    SOLANA_CUDA=
  fi
fi

if [[ -n $USE_INSTALL || ! -f "$SOLANA_ROOT"/Cargo.toml ]]; then
  gorbagana_program() {
    declare program="$1"
    if [[ -z $program ]]; then
      printf "gorbagana"
    else
      if [[ $program == "validator" || $program == "ledger-tool" || $program == "watchtower" || $program == "install" ]]; then
        printf "agave-%s" "$program"
      else
        printf "gorbagana-%s" "$program"
      fi
    fi
  }
else
  gorbagana_program() {
    declare program="$1"
    declare crate="$program"
    if [[ -z $program ]]; then
      crate="cli"
      program="gorbagana"
    elif [[ $program == "validator" || $program == "ledger-tool" || $program == "watchtower" || $program == "install" ]]; then
      program="agave-$program"
    else
      program="gorbagana-$program"
    fi

    if [[ -n $CARGO_BUILD_PROFILE ]]; then
      profile_arg="--profile $CARGO_BUILD_PROFILE"
    fi

    # Prebuild binaries so that CI sanity check timeout doesn't include build time
    if [[ $prebuild ]]; then
      (
        set -x
        # shellcheck disable=SC2086 # Don't want to double quote
        cargo $CARGO_TOOLCHAIN build $profile_arg --bin $program
      )
    fi

    printf "cargo $CARGO_TOOLCHAIN run $profile_arg --bin %s %s -- " "$program"
  }
fi

gorbagana_bench_tps=$(gorbagana_program bench-tps)
gorbagana_faucet=$(gorbagana_program faucet)
agave_validator=$(gorbagana_program validator)
agave_validator_cuda="$agave_validator --cuda"
gorbagana_genesis=$(gorbagana_program genesis)
gorbagana_gossip=$(gorbagana_program gossip)
gorbagana_keygen=$(gorbagana_program keygen)
gorbagana_ledger_tool=$(gorbagana_program ledger-tool)
gorbagana_cli=$(gorbagana_program)

export RUST_BACKTRACE=1

default_arg() {
  declare name=$1
  declare value=$2

  for arg in "${args[@]}"; do
    if [[ $arg = "$name" ]]; then
      return
    fi
  done

  if [[ -n $value ]]; then
    args+=("$name" "$value")
  else
    args+=("$name")
  fi
}

replace_arg() {
  declare name=$1
  declare value=$2

  default_arg "$name" "$value"

  declare index=0
  for arg in "${args[@]}"; do
    index=$((index + 1))
    if [[ $arg = "$name" ]]; then
      args[$index]="$value"
    fi
  done
}
