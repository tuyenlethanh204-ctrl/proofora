use sub_circuit_methods::{GUEST_ELF, GUEST_ID};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};

pub fn make_sub(value: u64, min: u64, max: u64) -> Receipt {
    let env = ExecutorEnv::builder()
        .write(&value).unwrap()
        .write(&min).unwrap()
        .write(&max).unwrap()
        .build()
        .unwrap();

    let prover = default_prover();
    let prove_info = prover.prove(env, GUEST_ELF).unwrap();
    prove_info.receipt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sub_receipt_verifies() {
        let receipt = make_sub(50, 0, 100);
        receipt.verify(GUEST_ID).unwrap();
    }
}
