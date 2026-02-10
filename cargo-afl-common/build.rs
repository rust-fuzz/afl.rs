use std::process::Command;

fn main() {
    let output = Command::new("git").args(["submodule", "status"]).output();

    let output = match output {
        Ok(output) => output,
        Err(e) => {
            println!(
                "cargo:warning=git command not found or failed; skipping submodule check: {e}"
            );
            return;
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    // output starts with `-` means not initialized
    if stdout.lines().any(|line| line.starts_with('-')) {
        println!(
            "cargo:warning=Submodule may not be fully initialized. Try running: git submodule update --init"
        );
    }

    println!("cargo:rerun-if-changed=AFLplusplus");
}
