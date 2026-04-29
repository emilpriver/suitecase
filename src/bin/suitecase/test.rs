use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

use crate::OutputMode;

const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

#[derive(Debug)]
struct CaseResult {
    name: String,
    status: CaseStatus,
    ms: u128,
    stderr: Vec<String>,
}

#[derive(Debug)]
enum CaseStatus {
    Pass,
    Fail,
}

pub fn run(args: Vec<String>, output: OutputMode) {
    let (cargo_args, test_args) = split_at_double_dash(&args);

    let mut cmd = Command::new("cargo");
    cmd.arg("test");
    for arg in &cargo_args {
        cmd.arg(arg);
    }
    cmd.arg("--");
    cmd.arg("--nocapture");
    for arg in &test_args {
        cmd.arg(arg);
    }

    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn cargo test");

    let stdout = BufReader::new(child.stdout.take().expect("no stdout"));
    let stderr = BufReader::new(child.stderr.take().expect("no stderr"));

    let mut cases: Vec<CaseResult> = Vec::new();
    let stderr_buf: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let all_stderr: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let stderr_buf_clone = Arc::clone(&stderr_buf);
    let all_stderr_clone = Arc::clone(&all_stderr);
    let stderr_reader = std::thread::spawn(move || {
        for line in stderr.lines() {
            let line = line.expect("read stderr");
            stderr_buf_clone.lock().unwrap().push(line.clone());
            all_stderr_clone.lock().unwrap().push(line);
        }
    });

    for line in stdout.lines() {
        let line = line.expect("read stdout");
        let trimmed = line.trim();

        if trimmed.starts_with("▶ ") {
            stderr_buf.lock().unwrap().clear();
        } else if let Some(result) = parse_case_line(&line) {
            let stderr_for_case = if output == OutputMode::Github {
                stderr_buf.lock().unwrap().clone()
            } else {
                Vec::new()
            };
            cases.push(CaseResult {
                name: result.name,
                status: result.status,
                ms: result.ms,
                stderr: stderr_for_case,
            });
        }
    }

    stderr_reader.join().expect("stderr thread panicked");
    let stderr_lines = all_stderr.lock().unwrap().clone();

    let exit_status = child.wait().expect("wait for cargo test");

    let failed: Vec<&CaseResult> = cases
        .iter()
        .filter(|c| matches!(c.status, CaseStatus::Fail))
        .collect();

    match output {
        OutputMode::Tui => {
            print_tui_summary(&cases);
            if !failed.is_empty() {
                print_tui_failures(&failed, &stderr_lines);
            }
        }
        OutputMode::Github => {
            print_github_actions_output(&cases);
        }
    }

    if !exit_status.success() {
        std::process::exit(1);
    }
}

fn split_at_double_dash(args: &[String]) -> (Vec<String>, Vec<String>) {
    let pos = args.iter().position(|a| a == "--");
    match pos {
        Some(i) => (args[..i].to_vec(), args[i + 1..].to_vec()),
        None => (args.to_vec(), Vec::new()),
    }
}

#[derive(Debug)]
struct ParsedCase {
    name: String,
    status: CaseStatus,
    ms: u128,
}

fn parse_case_line(line: &str) -> Option<ParsedCase> {
    let trimmed = line.trim();

    if let Some(rest) = trimmed.strip_prefix("✓ ") {
        if let Some((name, ms)) = parse_timing(rest) {
            return Some(ParsedCase {
                name,
                status: CaseStatus::Pass,
                ms,
            });
        }
    }

    if let Some(rest) = trimmed.strip_prefix("✗ ") {
        if let Some((name, ms)) = parse_timing(rest) {
            return Some(ParsedCase {
                name,
                status: CaseStatus::Fail,
                ms,
            });
        }
    }

    None
}

