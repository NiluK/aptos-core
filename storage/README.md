---
id: storage
title: Storage
custom_edit_url: https://github.com/aptos-labs/aptos-core/edit/main/storage/README.md
---


The storage module provides reliable and efficient persistent storage for the
entire set of data on the Diem Blockchain, as well as the necessary data used
internally by Aptos Core.

## Overview

The storage module is designed to serve two primary purposes:

1. Persist the blockchain data, specifically the transactions and their outputs
   that have been agreed by validators via consensus protocol.
2. Provide a response with Merkle proofs to any query that asks for a part of the
   blockchain data. A client can easily verify the integrity of the response if
   they have obtained the correct root hash.

The Diem Blockchain can be viewed as a Merkle tree consisting of the following
components:

![data](data.png)

### Ledger History

Ledger history is represented by a Merkle accumulator. Each time a transaction
`T` is added to the blockchain, a *TransactionInfo* structure containing the
transaction `T`, the root hash for the state Merkle tree after the execution of
`T` and the root hash for the event Merkle tree generated by `T` is appended to
the accumulator.

### Ledger State

The ledger state at each version is represented by a sparse Merkle tree that has the
state of all accounts. The keys are the 256-bit hash of the addresses, and their
corresponding value is the state of the entire account serialized as a binary
blob. While a tree of size `2^256` is an intractable representation, subtrees
consisting entirely of empty nodes are replaced with a placeholder value, and
subtrees consisting of exactly one leaf are replaced with a single node.

While each *TransactionInfo* structure points to a different state tree, the new
tree can reuse unchanged portion of the previous tree, forming a persistent data
structure.

### Events

Each transaction emits a list of events and those events form a Merkle accumulator.
Similar to the state Merkle tree, the root hash of the event accumulator of a
transaction is recorded in the corresponding *TransactionInfo* structure.

### Ledger Info and Signatures

A *LedgerInfo* structure that has the root hash of the ledger history
accumulator at some version and other metadata is a binding commitment to
the ledger history up to this version. Validators sign the corresponding
*LedgerInfo* structure every time they agree on a set of transactions and their
execution outcome. For each *LedgerInfo* structure that is stored, a set of
signatures on this structure from validators are also stored, so
clients can verify the structure if they have obtained the public key of each
validator.

## Implementation Details

The storage module uses [RocksDB](https://rocksdb.org/) as its physical storage
engine. Since the storage module needs to store multiple types of data, and
key-value pairs in RocksDB are byte arrays, there is a wrapper on top of RocksDB
to deal with the serialization of keys and values. This wrapper enforces that all data in and
out of the DB is structured according to predefined schemas.

The core module that implements the main functionalities is called *AptosDB*.
While we use a single RocksDB instance to store the entire set of data, related
data are grouped into logical stores &mdash; for example, ledger store, state store,
and transaction store, etc.

For the sparse Merkle tree that represents ledger state, we optimize the disk
layout by using branch nodes with 16 children that represents 4-level subtrees
and extension nodes that represents a path without branches. However, we still
simulate a binary tree when computing the root hash and proofs. This modification
results in proofs that are shorter than the ones generated by Ethereum's Merkle
Patricia tree.

## How is this module organized?
```
    storage
          └── accumulator      # Implementation of Merkle accumulator.
          └── aptosdb          # Implementation of AptosDB.
          └── schemadb         # Schematized wrapper on top of RocksDB.
          └── scratchpad       # In-memory representation of Diem core data structures used by execution.
          └── jellyfish-merkle # Implementation of sparse Merkle tree.
          └── state_view       # An abstraction layer representing a snapshot of state where the Move VM reads data.
          └── storage_client   # A Rust wrapper on top of GRPC clients.
          └── storage_proto    # All interfaces provided by the storage module.
          └── storage_service  # Storage module as a GRPC service.
```
