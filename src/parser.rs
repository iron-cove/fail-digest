// Parses the plain-text output of `cargo test` and compatible runners,
// which print one line per test in the shape:
//
//   test some::module::test_name ... ok
//   test some::module::other_name ... FAILED
//
// For failed tests it also picks up the panic location and message from the
// `---- name stdout ----` sections libtest prints afterwards. Everything else
// in the log (compiler warnings, other captured stdout, the libtest summary
// line) is ignored.

pub struct Panic {
    pub location: String,
    pub message: Vec<String>,
}

pub struct Failure {
    pub name: String,
    pub panic: Option<Panic>,
}

pub struct Report {
    pub passed: Vec<String>,
    pub failed: Vec<Failure>,
    pub ignored: Vec<String>,
}

impl Report {
    pub fn print(&self) {
        if self.failed.is_empty() && self.passed.is_empty() && self.ignored.is_empty() {
            println!("no test lines found in input");
            return;
        }

        if self.failed.is_empty() {
            println!("all {} tests passed", self.passed.len());
            return;
        }

        println!(
            "{} failed, {} passed, {} ignored",
            self.failed.len(),
            self.passed.len(),
            self.ignored.len()
        );
        println!();
        for failure in &self.failed {
            println!("FAILED  {}", failure.name);
            if let Some(panic) = &failure.panic {
                if !panic.location.is_empty() {
                    println!("          at {}", panic.location);
                }
                for line in &panic.message {
                    println!("          {}", line);
                }
            }
        }
    }
}

// libtest prints the captured output of each failing test after the run, under
// a header like `---- name stdout ----`. The panic line inside that section is
// the only place the message and location appear, so we pair them up with the
// FAILED lines by test name afterwards.
fn collect_panics(input: &str) -> Vec<(String, Panic)> {
    let lines: Vec<&str> = input.lines().map(|l| l.trim_end()).collect();
    let mut found = Vec::new();
    let mut section: Option<&str> = None;

    for (i, line) in lines.iter().enumerate() {
        if let Some(name) = section_name(line) {
            section = Some(name);
            continue;
        }
        if *line == "failures:" {
            section = None;
            continue;
        }
        let Some(name) = section else {
            continue;
        };
        let Some(rest) = panic_rest(line) else {
            continue;
        };

        let panic = if let Some(old) = rest.strip_prefix('\'') {
            // Pre-1.73 layout: `'message', src/file.rs:10:5`
            match old.rsplit_once("', ") {
                Some((msg, loc)) => Panic {
                    location: loc.to_string(),
                    message: msg.lines().map(str::to_string).collect(),
                },
                None => Panic {
                    location: String::new(),
                    message: vec![old.to_string()],
                },
            }
        } else {
            // Current layout: location on this line, message on the lines
            // below it up to a blank line or the RUST_BACKTRACE note.
            let message = lines[i + 1..]
                .iter()
                .take_while(|l| !l.is_empty() && !l.starts_with("note: "))
                .map(|l| l.to_string())
                .collect();
            Panic {
                location: rest.strip_suffix(':').unwrap_or(rest).to_string(),
                message,
            }
        };

        found.push((name.to_string(), panic));
        // Later panics in the same section (e.g. from other threads) are
        // noise; the first one is what failed the test.
        section = None;
    }

    found
}

fn section_name(line: &str) -> Option<&str> {
    line.strip_prefix("---- ")?
        .strip_suffix(" stdout ----")
}

fn panic_rest(line: &str) -> Option<&str> {
    let after = line.strip_prefix("thread '")?;
    let (_, rest) = after.split_once("' panicked at ")?;
    Some(rest)
}

pub fn scan(input: &str) -> Report {
    let mut passed = Vec::new();
    let mut failed = Vec::new();
    let mut ignored = Vec::new();

    for raw_line in input.lines() {
        let line = raw_line.trim_end();
        let Some(rest) = line.strip_prefix("test ") else {
            continue;
        };
        let Some((name, outcome)) = rest.rsplit_once(" ... ") else {
            continue;
        };

        match outcome.trim() {
            "ok" => passed.push(name.to_string()),
            "FAILED" => failed.push(Failure {
                name: name.to_string(),
                panic: None,
            }),
            "ignored" => ignored.push(name.to_string()),
            _ => {}
        }
    }

    for (name, panic) in collect_panics(input) {
        if let Some(f) = failed
            .iter_mut()
            .find(|f| f.name == name && f.panic.is_none())
        {
            f.panic = Some(panic);
        }
    }

    Report {
        passed,
        failed,
        ignored,
    }
}