fn parse_timing(s: &str) -> Option<(String, u128)> {
    if let Some(pos) = s.rfind(" (") {
        let name = s[..pos].to_string();
        let timing = &s[pos + 2..];
        if let Some(end) = timing.find("ms)") {
            if let Ok(ms) = timing[..end].parse::<u128>() {
                return Some((name, ms));
            }
        }
    }
    None
}

fn print_tui_summary(cases: &[CaseResult]) {
    let passed = cases
        .iter()
        .filter(|c| matches!(c.status, CaseStatus::Pass))
        .count();
    let failed = cases
        .iter()
        .filter(|c| matches!(c.status, CaseStatus::Fail))
        .count();
    let total_ms: u128 = cases.iter().map(|c| c.ms).sum();

    println!();
    println!("{BOLD}Suitecase Summary{RESET}");
    println!("────────────────────────────────────────────────");

    for case in cases {
        match case.status {
            CaseStatus::Pass => {
                println!("  {GREEN}✓{RESET} {} ({}ms)", case.name, case.ms);
            }
            CaseStatus::Fail => {
                println!("  {RED}✗{RESET} {} ({}ms)", case.name, case.ms);
            }
        }
    }

    println!("────────────────────────────────────────────────");

    let status_color = if failed > 0 { RED } else { GREEN };
    let status = if failed > 0 { "FAILED" } else { "PASSED" };

    println!(
        "  {status_color}{BOLD}{status}{RESET}  {passed} passed, {failed} failed  (total: {total_ms}ms)"
    );
}

fn print_tui_failures(failed: &[&CaseResult], stderr_lines: &[String]) {
    println!();
    println!("{RED}{BOLD}─── FAILURES ───{RESET}");
    println!();

    for case in failed {
        let panic_info = extract_panic_for_case(&case.name, stderr_lines);

        println!("{RED}{BOLD}✗ {name}{RESET}", name = case.name);
        println!("{DIM}  duration: {}ms{RESET}", case.ms);

        if let Some(info) = panic_info {
            println!("{RED}  {msg}{RESET}", msg = info);
        } else {
            println!("{DIM}  (no panic details captured){RESET}");
        }
        println!();
    }

    println!("{RED}{BOLD}─── END FAILURES ───{RESET}");
}

fn print_github_actions_output(cases: &[CaseResult]) {
    for case in cases {
        let has_output = !case.stderr.is_empty();

        if has_output {
            println!("::group::{} {} ({}ms)", status_label(&case.status), case.name, case.ms);
        } else {
            print!("  {} {} ({}ms)", status_label(&case.status), case.name, case.ms);
        }

        for line in &case.stderr {
            println!("{}", line);
        }

        if has_output {
            println!("::endgroup::");
        } else {
            println!();
        }
    }
}

fn status_label(status: &CaseStatus) -> &'static str {
    match status {
        CaseStatus::Pass => "PASS",
        CaseStatus::Fail => "FAIL",
    }
}

fn extract_panic_for_case(_case_name: &str, stderr_lines: &[String]) -> Option<String> {
    let mut panic_lines: Vec<&str> = Vec::new();
    let mut in_panic = false;

    for line in stderr_lines {
        let trimmed = line.trim();

        if trimmed.starts_with("thread ") && trimmed.contains("panicked") {
            in_panic = true;
            if let Some(colon_pos) = trimmed.find("panicked at") {
                let after = &trimmed[colon_pos + 11..];
                if let Some(colon) = after.find(':') {
                    panic_lines.push(after[..colon].trim());
                }
            }
            continue;
        }

        if in_panic {
            if trimmed.is_empty()
                || trimmed.starts_with("note:")
                || trimmed.starts_with("error:")
                || trimmed.starts_with("stack backtrace:")
            {
                in_panic = false;
                continue;
            }
            if !trimmed.starts_with("thread") {
                panic_lines.push(trimmed);
            }
        }
    }

    if panic_lines.is_empty() {
        return None;
    }

    Some(panic_lines.join("\n  "))
}
