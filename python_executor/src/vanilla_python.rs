use std::io::Write;
use std::process::Command;

fn run_python_script(script: &str) -> Result<String, std::io::Error> {
    let mut child = Command::new("python")
        .arg("-c")
        .arg(script)
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_python_script() {
        let script = r#"
import numpy as np
print(np.array([1, 2, 3]).mean())
"#;

        match run_python_script(script) {
            Ok(output) => println!("Output: {}", output),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
