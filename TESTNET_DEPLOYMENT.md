# zkAggregator Testnet Deployment Evidence

Network: Stellar testnet  
Source account identity: `deployer`  
Contract ID: `CASSNKDNJJCSILZ7SEF25SP336AXEZRPGSBFAJSTHST5U3FPV5OG4RCY`  
WASM hash: `6bcc3d3107db34038815faaa1415d94420e79a084e44b444f128b5e5a96ed60e`

## Transactions

- Upload WASM: `1c0e1452093092a1ae606972e54a5ea646c36fbdbb4ee21845b69f6b3dea089d`
  - https://stellar.expert/explorer/testnet/tx/1c0e1452093092a1ae606972e54a5ea646c36fbdbb4ee21845b69f6b3dea089d
- Deploy contract: `fdc0aceef86b205ea35f57c9a1b48b7abd53eef46c8f5b68977eba8cbb9256fe`
  - https://stellar.expert/explorer/testnet/tx/fdc0aceef86b205ea35f57c9a1b48b7abd53eef46c8f5b68977eba8cbb9256fe
- Verify batch N=2: `d2cfe9051ee9e8114de49ce2f3ab82f6d0a38fb398f0a2e78daaa66f3d1925d4`
  - https://stellar.expert/explorer/testnet/tx/d2cfe9051ee9e8114de49ce2f3ab82f6d0a38fb398f0a2e78daaa66f3d1925d4

## On-Chain Return

`verify_many` returned:

```json
{"n":2,"root":"f818afd37a6dc3bc92fb44731011277006db4efa6e9023cd7468c02335d22a4d"}
```

## Limitation

This is the plan-approved batch verification fallback. It reduces transaction/application overhead from N calls to one batch call, but it is not RISC Zero recursive proof aggregation.
