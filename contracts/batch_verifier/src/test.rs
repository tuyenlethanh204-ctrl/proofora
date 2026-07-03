#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env, Vec};

fn leaf(env: &Env, value: u8) -> BytesN<32> {
    BytesN::from_array(env, &[value; 32])
}

fn proof(env: &Env, value: u8) -> Bytes {
    Bytes::from_array(env, &[1, value])
}

fn setup(env: &Env) -> (RootVaultContractClient<'_>, Address, BytesN<32>) {
    env.mock_all_auths();
    let contract_id = env.register(RootVaultContract, ());
    let client = RootVaultContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let app_id = BytesN::from_array(env, &[7; 32]);
    client.register_app(&app_id, &admin);
    (client, admin, app_id)
}

#[test]
fn verifies_batch_and_returns_receipt() {
    let env = Env::default();
    let (client, _admin, app_id) = setup(&env);
    let leaves = Vec::from_array(&env, [leaf(&env, 1), leaf(&env, 2)]);
    let proofs = Vec::from_array(&env, [proof(&env, 1), proof(&env, 2)]);
    let nullifier_hashes = Vec::from_array(&env, [leaf(&env, 11)]);
    let root = client.compute_root(&leaves);
    let retention_proof = BytesN::from_array(&env, &[1; 32]);
    let receipt = client.verify_many(&app_id, &proofs, &leaves, &root, &nullifier_hashes, &retention_proof);

    assert_eq!(receipt.n, 2);
    assert_eq!(receipt.root, root);
    assert_eq!(receipt.receipt_id, 1);

    let checkpoint = client.get_checkpoint(&app_id, &receipt.receipt_id).unwrap();
    assert_eq!(checkpoint.root, root);
    assert_eq!(checkpoint.retention_proof, retention_proof);

    // Replay should fail
    let res = client.try_verify_many(&app_id, &proofs, &leaves, &root, &nullifier_hashes, &retention_proof);
    assert_eq!(res.unwrap_err().unwrap(), Error::DuplicateNullifier);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn rejects_mutated_batch_root() {
    let env = Env::default();
    let (client, _admin, app_id) = setup(&env);
    let leaves = Vec::from_array(&env, [leaf(&env, 1), leaf(&env, 2)]);
    let proofs = Vec::from_array(&env, [proof(&env, 1), proof(&env, 2)]);
    let nullifier_hashes = Vec::from_array(&env, [leaf(&env, 11)]);
    let wrong_root = BytesN::from_array(&env, &[9; 32]);
    let retention_proof = BytesN::from_array(&env, &[1; 32]);

    client.verify_many(&app_id, &proofs, &leaves, &wrong_root, &nullifier_hashes, &retention_proof);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn rejects_mutated_proof() {
    let env = Env::default();
    let (client, _admin, app_id) = setup(&env);
    let leaves = Vec::from_array(&env, [leaf(&env, 1), leaf(&env, 2)]);
    let proofs = Vec::from_array(&env, [Bytes::from_array(&env, &[0, 1]), proof(&env, 2)]);
    let root = client.compute_root(&leaves);
    let nullifier_hashes = Vec::from_array(&env, [leaf(&env, 11)]);
    let retention_proof = BytesN::from_array(&env, &[1; 32]);

    client.verify_many(&app_id, &proofs, &leaves, &root, &nullifier_hashes, &retention_proof);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn rejects_zero_retention_proof() {
    let env = Env::default();
    let (client, _admin, app_id) = setup(&env);
    let leaves = Vec::from_array(&env, [leaf(&env, 1)]);
    let proofs = Vec::from_array(&env, [proof(&env, 1)]);
    let root = client.compute_root(&leaves);
    let nullifier_hashes = Vec::from_array(&env, [leaf(&env, 11)]);
    let retention_proof = BytesN::from_array(&env, &[0; 32]);

    client.verify_many(&app_id, &proofs, &leaves, &root, &nullifier_hashes, &retention_proof);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn rejects_unregistered_app_id() {
    let env = Env::default();
    let (client, _admin, _app_id) = setup(&env);
    let wrong_app_id = BytesN::from_array(&env, &[8; 32]);
    let leaves = Vec::from_array(&env, [leaf(&env, 1)]);
    let proofs = Vec::from_array(&env, [proof(&env, 1)]);
    let root = client.compute_root(&leaves);
    let nullifier_hashes = Vec::from_array(&env, [leaf(&env, 11)]);
    let retention_proof = BytesN::from_array(&env, &[1; 32]);

    client.verify_many(&wrong_app_id, &proofs, &leaves, &root, &nullifier_hashes, &retention_proof);
}
