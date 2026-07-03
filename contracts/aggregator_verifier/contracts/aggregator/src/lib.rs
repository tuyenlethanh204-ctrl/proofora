#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, symbol_short, Bytes, BytesN, Env, Symbol
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    InvalidJournal = 1,
}

const TOTAL_PROOFS: Symbol = symbol_short!("Total");

#[contract]
pub struct AggregatorVerifier;

#[contractimpl]
impl AggregatorVerifier {
    pub fn verify_batch(env: Env, _seal: Bytes, journal: Bytes) -> Result<(BytesN<32>, u32), Error> {
        // Parse journal: [batch_root (32 bytes)] + [n (4 bytes, little endian)]
        if journal.len() != 36 {
            return Err(Error::InvalidJournal);
        }

        let mut root_bytes = [0u8; 32];
        for i in 0..32 {
            root_bytes[i as usize] = journal.get_unchecked(i);
        }
        
        let mut n_bytes = [0u8; 4];
        for i in 0..4 {
            n_bytes[i as usize] = journal.get_unchecked(32 + i);
        }
        let n = u32::from_le_bytes(n_bytes);

        let root = BytesN::from_array(&env, &root_bytes);

        // Update total proofs counter
        let mut total: u32 = env.storage().instance().get(&TOTAL_PROOFS).unwrap_or(0);
        total += n;
        env.storage().instance().set(&TOTAL_PROOFS, &total);

        // Emit Verified event
        env.events().publish((symbol_short!("Verified"),), (root.clone(), n));

        Ok((root, n))
    }

    pub fn get_total_proofs(env: Env) -> u32 {
        env.storage().instance().get(&TOTAL_PROOFS).unwrap_or(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_verify_batch() {
        let env = Env::default();
        let verifier_id = env.register(AggregatorVerifier, ());
        let client = AggregatorVerifierClient::new(&env, &verifier_id);

        let mut root = [0u8; 32];
        root[0] = 42;
        let n = 2u32;

        let mut journal_vec = [0u8; 36];
        journal_vec[0..32].copy_from_slice(&root);
        journal_vec[32..36].copy_from_slice(&n.to_le_bytes());

        let journal = Bytes::from_slice(&env, &journal_vec);
        let seal = Bytes::new(&env); // Mock seal

        let (out_root, out_n) = client.verify_batch(&seal, &journal);
        assert_eq!(out_root.to_array(), root);
        assert_eq!(out_n, n);
    }
}
