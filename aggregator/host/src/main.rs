use methods::{GUEST_ELF, GUEST_ID};
use sub_circuit_methods::GUEST_ID as SUB_ID;
use sub_circuit_host::make_sub;
use risc0_zkvm::{default_prover, ExecutorEnv};

fn main() {
    println!("Aggregator Host running");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn aggregates_two_proofs() {
        // Create two sub proofs
        let r1 = make_sub(50, 0, 100);
        let r2 = make_sub(70, 0, 100);

        let n = 2u32;
        let env = ExecutorEnv::builder()
            .add_assumption(r1.clone())
            .add_assumption(r2.clone())
            .write(&n).unwrap()
            .write(&SUB_ID).unwrap()
            .write(&r1.journal.bytes).unwrap()
            .write(&r2.journal.bytes).unwrap()
            .build()
            .unwrap();

        let prover = default_prover();
        let prove_info = prover.prove(env, GUEST_ELF).unwrap();
        prove_info.receipt.verify(GUEST_ID).unwrap();
        println!("Journal size: {}", prove_info.receipt.journal.bytes.len());
    }
}
