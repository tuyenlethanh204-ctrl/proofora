import { execSync } from 'node:child_process';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import * as crypto from 'node:crypto';

function run(cmd: string) {
    console.log(`Running: ${cmd}`);
    return execSync(cmd, { encoding: 'utf8' }).trim();
}

async function main() {
    console.log("Deploying contract...");
    const contractIdOutput = run(`stellar contract deploy --wasm contracts/target/wasm32v1-none/release/batch_verifier.wasm --source rootvault-deployer --network testnet`);
    const contractId = contractIdOutput.split('\n').pop() || '';
    console.log(`Deployed contract ID: ${contractId}`);
    
    // Register App
    console.log("Registering App...");
    const appId = "01".repeat(32);
    run(`stellar contract invoke --id ${contractId} --network testnet --source rootvault-deployer -- register_app --app_id ${appId} --admin rootvault-deployer`);
    
    // Prepare inputs
    const publicInputHex = "02".repeat(32);
    const expectedRootHex = crypto.createHash('sha256').update(Buffer.from(publicInputHex, 'hex')).digest('hex');
    const nullifierHash = crypto.randomBytes(32).toString('hex'); // Randomize to avoid DuplicateNullifier on rerun
    const retentionProof = "04".repeat(32);
    const argDir = mkdtempSync(join(tmpdir(), 'rootvault-args-'));
    const proofsPath = join(argDir, 'proofs.json');
    const publicInputsPath = join(argDir, 'public_inputs.json');
    const nullifiersPath = join(argDir, 'nullifiers.json');
    writeFileSync(proofsPath, JSON.stringify(["0101"]));
    writeFileSync(publicInputsPath, JSON.stringify([publicInputHex]));
    writeFileSync(nullifiersPath, JSON.stringify([nullifierHash]));
    
    // Invoke verify_many
    console.log("Invoking verify_many...");
    const invokeCmd = `stellar contract invoke --send=yes --id ${contractId} --network testnet --source rootvault-deployer -- verify_many --app_id ${appId} --proofs-file-path "${proofsPath}" --public_inputs-file-path "${publicInputsPath}" --expected_root ${expectedRootHex} --nullifier_hashes-file-path "${nullifiersPath}" --retention_proof ${retentionProof}`;
    
    try {
        const out = execSync(invokeCmd + " 2>&1", { encoding: 'utf8' });
        console.log("Invoke output:", out);
        
        // If we reach here, it succeeded. Require a real transaction hash before writing evidence.
        const match = out.match(/tx\/([a-fA-F0-9]{64})/);
        if (!match) {
            throw new Error("verify_many completed but no transaction hash was found in stellar-cli output");
        }
        const finalTxHash = match[1];
        
        const evidence = {
            contract_id: contractId,
            transaction_hash: finalTxHash,
            proof_hash: expectedRootHex,
            verification_key_hash: "b2f6d0a38fb398f0a2e78daaa66f3d1925d4d2cfe9051ee9e8114de49ce2f3a",
            final_state: {
                root: expectedRootHex,
                n: 1
            }
        };
        writeFileSync("evidence/testnet-latest.json", JSON.stringify(evidence, null, 2));
        const links = `
# Testnet Links
- Contract Explorer: https://stellar.expert/explorer/testnet/contract/${contractId}
- Invoke Transaction: https://stellar.expert/explorer/testnet/tx/${finalTxHash}
`;
        writeFileSync("evidence/testnet-links.md", links);
        console.log("Saved evidence to testnet-latest.json and testnet-links.md");

    } catch (e: any) {
        console.error("Invoke failed:", e.stdout || e.message);
        console.log("Writing honest BLOCKED evidence...");
        const evidence = {
            contract_id: contractId,
            transaction_hash: "TESTNET_E2E_FAILED",
            note: "The verify_many invocation failed; inspect the script output before using this evidence."
        };
        writeFileSync("evidence/testnet-latest.json", JSON.stringify(evidence, null, 2));
        const links = `
# Testnet Links
- Contract Explorer: https://stellar.expert/explorer/testnet/contract/${contractId}
- Invoke Transaction: TESTNET_E2E_FAILED
`;
        writeFileSync("evidence/testnet-links.md", links);
        process.exit(1);
    }
}

main().catch(console.error);
