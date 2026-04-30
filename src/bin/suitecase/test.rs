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
#[allow(dead_code)]
struct SuiteResult {
    storage_name: String,
    test_name: String,
    cases: Vec<CaseResult>,
}

#[derive(Debug)]
struct CaseResult {
    name: String,
    status: CaseStatus,
    ms: u128,
    suite_test_name: Option<String>,
}

#[derive(Debug, Clone, Copy)]
enum CaseStatus {
    Pass,
    Fail,
}

#[derive(Debug)]
struct ParsedCase {
    name: String,
    status: CaseStatus,
    ms: u128,
}

#[derive(Debug)]
struct PanicInfo {
    thread_name: String,
    file: Option<String>,
    line: Option<u32>,
    message: String,
}

pub fn run(args: Vec<String>, output: OutputMode, workspace: bool, release: bool) {
    let (cargo_args, test_args) = split_at_double_dash(&args);

    let mut cmd = Command::new("cargo");
    cmd.arg("test");
    if workspace {
        cmd.arg("--workspace");
    }
    if release {
        cmd.arg("--release");
    }
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
    let mut current_storage: Option<String> = None;
    let mut current_test: Option<String> = None;
    let mut current_cases: Vec<CaseResult> = Vec::new();
    let mut regular_tests: Vec<CaseResult> = Vec::new();
    let all_stderr: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let mut total_results: usize = 0;

    let all_stderr_clone = Arc::clone(&all_stderr);
    let stderr_reader = std::thread::spawn(move || {
        for line in stderr.lines() {
            let line = line.expect("read stderr");
            all_stderr_clone.lock().unwrap().push(line);
        }
    });

    let is_github = matches!(output, OutputMode::Github);

    for line in stdout.lines() {
        let line = line.expect("read stdout");
        let trimmed = line.trim();

        if let Some(rest) = trimmed.strip_prefix("◆ ") {
            flush_suite(
                &mut suites,
                &mut current_test,
                &mut current_storage,
                &mut current_cases,
                is_github,
            );

            let parts: Vec<&str> = rest.splitn(2, ' ').collect();
            if parts.len() == 2 {
                current_storage = Some(parts[0].to_string());
                current_test = Some(parts[1].to_string());
                if is_github {
                    println!();
                    println!("::group::{}::{}", parts[0], parts[1]);
                } else {
                    println!(
                        "{BOLD}── {storage} :: {test} ──{RESET}",
                        storage = parts[0],
                        test = parts[1]
                    );
                }
            }
        } else if let Some(result) = parse_case_line(&line) {
            total_results += 1;
            let suite_name = current_test.clone();
            let storage_name = current_storage.clone();
            stream_case_result(
                &result,
                suite_name,
                storage_name,
                output,
                &mut current_cases,
            );
        } else if let Some(result) = parse_cargo_test_line(&line) {
            total_results += 1;
            let case_result = CaseResult {
                name: result.name.clone(),
                status: result.status,
                ms: result.ms,
                suite_test_name: None,
            };
            stream_regular_result(&case_result, output);
            regular_tests.push(case_result);
        } else if !trimmed.is_empty()
            && !trimmed.starts_with("▶ ")
            && !is_cargo_summary_line(trimmed)
        {
            stream_user_output(trimmed, output);
        }
    }

    flush_suite(
        &mut suites,
        &mut current_test,
        &mut current_storage,
        &mut current_cases,
        is_github,
    );

    stderr_reader.join().expect("stderr thread panicked");
    let stderr_lines = all_stderr.lock().unwrap().clone();

    let exit_status = child.wait().expect("wait for cargo test");

    if total_results == 0 && !exit_status.success() {
        eprintln!("\n{RED}{BOLD}Compilation failed{RESET}\n");
        for line in &stderr_lines {
            eprintln!("{}", line);
        }
        std::process::exit(exit_status.code().unwrap_or(1));
    }

    let suite_test_names: Vec<&str> = suites.iter().map(|s| s.test_name.as_str()).collect();
    let filtered_regular: Vec<&CaseResult> = regular_tests
        .iter()
        .filter(|t| !suite_test_names.contains(&t.name.as_str()))
        .collect();

    let mut all_cases: Vec<&CaseResult> = suites.iter().flat_map(|s| s.cases.iter()).collect();
    all_cases.extend(filtered_regular.iter().copied());

    let failed: Vec<&CaseResult> = all_cases
        .iter()
        .filter(|c| matches!(c.status, CaseStatus::Fail))
        .copied()
        .collect();

    let panics = parse_panics(&stderr_lines);

    match output {
        OutputMode::Tui => {
            print_tui_summary(&all_cases);
            if !failed.is_empty() {
                print_tui_failures(&failed, &panics);
            }
        }
        OutputMode::Github => {
            print_github_failure_details(&failed, &panics);
        }
    }

    if !exit_status.success() {
        std::process::exit(exit_status.code().unwrap_or(1));
    }
}

