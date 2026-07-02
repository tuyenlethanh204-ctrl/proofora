# Proofora

**Batch Verification on Stellar.**

- [Demo video](https://youtu.be/CbwDxRS5t7Q)
- [Live demo](https://proofora-batch.vercel.app)
- [Testnet contract](https://stellar.expert/explorer/testnet/contract/CASSNKDNJJCSILZ7SEF25SP336AXEZRPGSBFAJSTHST5U3FPV5OG4RCY)
- [On-chain verification transaction](https://stellar.expert/explorer/testnet/tx/d2cfe9051ee9e8114de49ce2f3ab82f6d0a38fb398f0a2e78daaa66f3d1925d4)

## Project Overview

Proofora is a Stellar/Soroban proof-batching prototype designed for applications that need to apply several proof-backed actions without submitting a separate contract invocation for every proof.

In a conventional flow, each action carries its own proof and public input, and each item is submitted independently. For a batch of `N` actions, this produces `N` transactions or contract calls, repeating submission, authorization, ledger inclusion, and application overhead. That model is simple, but it becomes inefficient for systems that process many independent attestations, eligibility checks, private votes, identity claims, or other proof-derived state transitions.

Proofora introduces a batch boundary between proof production and on-chain application:

1. Independent proof envelopes and their public inputs are collected off-chain.
2. Each public input is represented as a fixed 32-byte leaf.
3. The ordered leaves are concatenated and committed to a SHA-256 batch root.
4. A single Soroban invocation submits the proof envelopes, public-input leaves, and expected root.
5. The contract validates the batch shape, rejects invalid proof envelopes, recomputes the root, and returns a receipt containing the accepted root and batch size.

The result is an `N-to-1` transaction path: multiple proof-backed inputs are bound to one deterministic commitment and processed through one contract call. The current testnet deployment demonstrates this flow for `N = 2`, while the local benchmark covers `N = 2`, `4`, and `8`.

This repository implements the plan-approved **batch verification fallback**. It is intentionally precise about its security boundary: the deployed contract validates proof-envelope structure and public-input integrity, but it does not verify recursive RISC Zero receipts or a Groth16 proof. Its demonstrated value is transaction and application-overhead reduction, not recursive cryptographic proof compression.

## What the Project Demonstrates

- Deterministic construction of proof-like test fixtures and public-input leaves.
- SHA-256 commitment over an ordered batch of public inputs.
- One Soroban `verify_many` call for `N` proof-backed inputs.
- Explicit rejection of empty batches, proof-count mismatches, malformed proof envelopes, and incorrect roots.
- A receipt containing the accepted batch root and number of items.
- Local benchmarks comparing `N` individual checks with one batch path.
- A deployed Stellar testnet contract with reproducible transaction evidence.
- A public web demo that reads live Soroban RPC data and recomputes the recorded root in the browser.

## Verification Flow

```text
Independent actions
       |
       v
proof envelope + public input
       |
       v
32-byte public-input leaves
       |
       v
SHA-256(leaf[0] || leaf[1] || ... || leaf[n-1])
       |
       v
proofs + leaves + expected root
       |
       v
Soroban BatchVerifier.verify_many
       |
       +-- validate non-empty batch
       +-- validate proofs.len == public_inputs.len
       +-- validate each proof envelope
       +-- recompute and compare batch root
       |
       v
BatchReceipt { root, n }
```

Leaf order is part of the commitment. Reordering, replacing, or mutating any public input changes the computed root and causes the submitted batch commitment to fail.

## Architecture

### Off-chain batch model

[`scripts/aggregator.mjs`](scripts/aggregator.mjs) provides the deterministic reference implementation used by the tests and benchmark.

- `makeSubProof` creates a proof-like envelope for a bounded value claim.
- `leafForPublicInput` hashes the public claim fields into a leaf.
- `batchRoot` hashes the ordered concatenation of all leaves.
- `verifySubProof` checks envelope encoding, field consistency, range constraints, and leaf integrity.
- `verifyBatch` validates every item and compares the recomputed root with the expected root.
- `fixture` generates reproducible batches for `N = 2`, `4`, and `8`.

These proof envelopes are fixtures for exercising the batching pipeline. They are not presented as production zero-knowledge proofs.

### Soroban contract

[`contracts/batch_verifier`](contracts/batch_verifier) contains the deployed Rust contract.

The contract exposes two public functions:

```rust
compute_root(public_inputs: Vec<BytesN<32>>) -> Result<BytesN<32>, Error>

verify_many(
    proofs: Vec<Bytes>,
    public_inputs: Vec<BytesN<32>>,
    expected_root: BytesN<32>,
) -> Result<BatchReceipt, Error>
```

`compute_root` concatenates the ordered 32-byte leaves and computes their SHA-256 digest using the Soroban host crypto API.

`verify_many` enforces the batch invariants and returns:

```rust
BatchReceipt {
    root: expected_root,
    n: public_inputs.len(),
}
```

Contract errors are stable and explicit:

| Code | Error | Condition |
|---:|---|---|
| `1` | `EmptyBatch` | No public inputs were supplied |
| `2` | `ProofCountMismatch` | Proof and public-input counts differ |
| `3` | `InvalidProof` | A proof envelope is empty or has an invalid leading byte |
| `4` | `BatchRootMismatch` | The recomputed SHA-256 root differs from the submitted root |

### Web demo

[`frontend/index.html`](frontend/index.html) is the public evidence dashboard deployed at [proofora-batch.vercel.app](https://proofora-batch.vercel.app).

The demo:

- queries the Stellar testnet Soroban RPC for the latest ledger;
- retrieves the recorded `verify_many` transaction status;
- links directly to the contract and deployment transactions;
- recomputes the `N = 2` batch root with the browser Web Crypto API;
- compares the local result with the recorded on-chain return root;
- displays the `N = 2`, `4`, and `8` benchmark results;
- labels the implementation as a batch-verification fallback without recursive-compression claims.

Press **Run fallback verification** to execute the root computation and receipt comparison locally in the browser. A successful run ends with `receipt comparison: pass` and `Root matched`.

## Testnet Deployment

| Item | Value |
|---|---|
| Network | Stellar Testnet |
| Contract ID | `CASSNKDNJJCSILZ7SEF25SP336AXEZRPGSBFAJSTHST5U3FPV5OG4RCY` |
| WASM SHA-256 | `6bcc3d3107db34038815faaa1415d94420e79a084e44b444f128b5e5a96ed60e` |
| Verified batch size | `2` |
| Returned root | `f818afd37a6dc3bc92fb44731011277006db4efa6e9023cd7468c02335d22a4d` |

### Deployment transactions

| Operation | Transaction |
|---|---|
| Upload contract WASM | [`1c0e1452...dea089d`](https://stellar.expert/explorer/testnet/tx/1c0e1452093092a1ae606972e54a5ea646c36fbdbb4ee21845b69f6b3dea089d) |
| Deploy contract | [`fdc0acee...b9256fe`](https://stellar.expert/explorer/testnet/tx/fdc0aceef86b205ea35f57c9a1b48b7abd53eef46c8f5b68977eba8cbb9256fe) |
| Execute `verify_many` with `N = 2` | [`d2cfe905...d1925d4`](https://stellar.expert/explorer/testnet/tx/d2cfe9051ee9e8114de49ce2f3ab82f6d0a38fb398f0a2e78daaa66f3d1925d4) |

The recorded call returned:

```json
{
  "n": 2,
  "root": "f818afd37a6dc3bc92fb44731011277006db4efa6e9023cd7468c02335d22a4d"
}
```

The fetched testnet WASM hash matches the output of the local Soroban contract build. Full deployment evidence is preserved in [`TESTNET_DEPLOYMENT.md`](TESTNET_DEPLOYMENT.md).

## Benchmark

The benchmark measures transaction-count reduction and local fallback-verification behavior. It does not measure recursive proof generation or recursive verifier performance.

| Batch size | Individual path | Batch path | Transaction reduction |
|---:|---:|---:|---:|
| `2` | `2` transactions | `1` transaction | `2x` |
| `4` | `4` transactions | `1` transaction | `4x` |
| `8` | `8` transactions | `1` transaction | `8x` |

Run the benchmark with:

```bash
npm run bench
```

The command writes the machine-readable result to [`artifacts/benchmark.json`](artifacts/benchmark.json). Timing values are local microbenchmarks and can vary by machine; the deterministic result being demonstrated is the reduction from `N` application submissions to one batch submission.

## Repository Structure

```text
proofora/
|-- contracts/
|   `-- batch_verifier/       Deployed Soroban batch-verification contract
|-- scripts/
|   |-- aggregator.mjs        Deterministic fixture and batch logic
|   |-- aggregator.test.mjs   Node.js integrity and mutation tests
|   `-- bench.mjs             N=2,4,8 benchmark generator
|-- frontend/
|   |-- assets/               Proofora visual assets
|   |-- index.html            Live testnet evidence dashboard
|   |-- package.json
|   `-- package-lock.json
|-- web/
|   |-- build.mjs             Static benchmark dashboard generator
|   `-- dist/                 Generated static dashboard output
|-- artifacts/
|   |-- benchmark.json        Latest benchmark result
|   |-- testnet-proofs-n2.json
|   `-- testnet-public-inputs-n2.json
|-- TESTNET_DEPLOYMENT.md     Deployment identifiers and transaction evidence
`-- package.json              JavaScript test, benchmark, and demo commands
```

## Requirements

- Node.js 20 or newer.
- Rust toolchain compatible with the contract workspace.
- Stellar CLI with Soroban contract build support.
- The `wasm32v1-none` Rust target required by the Stellar contract toolchain.

The JavaScript reference implementation uses Node.js built-in modules and has no runtime package dependency.

## Local Development

From the project root:

```bash
npm test
npm run bench
npm run demo
```

The demo starts a static server for `frontend/`. Open the local URL printed by the command.

To generate the compact static benchmark dashboard:

```bash
npm run web:build
```

## Contract Testing and Build

Run the Soroban contract tests:

```bash
cargo test --manifest-path contracts/batch_verifier/Cargo.toml
```

Build the deployable contract WASM:

```bash
stellar contract build --manifest-path contracts/batch_verifier/Cargo.toml
```

On constrained machines, limit Cargo parallelism:

```bash
CARGO_BUILD_JOBS=1 cargo test --manifest-path contracts/batch_verifier/Cargo.toml
CARGO_BUILD_JOBS=1 stellar contract build --manifest-path contracts/batch_verifier/Cargo.toml
```

The contract test suite covers:

- successful batch verification and receipt generation;
- rejection of a mutated batch root;
- rejection of a malformed proof envelope.

The Node.js test suite additionally verifies valid batches for `N = 2`, `4`, and `8`, mutated public-input rejection, and incorrect-root rejection.

## Security and Trust Model

The SHA-256 root provides deterministic integrity for an ordered set of public-input leaves. It allows the contract and clients to agree on exactly which inputs belong to a batch and detect mutation or reordering.

The current proof-envelope check is intentionally minimal: a proof must be non-empty and begin with a non-zero marker byte. Therefore, the fallback contract must not be treated as a production cryptographic proof verifier. A production version would replace this envelope predicate with verification of a real proof system while preserving the same batching interface and public-input commitment boundary.

The current implementation guarantees:

- proof and public-input array lengths match;
- the batch is non-empty;
- every submitted proof envelope passes the fallback shape check;
- the ordered public inputs reproduce the expected SHA-256 root;
- the returned receipt identifies the accepted root and item count.

It does not guarantee:

- recursive proof soundness;
- RISC Zero receipt verification on Soroban;
- Groth16 verification;
- semantic validity of arbitrary external proof formats;
- production-grade authorization, replay protection, or application-specific state transitions.

Those boundaries are explicit so that the deployed demo can be evaluated on what it actually proves: a working, testnet-backed `N-to-1` batch submission path with deterministic public-input integrity.
