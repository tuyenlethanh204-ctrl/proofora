import { createHash } from "node:crypto";

export function sha256Hex(buffer) {
  return createHash("sha256").update(buffer).digest("hex");
}

export function leafForPublicInput({ proofId, value, min, max }) {
  return sha256Hex(Buffer.from(`${proofId}:${value}:${min}:${max}`, "utf8"));
}

export function batchRoot(leaves) {
  if (!Array.isArray(leaves) || leaves.length === 0) {
    throw new Error("batch needs at least one public input leaf");
  }
  return sha256Hex(Buffer.concat(leaves.map((leaf) => Buffer.from(leaf, "hex"))));
}

export function makeSubProof(proofId, value, min = 0, max = 100) {
  if (value < min || value > max) {
    throw new Error(`value ${value} outside [${min}, ${max}]`);
  }
  return {
    proof: Buffer.from([1, proofId, value, min, max]).toString("hex"),
    publicInput: { proofId, value, min, max },
    leaf: leafForPublicInput({ proofId, value, min, max })
  };
}

export function verifySubProof(item) {
  const proof = Buffer.from(item.proof, "hex");
  if (proof.length !== 5 || proof[0] !== 1) return false;
  const [, proofId, value, min, max] = proof;
  return (
    proofId === item.publicInput.proofId &&
    value === item.publicInput.value &&
    min === item.publicInput.min &&
    max === item.publicInput.max &&
    value >= min &&
    value <= max &&
    item.leaf === leafForPublicInput(item.publicInput)
  );
}

export function verifyBatch(items, expectedRoot) {
  if (!items.every(verifySubProof)) {
    return { ok: false, reason: "sub-proof failed" };
  }
  const actualRoot = batchRoot(items.map((item) => item.leaf));
  return {
    ok: actualRoot === expectedRoot,
    reason: actualRoot === expectedRoot ? "ok" : "batch root mismatch",
    actualRoot,
    expectedRoot,
    n: items.length
  };
}

export function fixture(n) {
  const values = [12, 24, 36, 48, 60, 72, 84, 96];
  const items = values.slice(0, n).map((value, index) => makeSubProof(index + 1, value));
  return {
    mode: "batch verification fallback",
    n,
    items,
    batchRoot: batchRoot(items.map((item) => item.leaf))
  };
}
