use risc0_zkvm::guest::env;

fn main() {
    let value: u64 = env::read();
    let min: u64 = env::read();
    let max: u64 = env::read();
    assert!(value >= min && value <= max);
    env::commit(&(min, max));   // journal: public threshold
}
