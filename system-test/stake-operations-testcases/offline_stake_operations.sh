#!/usr/bin/env bash

# shellcheck disable=SC2086
# shellcheck disable=SC2068
# shellcheck disable=SC2206
# shellcheck disable=SC2162
# shellcheck disable=SC2178
# shellcheck disable=SC2145

# shellcheck disable=SC1090
# shellcheck disable=SC1091
source "$(dirname "$0")"/../automation_utils.sh

set -e

function get_signers_string() {
  sign_only_output="$1"

  signers=()
  while read LINE; do
    signers+=( --signer $LINE )
  done <<<"$(sed -Ee $'s/^  ([a-km-zA-HJ-NP-Z1-9]{32,44}=[a-km-zA-HJ-NP-Z1-9]{64,88})$/\\1/\nt\nd' <<<"$sign_only_output")"

  echo "${signers[@]}"
}

if [[ -n "$1" ]]; then
  url="$1"
fi

if [[ -z "$url" ]]; then
  echo Provide complete URL, ex: "$0" http://api.devnet.gorbagana.com:8899
  exit 1
fi
gorbagana config set --url $url

# Create a dummy keypair file with no balance for operations that require a "client keypair file" to exist even if they don't touch it
dummy_keypair=dummy.json
gorbagana-keygen new -o "$dummy_keypair" --no-passphrase --force --silent
gorbagana config set --keypair $dummy_keypair

### Offline stake account creation

# The nonce account and the system account funding its creation are online
online_nonce_account_keypair=nonce_keypair.json
online_system_account_keypair=online_system_account_keypair.json

gorbagana-keygen new -o "$online_system_account_keypair" --no-passphrase --force --silent
gorbagana-keygen new -o "$online_nonce_account_keypair" --no-passphrase --force --silent

online_system_account_pubkey="$(gorbagana-keygen pubkey $online_system_account_keypair)"
nonce_account_pubkey="$(gorbagana-keygen pubkey $online_nonce_account_keypair)"

# System account funding the stake account is offline, and the auth staker and withdrawer keypairs are offline
offline_system_account_keypair=offline_system_account_keypair.json
offline_staker_keypair=offline_staker_keypair.json
offline_withdrawer_keypair=offline_withdrawer_keypair.json
offline_custodian_keypair=offline_custodian_keypair.json

gorbagana-keygen new -o "$offline_system_account_keypair" --no-passphrase --force --silent
gorbagana-keygen new -o "$offline_staker_keypair" --no-passphrase --force --silent
gorbagana-keygen new -o "$offline_withdrawer_keypair" --no-passphrase --force --silent
gorbagana-keygen new -o "$offline_custodian_keypair" --no-passphrase --force --silent

offline_system_account_pubkey="$(gorbagana-keygen pubkey $offline_system_account_keypair)"
offline_withdrawer_pubkey="$(gorbagana-keygen pubkey $offline_withdrawer_keypair)"
offline_staker_pubkey="$(gorbagana-keygen pubkey $offline_staker_keypair)"
offline_custodian_pubkey="$(gorbagana-keygen pubkey $offline_custodian_keypair)"

# Airdrop some funds to the offline account.
gorbagana airdrop 100 $offline_system_account_pubkey
gorbagana airdrop 2 $online_system_account_pubkey

# Create a nonce account funded by the online account, with the authority given to the offline account
gorbagana create-nonce-account $online_nonce_account_keypair 1 --nonce-authority $offline_system_account_pubkey --keypair $online_system_account_keypair
nonce="$(gorbagana nonce $nonce_account_pubkey)"

execution_step OFFLINE SYSTEM ACCOUNT BALANCE BEFORE CREATING STAKE ACCOUNTS
(
  set -x
  gorbagana balance $offline_system_account_pubkey
)

################################
execution_step CREATE OFFLINE STAKE ACCOUNT
################################

# Create a stake account funded by the offline system account

