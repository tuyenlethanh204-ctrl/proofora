# zkAggregator Testnet Deployment Evidence

Network: Stellar testnet  
Source account identity: `rootvault-deployer`  
Contract ID: `CC6HXCAELE2M5W4ZLAIRVX2NQOOSUIJGBBFWYJRR5HT5FBGGGDYWPJJN`  
WASM hash: `fc034993c1c804c0f92b3910eed85f5fbb7b62da5f9ac20c26e941482f91367a`

## Transactions

- Upload WASM: `b8d631210d22064382ec179921a163cb1a47913aa716c1e1254c46bd8fb999c0`
  - https://stellar.expert/explorer/testnet/tx/b8d631210d22064382ec179921a163cb1a47913aa716c1e1254c46bd8fb999c0
- Deploy contract: `bcc31fba9673508da4122b882b1ad75fbebe5417be44ff0e2ea1381330f21e44`
  - https://stellar.expert/explorer/testnet/tx/bcc31fba9673508da4122b882b1ad75fbebe5417be44ff0e2ea1381330f21e44
- Register app: `b2e69b72b00ae2efb6d343c637534c2ba30777cb36f0f1fa7746b07989771c56`
  - https://stellar.expert/explorer/testnet/tx/b2e69b72b00ae2efb6d343c637534c2ba30777cb36f0f1fa7746b07989771c56
- Verify batch: `0e6c062dec34d48e7ba8b64e1bbb05ce23bb172ed1934c0e4992cdeb7a628667`
  - https://stellar.expert/explorer/testnet/tx/0e6c062dec34d48e7ba8b64e1bbb05ce23bb172ed1934c0e4992cdeb7a628667

## On-Chain Return

`verify_many` returned:

```json
{"n":1,"receipt_id":1,"root":"75877bb41d393b5fb8455ce60ecd8dda001d06316496b14dfa7f895656eeca4a"}
```

## Limitation

This is the plan-approved batch verification fallback. It reduces transaction/application overhead from N calls to one batch call, but it is not RISC Zero recursive proof aggregation.
