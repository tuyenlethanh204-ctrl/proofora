import { mkdirSync, writeFileSync } from "node:fs";
import { performance } from "node:perf_hooks";
import { fixture, verifyBatch, verifySubProof } from "./aggregator.mjs";

function measure(fn, rounds = 250) {
  const start = performance.now();
  for (let i = 0; i < rounds; i += 1) fn();
  return (performance.now() - start) / rounds;
}

const rows = [2, 4, 8].map((n) => {
  const batch = fixture(n);
  const individualMs = measure(() => {
    for (const item of batch.items) verifySubProof(item);
  });
  const batchMs = measure(() => verifyBatch(batch.items, batch.batchRoot));
  return {
    n,
    individualTxCount: n,
    batchTxCount: 1,
    txReduction: `${n}x`,
    individualVerifyMs: Number(individualMs.toFixed(4)),
    batchVerifyMs: Number(batchMs.toFixed(4)),
    root: batch.batchRoot
  };
});

mkdirSync("artifacts", { recursive: true });
writeFileSync(
  "artifacts/benchmark.json",
  JSON.stringify(
    {
      mode: "batch verification fallback",
      note: "This benchmark measures local fallback verification and transaction-count reduction, not recursive proof compression.",
      rows
    },
    null,
    2
  )
);

console.table(rows);
console.log("wrote artifacts/benchmark.json");