fn flush_suite(
    suites: &mut Vec<SuiteResult>,
    current_test: &mut Option<String>,
    current_storage: &mut Option<String>,
    current_cases: &mut Vec<CaseResult>,
    is_github: bool,
) {
    if let Some(test_name) = current_test.take() {
        if is_github {
            println!("::endgroup::");
        }
        suites.push(SuiteResult {
            storage_name: current_storage.take().unwrap_or_default(),
            test_name,
            cases: std::mem::take(current_cases),
        });
    }
}

fn stream_case_result(
    result: &ParsedCase,
    suite_test_name: Option<String>,
    storage_name: Option<String>,
    output: OutputMode,
    current_cases: &mut Vec<CaseResult>,
) {
    let display_name = match &storage_name {
        Some(storage) => format!("{}::{}", storage, result.name),
        None => result.name.clone(),
    };

    match output {
        OutputMode::Tui => match result.status {
            CaseStatus::Pass => println!("  {GREEN}✓{RESET} {} ({}ms)", display_name, result.ms),
            CaseStatus::Fail => println!("  {RED}✗{RESET} {} ({}ms)", display_name, result.ms),
        },
        OutputMode::Github => {
            println!(
                "  {} {} ({}ms)",
                status_label_short(&result.status),
                display_name,
                result.ms
            );
        }
    }
    current_cases.push(CaseResult {
        name: display_name,
        status: result.status,
        ms: result.ms,
        suite_test_name,
    });
}

fn stream_regular_result(result: &CaseResult, output: OutputMode) {
    match output {
        OutputMode::Tui => match result.status {
            CaseStatus::Pass => println!("  {GREEN}✓{RESET} {} ({}ms)", result.name, result.ms),
            CaseStatus::Fail => println!("  {RED}✗{RESET} {} ({}ms)", result.name, result.ms),
        },
        OutputMode::Github => {
            println!(
                "  {} {} ({}ms)",
                status_label_short(&result.status),
                result.name,
                result.ms
            );
        }
    }
}

fn stream_user_output(line: &str, output: OutputMode) {
    match output {
        OutputMode::Tui => println!("{DIM}  |{RESET} {}", line),
        OutputMode::Github => println!("  | {}", line),
    }
}

fn is_cargo_summary_line(line: &str) -> bool {
    line.starts_with("running ")
        || line.starts_with("test result:")
        || line == "failures:"
        || line.ends_with(" - should panic")
        || line.starts_with("all doctests")
        || line == "Couldn't compile the test."
        || line.starts_with("error: doctest failed")
        || line.contains("doctests ran in") && line.contains("merged doctests compilation")
        || (line.starts_with("src/") || line.starts_with("tests/")) && line.contains(" - ")
}

fn status_label_short(status: &CaseStatus) -> &'static str {
    match status {
        CaseStatus::Pass => "PASS",
        CaseStatus::Fail => "FAIL",
    }
}

fn split_at_double_dash(args: &[String]) -> (Vec<String>, Vec<String>) {
    let pos = args.iter().position(|a| a == "--");
    match pos {
        Some(i) => (args[..i].to_vec(), args[i + 1..].to_vec()),
        None => (args.to_vec(), Vec::new()),
    }
}

fn parse_cargo_test_line(line: &str) -> Option<ParsedCase> {
    let trimmed = line.trim();

    if let Some(rest) = trimmed.strip_prefix("test ")
        && let Some(name_end) = rest.find(" ... ")
    {
        let name = rest[..name_end].to_string();
        let result_part = &rest[name_end + 5..];
        let status = if result_part == "ok" {
            CaseStatus::Pass
        } else if result_part.starts_with("FAILED") {
            CaseStatus::Fail
        } else {
            return None;
        };
        return Some(ParsedCase {
            name,
            status,
            ms: 0,
        });
    }

    None
}

fn parse_case_line(line: &str) -> Option<ParsedCase> {
    let trimmed = line.trim();

    if let Some(rest) = trimmed.strip_prefix("✓ ")
        && let Some((name, ms)) = parse_timing(rest)
    {
        return Some(ParsedCase {
            name,
            status: CaseStatus::Pass,
            ms,
        });
    }

    if let Some(rest) = trimmed.strip_prefix("✗ ")
        && let Some((name, ms)) = parse_timing(rest)
    {
        return Some(ParsedCase {
            name,
            status: CaseStatus::Fail,
            ms,
        });
    }

    None
}

fn parse_timing(s: &str) -> Option<(String, u128)> {
    if let Some(pos) = s.rfind(" (") {
        let name = s[..pos].to_string();
        let timing = &s[pos + 2..];
        if let Some(end) = timing.find("ms)")
            && let Ok(ms) = timing[..end].parse::<u128>()
        {
            return Some((name, ms));
        }
    }
    None
}

