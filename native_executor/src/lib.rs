use std::ffi::OsStr;
use std::process::{Command, Stdio};

pub fn run_native_tool(
    command_and_pre_args: &str,
    payload: Option<String>,
    post_args: Option<String>
) -> Result<String, std::io::Error> {
    let mut full_command = String::from("sh -c '");
    full_command.push_str(command_and_pre_args);
    
    if let Some(p) = payload {
        full_command.push_str(" ");
        full_command.push_str(&p);
    }
    
    if let Some(args) = post_args {
        full_command.push_str(" ");
        full_command.push_str(&args);
    }
    
    full_command.push_str("'");

    let output = Command::new("sh")
        .arg("-c")
        .arg(&full_command)
        .stdout(Stdio::piped())
        .output()?;
    use std::ffi::OsStr;
    use std::process::{Command, Stdio};
    
    pub fn run_native_tool(
        command_and_pre_args: &str,
        payload: Option<String>,
        post_args: Option<String>
    ) -> Result<String, std::io::Error> {
        let mut full_command = String::from("sh -c '");
        full_command.push_str(command_and_pre_args);
        
        if let Some(p) = payload {
            full_command.push_str(" ");
            full_command.push_str(&p);
        }
        
        if let Some(args) = post_args {
            full_command.push_str(" ");
            full_command.push_str(&args);
        }
        
        full_command.push_str("'");
    
        let output = Command::new("sh")
            .arg("-c")
            .arg(&full_command)
            .stdout(Stdio::piped())
            .output()?;
    
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    #[cfg(test)]
    mod tests {
        use super::*;
    
        #[test]
        fn test_run_native_tool_cargo_tree() {
            let command = "cargo tree | grep";
            let payload = Some("serde".to_string());
            let args = Some("-C 3".to_string());
    
            match run_native_tool(command, payload, args) {
                Ok(output) => println!("Cargo tree output: {}", output),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    
        #[test]
        fn test_run_native_tool_ls() {
            let command = "ls";
            let payload = None;
            let args = Some("-l -a".to_string());
    
            match run_native_tool(command, payload, args) {
                Ok(output) => println!("ls output: {}", output),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_native_tool_cargo_tree() {
        let command = "cargo tree | grep";
        let payload = Some("serde".to_string());
        let args = Some("-C 3".to_string());

        match run_native_tool(command, payload, args) {
            Ok(output) => println!("Cargo tree output: {}", output),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    #[test]
    fn test_run_native_tool_ls() {
        let command = "ls";
        let payload = None;
        let args = Some("-l -a".to_string());

        match run_native_tool(command, payload, args) {
            Ok(output) => println!("ls output: {}", output),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}