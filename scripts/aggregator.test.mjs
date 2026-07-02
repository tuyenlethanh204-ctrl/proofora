import test from "node:test";
import assert from "node:assert/strict";
import { batchRoot, fixture, verifyBatch } from "./aggregator.mjs";

test("batch fallback verifies N=2,4,8", () => {
  for (const n of [2, 4, 8]) {
    const batch = fixture(n);
    const result = verifyBatch(batch.items, batch.batchRoot);
    assert.equal(result.ok, true);
    assert.equal(result.n, n);
  }
});

test("mutated public input is rejected", () => {
  const batch = fixture(4);
  batch.items[0].publicInput.value += 1;
  assert.deepEqual(verifyBatch(batch.items, batch.batchRoot), {
    ok: false,
    reason: "sub-proof failed"
  });
});

test("mutated batch root is rejected", () => {
  const batch = fixture(4);
  const wrongRoot = batchRoot(batch.items.slice(1).map((item) => item.leaf));
  const result = verifyBatch(batch.items, wrongRoot);
  assert.equal(result.ok, false);
  assert.equal(result.reason, "batch root mismatch");
});
