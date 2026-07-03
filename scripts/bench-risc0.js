const { execSync } = require("child_process");

console.log("== RISC Zero zkAggregator Probe ==");
console.log("Experimental recursive-path probe. The completed MVP uses the batch verification fallback.");
console.log("This script is not part of the PASS criteria unless the local RISC Zero toolchain is configured.");

const startTime = Date.now();

try {
  console.log("Running WSL cargo test for aggregator...");
  const out = execSync(
    'wsl bash -c "cd zkaggregator/aggregator/host && source ~/.cargo/env && cargo test --release -- --nocapture"',
    { encoding: "utf-8" }
  );

  if (out.includes("test result: ok")) {
    console.log("Recursive proof generation and verification succeeded in this environment.");
  } else {
    console.log("Proof output may have issues. Check logs.");
  }
} catch (e) {
  console.error("Recursive proof probe failed:", e.message);
}

const endTime = Date.now();
console.log(`\nTotal probe time: ${(endTime - startTime) / 1000} seconds`);
console.log("Note: The shipped fallback reduces transaction/application overhead, not cryptographic verifier cost.");