stake_account_keypair=stake_account_keypair.json
gorbagana-keygen new -o "$stake_account_keypair" --no-passphrase --force --silent
stake_account_address="$(gorbagana-keygen pubkey $stake_account_keypair)"

sign_only="$(gorbagana create-stake-account $stake_account_keypair 50 \
  --sign-only --blockhash $nonce --nonce $nonce_account_pubkey --nonce-authority $offline_system_account_keypair \
  --stake-authority $offline_staker_pubkey --withdraw-authority $offline_withdrawer_pubkey \
  --custodian $offline_custodian_pubkey \
  --lockup-epoch 999 \
  --keypair $offline_system_account_keypair --url http://0.0.0.0)"

signers="$(get_signers_string "${sign_only[@]}")"

gorbagana create-stake-account $stake_account_keypair 50 \
  --blockhash $nonce --nonce $nonce_account_pubkey --nonce-authority $offline_system_account_pubkey \
  --stake-authority $offline_staker_pubkey --withdraw-authority $offline_withdrawer_pubkey \
  --custodian $offline_custodian_pubkey \
  --lockup-epoch 999 \
  --from $offline_system_account_pubkey --fee-payer $offline_system_account_pubkey ${signers[@]}

execution_step VIEW STAKE ACCOUNT AFTER CREATION
(
  set -x
  gorbagana stake-account $stake_account_address
)


execution_step VIEW OFFLINE SYSTEM ACCOUNT BALANCE AFTER CREATING FIRST STAKE ACCOUNT
(
  set -x
  gorbagana balance $offline_system_account_pubkey
)

#####################
execution_step SPLIT STAKE OFFLINE
#####################

# Split the original stake account before delegating

split_stake_account_keypair=split_stake_account_keypair.json
gorbagana-keygen new -o $split_stake_account_keypair --no-passphrase --force --silent
split_stake_account_address=$(gorbagana-keygen pubkey $split_stake_account_keypair)

nonce="$(gorbagana nonce $nonce_account_pubkey)"

sign_only="$(gorbagana split-stake --blockhash $nonce --nonce $nonce_account_pubkey --nonce-authority $offline_system_account_keypair \
  --stake-authority $offline_staker_keypair $stake_account_address $split_stake_account_keypair 10 \
  --keypair $offline_system_account_keypair --sign-only --url http://0.0.0.0)"

signers="$(get_signers_string "${sign_only[@]}")"

gorbagana split-stake --blockhash $nonce --nonce $nonce_account_pubkey --nonce-authority $offline_system_account_pubkey \
  --stake-authority $offline_staker_pubkey $stake_account_address $split_stake_account_keypair 10 \
  --fee-payer $offline_system_account_pubkey ${signers[@]}

execution_step VIEW ORIGINAL STAKE ACCOUNT AFTER SPLITTING
(
  set -x
  gorbagana stake-account $stake_account_address
)

execution_step VIEW NEW STAKE ACCOUNT CREATED FROM SPLITTING ORIGINAL
(
  set -x
  gorbagana stake-account $split_stake_account_address
)

#####################
execution_step CHANGE CUSTODIAN LOCKUP
#####################

# Set the lockup epoch to 0 to allow stake to be withdrawn

nonce="$(gorbagana nonce $nonce_account_pubkey)"

sign_only="$(gorbagana stake-set-lockup --blockhash $nonce --nonce $nonce_account_pubkey --nonce-authority $offline_system_account_keypair \
  $split_stake_account_address --custodian $offline_custodian_keypair --lockup-epoch 0 \
  --keypair $offline_system_account_keypair --sign-only --url http://0.0.0.0)"

signers="$(get_signers_string "${sign_only[@]}")"

gorbagana stake-set-lockup --blockhash $nonce --nonce $nonce_account_pubkey --nonce-authority $offline_system_account_keypair \
  $split_stake_account_address --custodian $offline_custodian_pubkey --lockup-epoch 0 \
  --fee-payer $offline_system_account_pubkey ${signers[@]}