fn parse_panics(stderr_lines: &[String]) -> Vec<PanicInfo> {
    let mut panics: Vec<PanicInfo> = Vec::new();
    let mut i = 0;

    while i < stderr_lines.len() {
        let line = &stderr_lines[i];

        if line.starts_with("thread ") && line.contains("panicked") {
            let (thread_name, file, line_num) = parse_panic_header(line);
            let mut message_parts: Vec<String> = Vec::new();

            i += 1;
            while i < stderr_lines.len() {
                let next = stderr_lines[i].trim();
                if next.is_empty()
                    || next.starts_with("note:")
                    || next.starts_with("error:")
                    || next.starts_with("stack backtrace:")
                    || next.starts_with("thread ")
                {
                    break;
                }
                message_parts.push(next.to_string());
                i += 1;
            }

            panics.push(PanicInfo {
                thread_name,
                file,
                line: line_num,
                message: message_parts.join("\n"),
            });
        } else {
            i += 1;
        }
    }

    panics
}

fn parse_panic_header(line: &str) -> (String, Option<String>, Option<u32>) {
    let thread_name = line
        .strip_prefix("thread '")
        .and_then(|rest| rest.find('\'').map(|end| rest[..end].to_string()))
        .unwrap_or_default();

    let (file, line_num) = if let Some(colon_pos) = line.find("panicked at ") {
        let after = &line[colon_pos + 12..];
        if let Some(colon) = after.rfind(':') {
            let path_and_line = &after[..colon];
            // Check for column number after line
            let path_and_line = if let Some(inner_colon) = path_and_line.rfind(':') {
                let maybe_col = &path_and_line[inner_colon + 1..];
                if maybe_col.bytes().all(|b| b.is_ascii_digit()) {
                    &path_and_line[..inner_colon]
                } else {
                    path_and_line
                }
            } else {
                path_and_line
            };

            if let Some(last_colon) = path_and_line.rfind(':') {
                let file = path_and_line[..last_colon].to_string();
                let line_num: u32 = path_and_line[last_colon + 1..].parse().unwrap_or(0);
                (Some(file), if line_num > 0 { Some(line_num) } else { None })
            } else {
                (Some(path_and_line.to_string()), None)
            }
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    (thread_name, file, line_num)
}

fn find_panic_for_suite<'a>(
    suite_test_name: &str,
    panics: &'a [PanicInfo],
) -> Option<&'a PanicInfo> {
    panics.iter().find(|p| p.thread_name == suite_test_name)
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

    let status_color = if failed > 0 { RED } else { GREEN };
    let status = if failed > 0 { "FAILED" } else { "PASSED" };

    println!(
        "\n{status_color}{BOLD}{status}{RESET}  {passed} passed, {failed} failed  (total: {total_ms}ms)"
    );
}

fn print_tui_failures(failed: &[&CaseResult], panics: &[PanicInfo]) {
    println!();
    println!("{RED}{BOLD}─── FAILURES ───{RESET}");
    println!();

    let mut shown_panics: std::collections::HashSet<String> = std::collections::HashSet::new();

    for case in failed {
        println!("{RED}{BOLD}✗ {name}{RESET}", name = case.name);
        println!("{DIM}  duration: {}ms{RESET}", case.ms);

        if let Some(suite_test_name) = &case.suite_test_name
            && let Some(panic) = find_panic_for_suite(suite_test_name, panics)
        {
            if !shown_panics.contains(suite_test_name) {
                shown_panics.insert(suite_test_name.clone());
                if let Some(ref file) = panic.file {
                    let line_info = if let Some(line) = panic.line {
                        format!("{}:{}", file, line)
                    } else {
                        file.clone()
                    };
                    println!("{DIM}  at {line_info}{RESET}");
                }
                println!("{RED}  {msg}{RESET}", msg = panic.message);
            }
            println!();
            continue;
        }

        println!("{DIM}  (no panic details captured){RESET}");
        println!();
    }

    println!("{RED}{BOLD}─── END FAILURES ───{RESET}");
}

fn print_github_failure_details(failed: &[&CaseResult], panics: &[PanicInfo]) {
    let mut shown_panics: std::collections::HashSet<String> = std::collections::HashSet::new();

    for case in failed {
        if let Some(suite_test_name) = &case.suite_test_name {
            if let Some(panic) = find_panic_for_suite(suite_test_name, panics)
                && !shown_panics.contains(suite_test_name)
            {
                shown_panics.insert(suite_test_name.clone());
                let file_info = match (&panic.file, panic.line) {
                    (Some(f), Some(l)) => format!("file={},line={},", f, l),
                    _ => String::new(),
                };
                let message = panic.message.replace('\n', " ");
                println!(
                    "::error {file_info}title={name}::{name} failed: {message}",
                    name = case.name,
                );
            }
        } else {
            println!("::error title={name}::{name} failed", name = case.name,);
        }
    }
}
