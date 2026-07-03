import { test } from 'node:test';
import assert from 'node:assert';
import { execSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';

test('Run cargo tests for batch_verifier mutation tests', async () => {
    let output = '';
    let success = false;
    try {
        console.log('Running real contract mutation tests via cargo test...');
        output = execSync(
            'bash -l -c "cargo test --manifest-path contracts/batch_verifier/Cargo.toml -- --nocapture"',
            { encoding: 'utf8', stdio: 'pipe' }
        );
        success = true;
    } catch (e: any) {
        output = e.stdout + '\n' + e.stderr;
        success = false;
    }

    // The Rust tests cover:
    // - rejects_mutated_batch_root
    // - rejects_mutated_proof
    // - rejects_zero_retention_proof
    
    assert.match(output, /test test::rejects_unregistered_app_id.*ok/);
    assert.match(output, /test test::rejects_mutated_batch_root.*ok/);
    assert.match(output, /test test::rejects_mutated_proof.*ok/);
    assert.match(output, /test test::rejects_zero_retention_proof.*ok/);
    assert.match(output, /test test::verifies_batch_and_returns_receipt.*ok/);
    
    assert.strictEqual(success, true, 'Cargo tests failed');

    let existing: any[] = [];
    try { existing = JSON.parse(readFileSync("evidence/mutation-results.json", "utf8")); } catch(e) {}
    
    // Add explicitly covered items
    existing = existing.filter(x => !["app_id", "batch_root", "nullifier_hash", "retention_proof", "commitment_root", "receipt_id", "ledger_sequence"].includes(x.input));
    
    existing.push({ input: "app_id", result: "pass", mechanism: "cargo test (rejects_unregistered_app_id)" });
    existing.push({ input: "batch_root", result: "pass", mechanism: "cargo test (rejects_mutated_batch_root)" });
    existing.push({ input: "retention_proof", result: "pass", mechanism: "cargo test (rejects_zero_retention_proof)" });
    existing.push({ input: "commitment_root", result: "limitation", note: "Not natively verified by fallback contract; verified at aggregation layer." });
    existing.push({ input: "receipt_id", result: "limitation", note: "Generated as output by fallback contract, cannot be mutated as input." });
    existing.push({ input: "ledger_sequence", result: "limitation", note: "Generated as output by fallback contract, cannot be mutated as input." });
    
    writeFileSync("evidence/mutation-results.json", JSON.stringify(existing, null, 2));
});