execution_step VIEW SPLIT STAKE ACCOUNT AFTER CHANGING LOCKUP
(
  set -x
  gorbagana stake-account $split_stake_account_address
)

##########################
execution_step OFFLINE STAKE WITHDRAWAL
##########################

# Withdraw the lamports from the stake account that was split off and return them to the offline system account

nonce="$(gorbagana nonce $nonce_account_pubkey)"

sign_only="$(gorbagana withdraw-stake --blockhash $nonce --nonce $nonce_account_pubkey --nonce-authority $offline_system_account_keypair \
   $split_stake_account_address $offline_system_account_pubkey 10 \
  --withdraw-authority $offline_withdrawer_keypair \
  --keypair $offline_system_account_keypair --sign-only --url http://0.0.0.0)"

signers="$(get_signers_string "${sign_only[@]}")"

gorbagana withdraw-stake --blockhash $nonce --nonce $nonce_account_pubkey --nonce-authority $offline_system_account_pubkey \
  $split_stake_account_address $offline_system_account_pubkey 10 \
  --withdraw-authority $offline_withdrawer_pubkey \
  --fee-payer $offline_system_account_pubkey ${signers[@]}

execution_step VIEW OFFLINE SYSTEM ACCOUNT BALANCE AFTER WITHDRAWING SPLIT STAKE
(
  set -x
  gorbagana balance $offline_system_account_pubkey
)

##########################
execution_step OFFLINE STAKE DELEGATION
##########################

# Delegate stake from the original account to a vote account

vote_account_pubkey="$(awk '{if(NR==4) print $2}'<<<"$(gorbagana show-validators)")"
nonce="$(gorbagana nonce $nonce_account_pubkey)"

# Sign a stake delegation, assuming the authorized staker is held offline
sign_only="$(gorbagana delegate-stake --blockhash $nonce --nonce $nonce_account_pubkey --nonce-authority $offline_system_account_keypair \
--stake-authority $offline_staker_keypair $stake_account_address $vote_account_pubkey \
--keypair $offline_system_account_keypair --sign-only --url http://0.0.0.0)"

signers="$(get_signers_string "${sign_only[@]}")"

# Send the signed transaction on the cluster
gorbagana delegate-stake --blockhash $nonce --nonce $nonce_account_pubkey --nonce-authority $offline_system_account_pubkey \
--stake-authority $offline_staker_pubkey $stake_account_address $vote_account_pubkey \
--fee-payer $offline_system_account_pubkey ${signers[@]}

execution_step VIEW ORIGINAL STAKE ACCOUNT AFTER DELEGATION
(
  set -x
  gorbagana stake-account $stake_account_address
)

##########################
 execution_step OFFLINE STAKE DEACTIVATION
##########################

# Deactivate delegated stake

nonce="$(gorbagana nonce $nonce_account_pubkey)"

# Sign a stake delegation, assuming the authorized staker is held offline
sign_only="$(gorbagana deactivate-stake --blockhash $nonce --nonce $nonce_account_pubkey --nonce-authority $offline_system_account_keypair \
--stake-authority $offline_staker_keypair $stake_account_address \
--keypair $offline_system_account_keypair --sign-only --url http://0.0.0.0)"

signers="$(get_signers_string "${sign_only[@]}")"

# Send the signed transaction on the cluster
gorbagana deactivate-stake --blockhash $nonce --nonce $nonce_account_pubkey --nonce-authority $offline_system_account_pubkey \
--stake-authority $offline_staker_pubkey $stake_account_address \
--fee-payer $offline_system_account_pubkey ${signers[@]}

execution_step VIEW ORIGINAL STAKE ACCOUNT AFTER DEACTIVATION
(
   set -x
   gorbagana stake-account $stake_account_address
)
