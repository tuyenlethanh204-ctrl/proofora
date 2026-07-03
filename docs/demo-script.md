# Demo Video Script: Proofora RootVault

1. **Problem Statement**:
   - Briefly explain the problem: DApps on Stellar need to index nullifiers, store commitment roots, and issue receipts for ZK proofs without sending individual transactions for every proof.
   - Show the traditional 1-to-1 bottleneck on a diagram.
   
2. **Proof Generation (Local)**:
   - Open the live dashboard and click **Run fallback verification**.
   - Show the browser recomputing the recorded `N = 1` testnet root.
   - Confirm `receipt comparison: pass` and `Root matched`.

3. **On-chain Verification**:
   - Switch to the Stellar Testnet Explorer or the CLI.
   - Show the deployed RootVault contract `CC6HXCAELE2M5W4ZLAIRVX2NQOOSUIJGBBFWYJRR5HT5FBGGGDYWPJJN`.
   - Highlight the recorded `verify_many` transaction `0e6c062dec34d48e7ba8b64e1bbb05ce23bb172ed1934c0e4992cdeb7a628667`.

4. **Negative Test / Replay Rejection**:
   - Run `cargo test --manifest-path contracts/batch_verifier/Cargo.toml`.
   - Show the five passing Rust tests.
   - Highlight the replay assertion in `verifies_batch_and_returns_receipt`, plus mutated root, mutated proof, unknown app, and zero retention-proof rejection.

5. **Receipt and History**:
   - Show root `75877bb41d393b5fb8455ce60ecd8dda001d06316496b14dfa7f895656eeca4a`, batch size `1`, and `receipt_id = 1`.
   - Explain how indexers or auditors can use this receipt to independently verify the state transition on-chain.
