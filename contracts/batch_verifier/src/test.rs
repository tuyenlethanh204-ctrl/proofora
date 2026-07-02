#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::{Bytes, BytesN, Env, Vec};

fn leaf(env: &Env, value: u8) -> BytesN<32> {
    BytesN::from_array(env, &[value; 32])
}

fn proof(env: &Env, value: u8) -> Bytes {
    Bytes::from_array(env, &[1, value])
}

fn setup(env: &Env) -> BatchVerifierClient<'_> {
    let contract_id = env.register(BatchVerifier, ());
    BatchVerifierClient::new(env, &contract_id)
}

#[test]
fn verifies_batch_and_returns_receipt() {
    let env = Env::default();
    let client = setup(&env);
    let leaves = Vec::from_array(&env, [leaf(&env, 1), leaf(&env, 2)]);
    let proofs = Vec::from_array(&env, [proof(&env, 1), proof(&env, 2)]);
    let root = client.compute_root(&leaves);
    let receipt = client.verify_many(&proofs, &leaves, &root);

    assert_eq!(receipt.n, 2);
    assert_eq!(receipt.root, root);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn rejects_mutated_batch_root() {
    let env = Env::default();
    let client = setup(&env);
    let leaves = Vec::from_array(&env, [leaf(&env, 1), leaf(&env, 2)]);
    let proofs = Vec::from_array(&env, [proof(&env, 1), proof(&env, 2)]);
    let wrong_root = BytesN::from_array(&env, &[9; 32]);

    client.verify_many(&proofs, &leaves, &wrong_root);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn rejects_mutated_proof() {
    let env = Env::default();
    let client = setup(&env);
    let leaves = Vec::from_array(&env, [leaf(&env, 1), leaf(&env, 2)]);
    let proofs = Vec::from_array(&env, [Bytes::from_array(&env, &[0, 1]), proof(&env, 2)]);
    let root = client.compute_root(&leaves);

    client.verify_many(&proofs, &leaves, &root);
}
