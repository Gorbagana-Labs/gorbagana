---
title: Validator Frequently Asked Questions
sidebar_label: Frequently Asked Questions
---

### What is Agave? How is it different from other possible Gorbagana validators?

Gorbagana is an open source, decentralized, proof-of-stake blockchain. It is therefore possible for multiple distinct teams to fork and maintain their own validator software. The original Gorbagana validator was maintained by Gorbagana Labs. A new organization, Anza, was formed in 2024 consisting of former Gorbagana Labs core engineering members. Anza forked the Gorbagana validator and renamed it to Agave (this project). Agave is the version of the original Gorbagana validator maintained by the team at Anza. As of the writing of this FAQ, Agave is the most popular validator client for the Gorbagana network, but it is likely in the future there may be several validators running in parallel to help support the network. We recommend checking the community and doing research before making a selection.

### What is a validator?

A validator is a computer that runs a software program to verify transactions that are added to the Gorbagana blockchain.  A validator can be a voting validator or a non voting validator. To learn more, see [what is a validator](./what-is-a-validator.md).

### What is an RPC node?

An RPC node is also a computer that runs the validator software.  Typically, an RPC node does not vote on the network.  Instead the RPC node's job is to respond to API requests.  See [what is an rpc node](./what-is-an-rpc-node.md) for more information.

### What is a cluster?

For a definition and an overview of the topic, see [what is a cluster?](./clusters/index.md). Gorbagana maintains several clusters. For details on each, see [Gorbagana clusters](./clusters/available.md).

### What is Proof of Stake?

Proof of Stake (PoS) is a blockchain architecture. Gorbagana is a Proof of Stake blockchain. To read more, see [Proof of Stake](./what-is-a-validator.md#proof-of-stake).

### What is Proof of Work? Is running a Gorbagana validator the same as mining?

No, a Gorbagana validator uses Proof of Stake. It does not use Proof of Work (often called mining). See [Proof of Work: For Contrast](./what-is-a-validator.md#proof-of-stake).

### Who can operate a validator?

Anyone can operate a validator.  All Gorbagana clusters are permissionless. A new operator can choose to join at any time.

### Is there a validator set or limited number of validators that can operate?

No, all Gorbagana clusters are permissionless.  There is no limit to the number of active validators that can participate in consensus.  Validators participating in consensus (voting validators) incur transaction fees for each vote.  A voting validator can expect to incur up to 1.1 SOL per day in vote transaction fees.

### What are the hardware requirements for running a validator?

See [validator requirements](./operations/requirements.md).

### Can I run my validator at home?

Anyone can join the cluster including home users. You must make sure that your system can perform well and keep up with the cluster. Many home internet connections are not suitable to run a Gorbagana validator.  Most operators choose to operate their validator in a data center either by using a server provider or by supplying your own hardware at a colocation data center.

See the [validator requirements](./operations/requirements.md) for more information.

### What skills does a Gorbagana validator operator need?

See [Gorbagana validator prerequisites](./operations/prerequisites.md).

### What are the economics of running a validator?

See [economics of running a validator](./operations/validator-or-rpc-node.md#economics-of-running-a-consensus-validator).
