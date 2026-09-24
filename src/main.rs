use std::env;
use std::fs;
use std::io::{self, Read};

mod parser;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let input = match read_input(&args) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("faildigest: {}", err);
            std::process::exit(2);
        }
    };

    let report = parser::scan(&input);
    report.print();

    if !report.failed.is_empty() {
        std::process::exit(1);
    }
}

// With no paths we read stdin, so `cargo test 2>&1 | faildigest` works
// without the caller having to spell out a dash. A literal "-" still means
// stdin when it shows up alongside real paths, so stdin can be mixed with
// saved logs in one invocation.
fn read_input(paths: &[String]) -> io::Result<String> {
    if paths.is_empty() {
        return read_stdin();
    }

    let mut combined = String::new();
    for path in paths {
        let text = if path == "-" {
            read_stdin()?
        } else {
            fs::read_to_string(path)
                .map_err(|e| io::Error::new(e.kind(), format!("{}: {}", path, e)))?
        };
        combined.push_str(&text);
        if !combined.ends_with('\n') {
            combined.push('\n');
        }
    }
    Ok(combined)
}

fn read_stdin() -> io::Result<String> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}
