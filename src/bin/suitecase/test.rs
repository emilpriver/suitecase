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
struct SuiteResult {
    name: String,
    cases: Vec<CaseResult>,
    stderr: Vec<String>,
}

#[derive(Debug)]
struct CaseResult {
    name: String,
    status: CaseStatus,
    ms: u128,
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

    let mut suites: Vec<SuiteResult> = Vec::new();
    let mut current_suite: Option<String> = None;
    let mut current_cases: Vec<CaseResult> = Vec::new();
    let all_stderr: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let all_stderr_clone = Arc::clone(&all_stderr);
    let stderr_reader = std::thread::spawn(move || {
        for line in stderr.lines() {
            let line = line.expect("read stderr");
            all_stderr_clone.lock().unwrap().push(line);
        }
    });

    for line in stdout.lines() {
        let line = line.expect("read stdout");
        let trimmed = line.trim();

        if let Some(suite_name) = trimmed.strip_prefix("◆ ") {
            if let Some(name) = current_suite.take() {
                suites.push(SuiteResult {
                    name,
                    cases: std::mem::take(&mut current_cases),
                    stderr: Vec::new(),
                });
            }
            current_suite = Some(suite_name.to_string());
        } else if let Some(result) = parse_case_line(&line) {
            current_cases.push(CaseResult {
                name: result.name,
                status: result.status,
                ms: result.ms,
            });
        }
    }

    if let Some(name) = current_suite.take() {
        suites.push(SuiteResult {
            name,
            cases: std::mem::take(&mut current_cases),
            stderr: Vec::new(),
        });
    }

    stderr_reader.join().expect("stderr thread panicked");
    let stderr_lines = all_stderr.lock().unwrap().clone();

    if output == OutputMode::Github {
        let failed_suite_names: Vec<String> = suites
            .iter()
            .filter(|s| s.cases.iter().any(|c| matches!(c.status, CaseStatus::Fail)))
            .map(|s| s.name.clone())
            .collect();

        for suite in &mut suites {
            if failed_suite_names.contains(&suite.name) {
                suite.stderr = extract_all_stderr(&stderr_lines);
            }
        }
    }

    let exit_status = child.wait().expect("wait for cargo test");

    let all_cases: Vec<&CaseResult> = suites.iter().flat_map(|s| s.cases.iter()).collect();
    let failed: Vec<&CaseResult> = all_cases
        .iter()
        .filter(|c| matches!(c.status, CaseStatus::Fail))
        .copied()
        .collect();

    match output {
        OutputMode::Tui => {
            print_tui_summary(&all_cases);
            if !failed.is_empty() {
                print_tui_failures(&failed, &stderr_lines);
            }
        }
        OutputMode::Github => {
            print_github_actions_output(&suites);
        }
    }

    if !exit_status.success() {
        std::process::exit(1);
    }
}

fn extract_all_stderr(stderr_lines: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    let mut in_panic = false;

    for line in stderr_lines {
        if line.starts_with("thread ") && line.contains("panicked") {
            in_panic = true;
        }

        if in_panic {
            if line.starts_with("error: test failed") || line.starts_with("failures:") {
                break;
            }
            result.push(line.clone());
        }
    }

    result
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

fn print_tui_summary(cases: &[&CaseResult]) {
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

fn print_github_actions_output(suites: &[SuiteResult]) {
    for suite in suites {
        for case in &suite.cases {
            println!("  {} {}::{} ({}ms)", status_label(&case.status), suite.name, case.name, case.ms);
        }

        for line in &suite.stderr {
            println!("{}", line);
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
