# Proofora

**Batch Verification on Stellar.**

- [Demo video](https://youtu.be/F5SKTwubxF0)
- [Live demo](https://proofora-batch.vercel.app)
- [Testnet contract](https://stellar.expert/explorer/testnet/contract/CC6HXCAELE2M5W4ZLAIRVX2NQOOSUIJGBBFWYJRR5HT5FBGGGDYWPJJN)
- [On-chain `verify_many` transaction](https://stellar.expert/explorer/testnet/tx/0e6c062dec34d48e7ba8b64e1bbb05ce23bb172ed1934c0e4992cdeb7a628667)

## Project Overview

Proofora is a Stellar/Soroban proof-batching prototype designed for applications that need to apply several proof-backed actions without submitting a separate contract invocation for every proof.

In a conventional flow, each action carries its own proof and public input, and each item is submitted independently. For a batch of `N` actions, this produces `N` transactions or contract calls, repeating submission, authorization, ledger inclusion, and application overhead. That model is simple, but it becomes inefficient for systems that process many independent attestations, eligibility checks, private votes, identity claims, or other proof-derived state transitions.

Proofora introduces a batch boundary between proof production and on-chain application:

1. Independent proof envelopes and their public inputs are collected off-chain.
2. Each public input is represented as a fixed 32-byte leaf.
3. The ordered leaves are concatenated and committed to a SHA-256 batch root.
4. A single Soroban invocation submits the app id, proof envelopes, public-input leaves, expected root, nullifier hashes, and retention proof.
5. The contract validates the batch shape, rejects invalid proof envelopes, rejects unregistered apps, rejects duplicate nullifiers, rejects zero retention proofs, recomputes the root, stores a checkpoint, and returns a receipt containing the accepted root, batch size, and receipt id.

The result is an `N-to-1` transaction path: multiple proof-backed inputs are bound to one deterministic commitment and processed through one contract call. Local tests and benchmarks demonstrate this flow for `N = 2`, `4`, and `8`; the latest testnet contract demonstrates the on-chain path for `N = 1`.

This repository implements the plan-approved **batch verification fallback**. It is intentionally precise about its security boundary: the deployed contract validates proof-envelope structure and public-input integrity, but it does not verify recursive RISC Zero receipts or a Groth16 proof. Its demonstrated value is transaction and application-overhead reduction, not recursive cryptographic proof compression.

## What the Project Demonstrates

- Deterministic construction of proof-like test fixtures and public-input leaves.
- SHA-256 commitment over an ordered batch of public inputs.
- One Soroban `verify_many` call for `N` proof-backed inputs.
- Explicit rejection of empty batches, proof-count mismatches, malformed proof envelopes, incorrect roots, unregistered apps, duplicate nullifiers, and zero retention proofs.
- A receipt containing the accepted batch root, number of items, and receipt id.
- Persistent checkpoint storage for accepted roots, ledger sequence, and retention proof.
- Local benchmarks comparing `N` individual checks with one batch path.
- A deployed Stellar testnet contract with recorded invoke evidence.
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
       +-- validate registered app id
       +-- validate non-empty batch
       +-- validate proofs.len == public_inputs.len
       +-- reject duplicate nullifier hashes
       +-- validate each proof envelope
       +-- recompute and compare batch root
       +-- reject zero retention proof
       +-- store checkpoint
       |
       v
BatchReceipt { root, n, receipt_id }
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

## Competitive Benchmark

While other infrastructure solutions like Mnemox or general-purpose aggregators focus exclusively on proof compression off-chain, RootVault takes a different approach. RootVault focuses on solving the data availability and indexing pain points directly on Stellar. It provides durable commitment roots, nullifier set indexing to prevent double-spending, batch receipts, and retrievable historical proofs anchored natively to Soroban state. This ensures privacy apps have the basic required on-chain primitives before tackling complex recursive aggregation.

The contract exposes two public functions:

```rust
compute_root(public_inputs: Vec<BytesN<32>>) -> Result<BytesN<32>, Error>

verify_many(
    app_id: BytesN<32>,
    proofs: Vec<Bytes>,
    public_inputs: Vec<BytesN<32>>,
    expected_root: BytesN<32>,
    nullifier_hashes: Vec<BytesN<32>>,
    retention_proof: BytesN<32>,
) -> Result<BatchReceipt, Error>
```

`compute_root` concatenates the ordered 32-byte leaves and computes their SHA-256 digest using the Soroban host crypto API.

`verify_many` enforces the batch invariants and returns:

```rust
BatchReceipt {
    root: expected_root,
    n: public_inputs.len(),
    receipt_id,
}
```

Contract errors are stable and explicit:

| Code | Error | Condition |
|---:|---|---|
| `1` | `EmptyBatch` | No public inputs were supplied |
| `2` | `ProofCountMismatch` | Proof and public-input counts differ |
| `3` | `InvalidProof` | A proof envelope is empty or has an invalid leading byte |
| `4` | `BatchRootMismatch` | The recomputed SHA-256 root differs from the submitted root |
| `5` | `AppNotRegistered` | The submitted app id has not been registered |
| `6` | `DuplicateNullifier` | A submitted nullifier hash was already used for this app |
| `7` | `InvalidRetentionProof` | The retention proof is all zero bytes |
| `8` | `StaleRoot` | Reserved for stale-root policy checks |
| `9` | `StalePolicy` | Reserved for stale-policy checks |

### Web demo

[`frontend/index.html`](frontend/index.html) is the public evidence dashboard deployed at [proofora-batch.vercel.app](https://proofora-batch.vercel.app).

The demo:

- queries the Stellar testnet Soroban RPC for the latest ledger;
- links directly to the latest recorded testnet contract and `verify_many` transaction;
- recomputes the `N = 1` testnet batch root with the browser Web Crypto API;
- compares the local result with the recorded on-chain return root;
- displays the `N = 2`, `4`, and `8` benchmark results;
- labels the implementation as a batch-verification fallback without recursive-compression claims.

Press **Run fallback verification** to execute the root computation and receipt comparison locally in the browser. A successful run ends with `receipt comparison: pass` and `Root matched`.

## Testnet Deployment

| Item | Value |
|---|---|
| Network | Stellar Testnet |
| Contract ID | `CC6HXCAELE2M5W4ZLAIRVX2NQOOSUIJGBBFWYJRR5HT5FBGGGDYWPJJN` |
| WASM SHA-256 | `fc034993c1c804c0f92b3910eed85f5fbb7b62da5f9ac20c26e941482f91367a` |
| Verified batch size | `1` |
| Returned root | `75877bb41d393b5fb8455ce60ecd8dda001d06316496b14dfa7f895656eeca4a` |

### Deployment transactions

| Operation | Transaction |
|---|---|
| Upload contract WASM | [`b8d63121...999c0`](https://stellar.expert/explorer/testnet/tx/b8d631210d22064382ec179921a163cb1a47913aa716c1e1254c46bd8fb999c0) |
| Deploy contract | [`bcc31fba...21e44`](https://stellar.expert/explorer/testnet/tx/bcc31fba9673508da4122b882b1ad75fbebe5417be44ff0e2ea1381330f21e44) |
| Register app | [`b2e69b72...71c56`](https://stellar.expert/explorer/testnet/tx/b2e69b72b00ae2efb6d343c637534c2ba30777cb36f0f1fa7746b07989771c56) |
| Execute `verify_many` | [`0e6c062d...28667`](https://stellar.expert/explorer/testnet/tx/0e6c062dec34d48e7ba8b64e1bbb05ce23bb172ed1934c0e4992cdeb7a628667) |

The recorded `verify_many` call returned:

```json
{
  "n": 1,
  "receipt_id": 1,
  "root": "75877bb41d393b5fb8455ce60ecd8dda001d06316496b14dfa7f895656eeca4a"
}
```

Full deployment evidence is preserved in [`TESTNET_DEPLOYMENT.md`](TESTNET_DEPLOYMENT.md).

## Evidence Bundle

Machine-readable evidence is stored under [`evidence/`](evidence/):

| File | Purpose |
|---|---|
| [`evidence/tool-versions.txt`](evidence/tool-versions.txt) | Tool versions used for the latest local verification pass |
| [`evidence/public-input-manifest.json`](evidence/public-input-manifest.json) | Public-input order and field descriptions for app id, commitment root, nullifier hash, batch root, receipt id, ledger sequence, and retention proof |
| [`evidence/verifier-artifacts.json`](evidence/verifier-artifacts.json) | Local verifier artifact hashes and deterministic fixture data |
| [`evidence/local-latest.json`](evidence/local-latest.json) | Latest accepted local/testnet-aligned receipt summary |
| [`evidence/mutation-results.json`](evidence/mutation-results.json) | Public-input mutation and limitation matrix |
| [`evidence/testnet-latest.json`](evidence/testnet-latest.json) | Contract id, transaction hashes, proof hash, VK hash, and final accepted state |
| [`evidence/testnet-links.md`](evidence/testnet-links.md) | Stellar Expert links for contract and transactions |
| [`evidence/auditor-receipt.json`](evidence/auditor-receipt.json) | Judge/auditor receipt export |

The [demo video](https://youtu.be/F5SKTwubxF0) is a recording of the actual web dashboard. It introduces the fallback architecture, runs the browser-side root and receipt comparison, and shows benchmark and on-chain evidence.

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
zkaggregator/
|-- contracts/
|   `-- batch_verifier/       Deployed Soroban batch-verification contract
|-- scripts/
|   |-- aggregator.mjs        Deterministic fixture and batch logic
|   |-- aggregator.test.mjs   Node.js integrity and mutation tests
|   `-- bench.mjs             N=2,4,8 benchmark generator
|-- frontend/
|   |-- assets/               Proofora visual assets
|   `-- index.html            Live testnet evidence dashboard
|-- evidence/                 Tool versions, manifests, testnet links, receipts, mutation results
|-- web/
|   `-- build.mjs             Static benchmark dashboard generator
|-- artifacts/
|   |-- benchmark.json        Latest benchmark result
|   |-- testnet-proofs-n2.json
|   `-- testnet-public-inputs-n2.json
|-- TESTNET_DEPLOYMENT.md     Deployment identifiers and transaction evidence
`-- package.json              JavaScript test, benchmark, and demo commands
```

The `aggregator/`, `sub_circuit/`, and `contracts/aggregator_verifier/` directories preserve experimental recursive-aggregation scaffolding. They are not part of the deployed fallback execution path documented above.

## Requirements

- Node.js 20 or newer.
- Rust toolchain compatible with the contract workspace.
- Stellar CLI with Soroban contract build support.
- The `wasm32v1-none` Rust target required by the Stellar contract toolchain.
- Noir/Nargo for the `circuits/subproof_shape_validator` shape-check circuit.

The JavaScript reference implementation uses Node.js built-in modules and has no runtime package dependency.

## WSL Ubuntu Setup

WSL Ubuntu is the canonical reproducibility path:

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev curl git jq unzip
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
rustup target add wasm32-unknown-unknown
rustup target add wasm32v1-none
curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -
sudo apt install -y nodejs
npm install -g corepack
corepack enable
cargo install --locked stellar-cli
stellar network add testnet --rpc-url https://soroban-testnet.stellar.org --network-passphrase "Test SDF Network ; September 2015"
cd /mnt/d/dorahack/stellar/zkaggregator
```

## Local Development

From the project root:

```bash
npm install
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

Run the mutation wrappers:

```bash
npx tsx tests/mutation/public-inputs.test.ts
npx tsx tests/mutation/replay-stale.test.ts
```

Check the Noir shape circuit:

```bash
cd circuits/subproof_shape_validator
nargo check
cd ../..
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
- rejection of a malformed proof envelope;
- rejection of an unregistered app id;
- rejection of duplicate nullifier replay;
- rejection of a zero retention proof.

The Node.js test suite additionally verifies valid batches for `N = 2`, `4`, and `8`, mutated public-input rejection, and incorrect-root rejection. The TypeScript mutation wrappers record the current public-input matrix in [`evidence/mutation-results.json`](evidence/mutation-results.json): `app_id`, `batch_root`, `retention_proof`, and `nullifier_hash` are contract-backed pass cases; `commitment_root`, `receipt_id`, and `ledger_sequence` are documented fallback limitations because the deployed fallback contract either derives them outside the contract or emits them as output rather than accepting them as directly mutable inputs.

## Testnet Reproduction Commands

Use only throwaway testnet identities. Stop before funding, secrets, private keys, or persistent credentials are required.

```bash
stellar keys generate rootvault-deployer --network testnet
stellar keys fund rootvault-deployer --network testnet
stellar contract build --manifest-path contracts/batch_verifier/Cargo.toml
stellar contract deploy --wasm contracts/target/wasm32v1-none/release/batch_verifier.wasm --source rootvault-deployer --network testnet
npx tsx scripts/testnet-e2e.ts
```

The checked-in evidence records the successful testnet path; the commands above are for reproducing with a fresh throwaway identity.

## Privacy And Trust Boundary

- **Private witness**: application-specific witness data remains outside RootVault.
- **Public inputs**: app id, commitment root, nullifier hash, batch root, receipt id, ledger sequence, retention proof.
- **Trusted actors**: Project operators, credential/oracle/root issuers, relayers, auditors, anchors, or policy administrators named by the implementation.
- **Mocked components**: Any oracle, credential issuer, asset adapter, relayer, or auditor service that is not live on Stellar testnet MUST be labeled in README and evidence.
- **Not private**: Contract ids, transaction hashes, verifier domain, public commitments, nullifiers, roots, receipt ids, and final accepted state are visible to judges.
- **Fallback limitations**: The deployed contract validates app registration, nullifier replay, proof-envelope shape, retention proof non-zeroness, root integrity, and checkpoint storage. It does not validate a production proof system or private application witness semantics.

## Security and Trust Model

The SHA-256 root provides deterministic integrity for an ordered set of public-input leaves. It allows the contract and clients to agree on exactly which inputs belong to a batch and detect mutation or reordering.

The current proof-envelope check is intentionally minimal: a proof must be non-empty and begin with a non-zero marker byte. Therefore, the fallback contract must not be treated as a production cryptographic proof verifier. A production version would replace this envelope predicate with verification of a real proof system while preserving the same batching interface and public-input commitment boundary.

The current implementation guarantees:

- proof and public-input array lengths match;
- the batch is non-empty;
- the app id is registered before acceptance;
- duplicate nullifier hashes are rejected for the same app;
- every submitted proof envelope passes the fallback shape check;
- the ordered public inputs reproduce the expected SHA-256 root;
- a non-zero retention proof is stored with the checkpoint;
- the returned receipt identifies the accepted root, item count, and receipt id.

It does not guarantee:

- recursive proof soundness;
- RISC Zero receipt verification on Soroban;
- Groth16 verification;
- semantic validity of arbitrary external proof formats;
- production-grade application authorization or application-specific state transitions beyond RootVault checkpoint/nullifier/receipt storage.

Those boundaries are explicit so that the deployed demo can be evaluated on what it actually proves: a working, testnet-backed `N-to-1` batch submission path with deterministic public-input integrity.
