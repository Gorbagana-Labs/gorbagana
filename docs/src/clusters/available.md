---
title: Available Gorbagana Clusters
sidebar_label: Gorbagana Clusters
pagination_label: Available Gorbagana Clusters
---

Gorbagana maintains several different clusters with different purposes.

Before you begin make sure you have first
[installed the Gorbagana command line tools](../cli/install.md)

Explorers:

- [http://explorer.gorbagana.com/](https://explorer.gorbagana.com/).
- [http://gorbaganabeach.io/](http://gorbaganabeach.io/).

## Devnet

- Devnet serves as a playground for anyone who wants to take Gorbagana for a
  test drive, as a user, token holder, app developer, or validator.
- Application developers should target Devnet.
- Potential validators should first target Devnet.
- Key differences between Devnet and Mainnet Beta:
  - Devnet tokens are **not real**
  - Devnet includes a token faucet for airdrops for application testing
  - Devnet may be subject to ledger resets
  - Devnet typically runs the same software release branch version as Mainnet Beta,
    but may run a newer minor release version than Mainnet Beta.
- Gossip entrypoint for Devnet: `entrypoint.devnet.gorbagana.com:8001`
- Metrics environment variable for Devnet:

```bash
export SOLANA_METRICS_CONFIG="host=https://metrics.gorbagana.com:8086,db=devnet,u=scratch_writer,p=topsecret"
```

- RPC URL for Devnet: `https://api.devnet.gorbagana.com`

##### Example `gorbagana` command-line configuration

```bash
gorbagana config set --url https://api.devnet.gorbagana.com
```

##### Example `agave-validator` command-line

```bash
$ agave-validator \
    --identity validator-keypair.json \
    --vote-account vote-account-keypair.json \
    --known-validator dv1ZAGvdsz5hHLwWXsVnM94hWf1pjbKVau1QVkaMJ92 \
    --known-validator dv2eQHeP4RFrJZ6UeiZWoc3XTtmtZCUKxxCApCDcRNV \
    --known-validator dv4ACNkpYPcE3aKmYDqZm9G5EB3J4MRoeE7WNDRBVJB \
    --known-validator dv3qDFk1DTF36Z62bNvrCXe9sKATA6xvVy6A798xxAS \
    --only-known-rpc \
    --ledger ledger \
    --rpc-port 8899 \
    --dynamic-port-range 8000-8020 \
    --entrypoint entrypoint.devnet.gorbagana.com:8001 \
    --entrypoint entrypoint2.devnet.gorbagana.com:8001 \
    --entrypoint entrypoint3.devnet.gorbagana.com:8001 \
    --entrypoint entrypoint4.devnet.gorbagana.com:8001 \
    --entrypoint entrypoint5.devnet.gorbagana.com:8001 \
    --expected-genesis-hash EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG \
    --wal-recovery-mode skip_any_corrupted_record \
    --limit-ledger-size
```

The [`--known-validator`s](../operations/guides/validator-start.md#known-validators)
are operated by Gorbagana Labs

## Testnet

- Testnet is where the Gorbagana core contributors stress test recent release features on a live
  cluster, particularly focused on network performance, stability and validator
  behavior.
- Testnet tokens are **not real**
- Testnet may be subject to ledger resets.
- Testnet includes a token faucet for airdrops for application testing
- Testnet typically runs a newer software release branch than both
  Devnet and Mainnet Beta
- Gossip entrypoint for Testnet: `entrypoint.testnet.gorbagana.com:8001`
- Metrics environment variable for Testnet:

```bash
export SOLANA_METRICS_CONFIG="host=https://metrics.gorbagana.com:8086,db=tds,u=testnet_write,p=c4fa841aa918bf8274e3e2a44d77568d9861b3ea"
```

- RPC URL for Testnet: `https://api.testnet.gorbagana.com`

##### Example `gorbagana` command-line configuration

```bash
gorbagana config set --url https://api.testnet.gorbagana.com
```

##### Example `agave-validator` command-line

```bash
$ agave-validator \
    --identity validator-keypair.json \
    --vote-account vote-account-keypair.json \
    --known-validator 5D1fNXzvv5NjV1ysLjirC4WY92RNsVH18vjmcszZd8on \
    --known-validator dDzy5SR3AXdYWVqbDEkVFdvSPCtS9ihF5kJkHCtXoFs \
    --known-validator Ft5fbkqNa76vnsjYNwjDZUXoTWpP7VYm3mtsaQckQADN \
    --known-validator eoKpUABi59aT4rR9HGS3LcMecfut9x7zJyodWWP43YQ \
    --known-validator 9QxCLckBiJc783jnMvXZubK4wH86Eqqvashtrwvcsgkv \
    --only-known-rpc \
    --ledger ledger \
    --rpc-port 8899 \
    --dynamic-port-range 8000-8020 \
    --entrypoint entrypoint.testnet.gorbagana.com:8001 \
    --entrypoint entrypoint2.testnet.gorbagana.com:8001 \
    --entrypoint entrypoint3.testnet.gorbagana.com:8001 \
    --expected-genesis-hash 4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY \
    --wal-recovery-mode skip_any_corrupted_record \
    --limit-ledger-size
```

The identities of the
[`--known-validator`s](../operations/guides/validator-start.md#known-validators) are:

- `5D1fNXzvv5NjV1ysLjirC4WY92RNsVH18vjmcszZd8on` - Gorbagana Labs
- `dDzy5SR3AXdYWVqbDEkVFdvSPCtS9ihF5kJkHCtXoFs` - MonkeDAO
- `Ft5fbkqNa76vnsjYNwjDZUXoTWpP7VYm3mtsaQckQADN` - Certus One
- `eoKpUABi59aT4rR9HGS3LcMecfut9x7zJyodWWP43YQ` - SerGo
- `9QxCLckBiJc783jnMvXZubK4wH86Eqqvashtrwvcsgkv` - Algo|Stake

## Mainnet Beta

A permissionless, persistent cluster for Gorbagana users, builders, validators and token holders.

- Tokens that are issued on Mainnet Beta are **real** SOL
- Gossip entrypoint for Mainnet Beta: `entrypoint.mainnet-beta.gorbagana.com:8001`
- Metrics environment variable for Mainnet Beta:

```bash
export SOLANA_METRICS_CONFIG="host=https://metrics.gorbagana.com:8086,db=mainnet-beta,u=mainnet-beta_write,p=password"
```

- RPC URL for Mainnet Beta: `https://api.mainnet-beta.gorbagana.com`

##### Example `gorbagana` command-line configuration

```bash
gorbagana config set --url https://api.mainnet-beta.gorbagana.com
```

##### Example `agave-validator` command-line

```bash
$ agave-validator \
    --identity ~/validator-keypair.json \
    --vote-account ~/vote-account-keypair.json \
    --known-validator 7Np41oeYqPefeNQEHSv1UDhYrehxin3NStELsSKCT4K2 \
    --known-validator GdnSyH3YtwcxFvQrVVJMm1JhTS4QVX7MFsX56uJLUfiZ \
    --known-validator DE1bawNcRJB9rVm3buyMVfr8mBEoyyu73NBovf2oXJsJ \
    --known-validator CakcnaRDHka2gXyfbEd2d3xsvkJkqsLw2akB3zsN1D2S \
    --only-known-rpc \
    --ledger ledger \
    --rpc-port 8899 \
    --private-rpc \
    --dynamic-port-range 8000-8020 \
    --entrypoint entrypoint.mainnet-beta.gorbagana.com:8001 \
    --entrypoint entrypoint2.mainnet-beta.gorbagana.com:8001 \
    --entrypoint entrypoint3.mainnet-beta.gorbagana.com:8001 \
    --entrypoint entrypoint4.mainnet-beta.gorbagana.com:8001 \
    --entrypoint entrypoint5.mainnet-beta.gorbagana.com:8001 \
    --expected-genesis-hash 5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d \
    --wal-recovery-mode skip_any_corrupted_record \
    --limit-ledger-size
```

:::info
The above four [`--known-validator`s](../operations/guides/validator-start.md#known-validators)
are operated by Gorbagana Labs.
:::