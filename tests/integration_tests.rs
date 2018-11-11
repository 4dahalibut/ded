use std::env;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::process;
use std::thread;

use std::io::Write;

fn ded_command() -> process::Command {
    let root = env::current_exe()
        .unwrap()
        .parent()
        .expect("executable's directory")
        .to_path_buf();
    let path = root.join("../ded");
    process::Command::new(&path)
}

fn sed_command() -> process::Command {
    process::Command::new("sed")
}

fn expect_success(cmd: &process::Command, o: process::Output) -> process::Output {
    if !o.status.success() {
        let suggest = if o.stderr.is_empty() {
            "\n\nDid your search end up with no results?".to_string()
        } else {
            "".to_string()
        };

        panic!(
            "\n\n==========\n\
             command failed but expected success!\
             {}\
             \n\ncommand: {:?}\
             \n\nstatus: {}\
             \n\nstdout: {}\
             \n\nstderr: {}\
             \n\n==========\n",
            suggest,
            cmd,
            o.status,
            String::from_utf8_lossy(&o.stdout),
            String::from_utf8_lossy(&o.stderr)
        );
    }
    o
}

pub fn pipe(cmd: &mut process::Command, input: &str) -> process::Output {
    cmd.stdin(process::Stdio::piped());
    cmd.stdout(process::Stdio::piped());
    cmd.stderr(process::Stdio::piped());

    let mut child = cmd.spawn().unwrap();

    // Pipe input to child process using a separate thread to avoid
    // risk of deadlock between parent and child process.
    let mut stdin = child.stdin.take().expect("expected standard input");
    let input = input.to_owned();
    let worker = thread::spawn(move || write!(stdin, "{}", input));

    let output = expect_success(cmd, child.wait_with_output().unwrap());
    worker.join().unwrap().unwrap();
    output
}

pub fn run(mut cmd: process::Command, argstring: &str, input: &str) -> String {
    let mut cmd = cmd.arg("-e ".to_string() + argstring);
    let output = pipe(&mut cmd, input);
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.parse().unwrap()
}

pub fn compare_with_sed(argstring: String, input: &str) {
    assert_eq!(
        run(sed_command(), &argstring, input),
        run(ded_command(), &argstring, input)
    );
}

#[test]
fn test_from_file() {
    let file = File::open("valid-tests").unwrap();
    let input: String = "yoohoo\nyeehee\n".to_string();
    for line in BufReader::new(file).lines() {
        print!("Running command: {}...", line.as_ref().unwrap());
        compare_with_sed(line.unwrap(), &input);
        println!("Pass!");
    }
}
