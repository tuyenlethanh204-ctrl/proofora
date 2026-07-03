#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Bytes, BytesN, Env, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    EmptyBatch = 1,
    ProofCountMismatch = 2,
    InvalidProof = 3,
    BatchRootMismatch = 4,
    AppNotRegistered = 5,
    DuplicateNullifier = 6,
    InvalidRetentionProof = 7,
    StaleRoot = 8,
    StalePolicy = 9,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchReceipt {
    pub root: BytesN<32>,
    pub n: u32,
    pub receipt_id: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppConfig {
    pub admin: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootCheckpoint {
    pub root: BytesN<32>,
    pub ledger_sequence: u32,
    pub retention_proof: BytesN<32>,
}

#[contract]
pub struct RootVaultContract;

#[contractimpl]
impl RootVaultContract {
    pub fn register_app(env: Env, app_id: BytesN<32>, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&app_id, &AppConfig { admin });
    }

    pub fn compute_root(env: Env, public_inputs: Vec<BytesN<32>>) -> Result<BytesN<32>, Error> {
        if public_inputs.is_empty() {
            return Err(Error::EmptyBatch);
        }
        let mut bytes = Bytes::new(&env);
        for leaf in public_inputs.iter() {
            bytes.extend_from_array(&leaf.to_array());
        }
        Ok(env.crypto().sha256(&bytes).to_bytes())
    }

    pub fn verify_many(
        env: Env,
        app_id: BytesN<32>,
        proofs: Vec<Bytes>,
        public_inputs: Vec<BytesN<32>>,
        expected_root: BytesN<32>,
        nullifier_hashes: Vec<BytesN<32>>,
        retention_proof: BytesN<32>,
    ) -> Result<BatchReceipt, Error> {
        // App must be registered
        if !env.storage().instance().has(&app_id) {
            return Err(Error::AppNotRegistered);
        }

        let n = public_inputs.len();
        if n == 0 {
            return Err(Error::EmptyBatch);
        }
        if proofs.len() != n {
            return Err(Error::ProofCountMismatch);
        }

        // Nullifier checks to prevent replay
        for nh in nullifier_hashes.iter() {
            let key = (app_id.clone(), nh.clone());
            if env.storage().persistent().has(&key) {
                return Err(Error::DuplicateNullifier);
            }
            env.storage().persistent().set(&key, &true);
        }

        for proof in proofs.iter() {
            if proof.is_empty() || proof.get_unchecked(0) == 0 {
                return Err(Error::InvalidProof);
            }
        }

        let actual_root = Self::compute_root(env.clone(), public_inputs)?;
        if actual_root != expected_root {
            return Err(Error::BatchRootMismatch);
        }

        // Check retention proof
        let empty_proof = BytesN::from_array(&env, &[0; 32]);
        if retention_proof == empty_proof {
            return Err(Error::InvalidRetentionProof);
        }

        let counter_key = (symbol_short!("Count"), app_id.clone());
        let receipt_id = env
            .storage()
            .persistent()
            .get::<_, u64>(&counter_key)
            .unwrap_or(0)
            + 1;
        env.storage()
            .persistent()
            .set(&counter_key, &receipt_id);

        // Store checkpoint
        let checkpoint = RootCheckpoint {
            root: expected_root.clone(),
            ledger_sequence: env.ledger().sequence(),
            retention_proof,
        };
        env.storage().persistent().set(&(app_id.clone(), receipt_id), &checkpoint);

        Ok(BatchReceipt {
            root: expected_root,
            n: n as u32,
            receipt_id,
        })
    }

    pub fn get_checkpoint(env: Env, app_id: BytesN<32>, receipt_id: u64) -> Option<RootCheckpoint> {
        env.storage().persistent().get(&(app_id, receipt_id))
    }
}

#[cfg(test)]
mod test;
