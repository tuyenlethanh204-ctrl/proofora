import { createHash, randomBytes } from "node:crypto";
import { writeFileSync } from "node:fs";

export function sha256Hex(buffer: Buffer): string {
    return createHash("sha256").update(buffer).digest("hex");
}

export function leafForPublicInput(
    appId: string,
    commitmentRoot: string,
    nullifierHash: string,
    batchRoot: string,
    receiptId: string,
    ledgerSequence: string,
    retentionProof: string
): string {
    const payload = Buffer.from(
        `${appId}:${commitmentRoot}:${nullifierHash}:${batchRoot}:${receiptId}:${ledgerSequence}:${retentionProof}`,
        "utf8"
    );
    return sha256Hex(payload);
}

export function batchRoot(leaves: string[]): string {
    if (!Array.isArray(leaves) || leaves.length === 0) {
        throw new Error("batch needs at least one public input leaf");
    }
    return sha256Hex(Buffer.concat(leaves.map((leaf) => Buffer.from(leaf, "hex"))));
}

export function makeSubProof(
    appId: string,
    commitmentRoot: string,
    nullifierHash: string,
    batchRoot: string,
    receiptId: string,
    ledgerSequence: string,
    retentionProof: string
) {
    const publicInput = {
        appId,
        commitmentRoot,
        nullifierHash,
        batchRoot,
        receiptId,
        ledgerSequence,
        retentionProof
    };
    return {
        proof: Buffer.from([1, ...randomBytes(4)]).toString("hex"),
        publicInput,
        leaf: leafForPublicInput(appId, commitmentRoot, nullifierHash, batchRoot, receiptId, ledgerSequence, retentionProof)
    };
}

export function buildWitness(n: number = 2) {
    const appId = "app123";
    const batchRootVal = "mock_batch_root";
    const receiptId = "receipt_001";
    const ledgerSequence = "12345";
    const retentionProof = "proof_123";

    const items = [];
    for (let i = 0; i < n; i++) {
        items.push(
            makeSubProof(
                appId,
                `commitment_${i}`,
                `nullifier_${i}`,
                batchRootVal,
                receiptId,
                ledgerSequence,
                retentionProof
            )
        );
    }
    
    const expectedRoot = batchRoot(items.map((item) => item.leaf));
    
    const witness = {
        mode: "batch verification fallback with RootVault",
        appId,
        n,
        items,
        expectedRoot,
        nullifierHashes: items.map(i => sha256Hex(Buffer.from(i.publicInput.nullifierHash))),
        retentionProof: sha256Hex(Buffer.from(retentionProof))
    };

    writeFileSync("evidence/verifier-artifacts.json", JSON.stringify(witness, null, 2));
    console.log("Witness saved to evidence/verifier-artifacts.json");
}

if (import.meta.url === `file://${process.argv[1]}`) {
    buildWitness();
}
