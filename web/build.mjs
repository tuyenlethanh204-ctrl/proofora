import { mkdirSync, readFileSync, writeFileSync } from "node:fs";

let benchmark = { rows: [] };
try {
  benchmark = JSON.parse(readFileSync("artifacts/benchmark.json", "utf8"));
} catch {}

const rows = benchmark.rows
  .map(
    (row) => `<tr><td>${row.n}</td><td>${row.individualTxCount}</td><td>${row.batchTxCount}</td><td>${row.txReduction}</td><td>${row.batchVerifyMs} ms</td><td><code>${row.root.slice(0, 18)}...</code></td></tr>`
  )
  .join("");

mkdirSync("web/dist", { recursive: true });
writeFileSync(
  "web/dist/index.html",
  `<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>zkAggregator Evidence</title>
  <style>
    body { margin: 0; font-family: Inter, ui-sans-serif, system-ui, sans-serif; background: #f6f8fb; color: #172033; }
    main { max-width: 980px; margin: 0 auto; padding: 40px 20px; }
    h1 { font-size: 34px; margin: 0 0 8px; }
    p { line-height: 1.55; }
    table { width: 100%; border-collapse: collapse; margin-top: 24px; background: white; border: 1px solid #d8dee9; }
    th, td { text-align: left; padding: 12px; border-bottom: 1px solid #e5e9f0; }
    th { background: #1f6f5b; color: white; }
    code { font-size: 12px; }
    .status { display: inline-block; padding: 6px 10px; background: #dff7ea; color: #17613d; border: 1px solid #a8dfbd; border-radius: 6px; }
  </style>
</head>
<body>
  <main>
    <span class="status">Batch verification fallback</span>
    <h1>zkAggregator Evidence Dashboard</h1>
    <p>This MVP verifies a batch root and applies N proof-like receipts in one contract call. It does not claim RISC Zero recursive proof compression.</p>
    <table>
      <thead><tr><th>N</th><th>Individual tx</th><th>Batch tx</th><th>Tx reduction</th><th>Batch verify</th><th>Root</th></tr></thead>
      <tbody>${rows}</tbody>
    </table>
  </main>
</body>
</html>`
);
console.log("wrote web/dist/index.html");
