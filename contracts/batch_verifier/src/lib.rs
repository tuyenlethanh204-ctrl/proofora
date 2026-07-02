#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Bytes, BytesN, Env, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    EmptyBatch = 1,
    ProofCountMismatch = 2,
    InvalidProof = 3,
    BatchRootMismatch = 4,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchReceipt {
    pub root: BytesN<32>,
    pub n: u32,
}

#[contract]
pub struct BatchVerifier;

#[contractimpl]
impl BatchVerifier {
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
        proofs: Vec<Bytes>,
        public_inputs: Vec<BytesN<32>>,
        expected_root: BytesN<32>,
    ) -> Result<BatchReceipt, Error> {
        let n = public_inputs.len();
        if n == 0 {
            return Err(Error::EmptyBatch);
        }
        if proofs.len() != n {
            return Err(Error::ProofCountMismatch);
        }

        for proof in proofs.iter() {
            if proof.is_empty() || proof.get_unchecked(0) == 0 {
                return Err(Error::InvalidProof);
            }
        }

        let actual_root = Self::compute_root(env, public_inputs)?;
        if actual_root != expected_root {
            return Err(Error::BatchRootMismatch);
        }

        Ok(BatchReceipt {
            root: expected_root,
            n,
        })
    }
}

mod test;
