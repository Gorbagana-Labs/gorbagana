---
title: Connecting to a Cluster with the Gorbagana CLI
pagination_label: "Gorbagana CLI: Connecting to a Cluster"
sidebar_label: Connecting to a Cluster
---

See [Gorbagana Clusters](../../clusters/available.md) for general information about the
available clusters.

## Configure the command-line tool

You can check what cluster the Gorbagana command-line tool (CLI) is currently targeting by
running the following command:

```bash
gorbagana config get
```

Use `gorbagana config set` command to target a particular cluster. After setting
a cluster target, any future subcommands will send/receive information from that
cluster.

For example to target the Devnet cluster, run:

```bash
gorbagana config set --url https://api.devnet.gorbagana.com
```

## Ensure Versions Match

Though not strictly necessary, the CLI will generally work best when its version
matches the software version running on the cluster. To get the locally-installed
CLI version, run:

```bash
gorbagana --version
```

To get the cluster version, run:

```bash
gorbagana cluster-version
```

Ensure the local CLI version is greater than or equal to the cluster version.
