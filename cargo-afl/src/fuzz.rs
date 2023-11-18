fn main() {
    if !cfg!(fuzzing) {
        panic!("Crash because fuzzing is not set. ");
    }
    println!("Running Normally");
}