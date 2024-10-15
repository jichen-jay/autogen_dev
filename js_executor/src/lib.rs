use std::io::Write;
use std::process::Command;

fn run_javascript(script: &str) -> Result<String, std::io::Error> {
    let mut child = Command::new("deno")
        .arg("eval")
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
    fn test_javascript() {
        let script = r#"
const numbers = [1, 2, 3];
const mean = numbers.reduce((sum, num) => sum + num, 0) / numbers.length;
console.log(mean);
"#;

        match run_javascript(script) {
            Ok(output) => {
                println!("Output: {}", output);
                assert_eq!(output.trim(), "2", "Expected mean of [1, 2, 3] to be 2");
            },
            Err(e) => {
                eprintln!("Error: {}", e);
                panic!("JavaScript execution failed");
            },
        }
    }
}