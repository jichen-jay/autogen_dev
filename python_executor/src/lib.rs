pub mod vanilla_python;

use anyhow::Result;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use tokio::time::timeout;
// use std::io::{BufRead, BufReader, Stdin, Stdout, Stderr, Read, Write};
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task;

pub async fn run_python_wrapper(code_wrapped_in_text: &str) -> (bool, String, String) {
    println!("raw code: {:?}\n\n", code_wrapped_in_text);
    let code = extract_code(code_wrapped_in_text);
    println!("clean code: {:?}\n\n", code.clone());

    let _ = save_py_to_disk("src/test.py", &code);

    match run_python_func_react("/Users/jichen/Projects/autogen_rust/src/test.py").await {
        Ok(success_result_text) => {
            println!("success: {:?}", success_result_text);

            (true, code, success_result_text)
        }
        Err(err_msg) => {
            println!("failure: {:?}", err_msg.to_string());

            (false, code, err_msg.to_string())
        }
    }
}

pub async fn run_python_func_react(func_path: &str) -> Result<String> {
    let mut cmd = Command::new("/Users/jichen/miniconda3/bin/python")
        .arg(func_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let mut stdin = cmd.stdin.take().unwrap();
    let mut stdout = cmd.stdout.take().unwrap();

    let (stdout_tx, mut stdout_rx) = mpsc::channel(100);

    let stdout_task = task::spawn(async move {
        let mut buffer = [0; 1024];
        loop {
            if let Ok(n) = stdout.read(&mut buffer) {
                if n == 0 {
                    break;
                } else {
                    let output = String::from_utf8_lossy(&buffer[0..n]).to_string();
                    let _ = stdout_tx.send(output).await;
                }
            }
        }
    });

    let mut combined_output = String::new();

    while let Some(block) = stdout_rx.recv().await {
        let input = block.trim().to_string();
        if input.is_empty() {
            break;
        }
        combined_output.push_str(&input);

        pretty_print_board(&input);

        match get_user_feedback().await {
            Ok(human_input) => {
                combined_output.push_str(&human_input);
                combined_output.push('\n');
                stdin.write_all(human_input.as_bytes()).unwrap();
                stdin.write_all(b"\n").unwrap();
            }
            Err(_) => {
                // println!("error: {:?}", e);
                break;
            }
        }
    }

    let _ = stdout_task.await;

    let status = cmd.wait()?;

    if status.success() {
        Ok(combined_output)
    } else {
        let mut stderr_output = String::new();
        if let Some(mut stderr) = cmd.stderr.take() {
            let _ = stderr.read_to_string(&mut stderr_output)?;
        }
        let final_error = format!("{}\n{}", combined_output, stderr_output);
        Err(anyhow::anyhow!("Error: {}", final_error))
    }
}

fn pretty_print_board(board: &str) {
    let lines: Vec<&str> = board.split('\n').collect();
    for line in lines {
        println!("{}", line);
    }
}
pub async fn run_python_func(func_path: &str) -> Result<String> {
    let mut cmd = Command::new("/Users/jichen/miniconda3/bin/python")
        .arg(func_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = cmd.stdout.take().unwrap();
    let mut stderr = cmd.stderr.take().unwrap();
    let mut stderr_output = String::new();
    let mut stdout_output = String::new();

    let mut stdout_lines = BufReader::new(stdout).lines();
    let _ = stderr.read_to_string(&mut stderr_output)?;

    while let Some(line) = stdout_lines.next() {
        stdout_output.push_str(&line?);
        stdout_output.push('\n');
    }

    let status = cmd.wait()?;

    if status.success() {
        Ok(stdout_output)
    } else {
        Err(anyhow::anyhow!("Error: {}", stderr_output))
    }
}

pub fn extract_code(text: &str) -> String {
    let multi_line_pattern = r"(?s)```python(.*?)```";
    let mut program = String::new();

    let multi_line_regex = Regex::new(multi_line_pattern).unwrap();
    for cap in multi_line_regex.captures_iter(text) {
        if let Some(code) = cap.get(1) {
            program.push_str(code.as_str().trim());
        }
    }

    program
}

pub fn extract_code_blocks(
    text: &str,
    detect_single_line_code: bool,
) -> Vec<(Option<String>, String)> {
    // Adjust regex pattern to handle both Unix and Windows line endings and optional language specifier
    let multi_line_pattern = r"```[ \t]*(\w+)?[ \t]*\r?\n(.*?)\r?\n[ \t]*```";
    let single_line_pattern = r"`([^`]+)`";
    let mut results: Vec<(Option<String>, String)> = Vec::new();

    let multi_line_regex = Regex::new(multi_line_pattern).unwrap();
    for cap in multi_line_regex.captures_iter(text) {
        let language = cap
            .get(1)
            .map_or(None, |m| Some(m.as_str().trim().to_string()));
        let code = cap.get(2).unwrap().as_str().trim().to_string();
        results.push((language.clone(), code.clone()));
        // println!("Matched multi-line code block: Language: {:?}, Code: {}", language, code);
    }

    if detect_single_line_code {
        let single_line_regex = Regex::new(single_line_pattern).unwrap();
        for cap in single_line_regex.captures_iter(text) {
            results.push((None, cap.get(1).unwrap().as_str().trim().to_string()));
            // println!("Matched single-line code: {}", cap.get(1).unwrap().as_str().trim());
        }
    }

    results
}
pub async fn get_user_feedback() -> Result<String> {
    // print!("User input: ");

    let mut input = String::new();
    let mut reader = BufReader::new(std::io::stdin());

    match timeout(Duration::from_secs(10), async {
        reader.read_line(&mut input).expect("Failed to read line");
        input
    })
    .await
    {
        Ok(mut input) => {
            if let Some('\n') = input.clone().chars().next_back() {
                input.pop();
            }
            if let Some('\r') = input.chars().next_back() {
                input.pop();
            }

            if input == "stop" {
                std::process::exit(0);
            }
            if input == "back" {
                return Err(anyhow::Error::msg("back to main"));
            }
            Ok(input)
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(0);
        }
    }
}

pub fn save_py_to_disk(path: &str, code: &str) -> Result<()> {
    let mut file = File::create(path).expect("Failed to create or open file");

    file.write_all(code.as_bytes())
        .expect("Failed to write code to file");

    Ok(())
}
// export DYLD_LIBRARY_PATH=/Users/jichen/miniconda3/lib:$DYLD_LIBRARY_PATH
// export PYO3_PYTHON=/Users/jichen/miniconda3/bin/python
// export DYLD_LIBRARY_PATH=/Users/jichen/miniconda3/lib
