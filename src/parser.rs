// Parses the plain-text output of `cargo test` and compatible runners,
// which print one line per test in the shape:
//
//   test some::module::test_name ... ok
//   test some::module::other_name ... FAILED
//
// Everything else in the log (compiler warnings, captured stdout, the
// libtest summary line) is ignored for now. That's enough to answer the
// one question this tool exists for: which tests failed.

pub struct Report {
    pub passed: Vec<String>,
    pub failed: Vec<String>,
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
        for name in &self.failed {
            println!("FAILED  {}", name);
        }
    }
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
            "FAILED" => failed.push(name.to_string()),
            "ignored" => ignored.push(name.to_string()),
            _ => {}
        }
    }

    Report {
        passed,
        failed,
        ignored,
    }
}
