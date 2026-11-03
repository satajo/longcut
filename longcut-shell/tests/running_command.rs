//! A started command can be polled for its outcome without blocking, and killed as a whole.

use longcut_shell::{Poll, RunError, RunningCommand, ShellService};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const DEADLINE: Duration = Duration::from_secs(5);

/// Polls the command until it finishes or the deadline passes.
fn poll_to_completion(mut running: RunningCommand) -> Option<Result<String, RunError>> {
    let started = Instant::now();
    while started.elapsed() < DEADLINE {
        running = match running.poll() {
            Poll::Finished(result) => return Some(result),
            Poll::Running(running) => running,
        };
        std::thread::sleep(Duration::from_millis(5));
    }
    None
}

fn marker_path(test_name: &str) -> PathBuf {
    let file_name = format!("longcut-shell-running-{}-{test_name}", std::process::id());
    std::env::temp_dir().join(file_name)
}

/// The lines of the marker file once it holds the wanted number of them.
fn wait_for_lines(path: &Path, count: usize) -> Option<Vec<String>> {
    let started = Instant::now();
    while started.elapsed() < DEADLINE {
        if let Ok(content) = fs::read_to_string(path) {
            let lines: Vec<String> = content.lines().map(ToString::to_string).collect();
            if lines.len() == count {
                return Some(lines);
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    None
}

/// Whether the process is gone or has exited and only awaits reaping.
fn is_dead(pid: &str) -> bool {
    let Ok(stat) = fs::read_to_string(format!("/proc/{pid}/stat")) else {
        return true;
    };
    let state = stat
        .rsplit_once(')')
        .and_then(|(_, fields)| fields.split_whitespace().next());
    state == Some("Z")
}

fn wait_until_dead(pids: &[String]) -> bool {
    let started = Instant::now();
    while started.elapsed() < DEADLINE {
        if pids.iter().all(|pid| is_dead(pid)) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    false
}

#[test]
fn finished_command_reports_its_stdout() {
    let running = ShellService::start("echo 'Hello, world!'").unwrap();

    let result = poll_to_completion(running).unwrap();

    assert_eq!(result.unwrap(), "Hello, world!\n");
}

#[test]
fn failed_command_reports_stderr_as_the_details() {
    let running = ShellService::start("echo out; echo oops >&2; exit 3").unwrap();

    let result = poll_to_completion(running).unwrap();

    assert!(
        matches!(result, Err(RunError::Runtime(ref details)) if details == "oops\n"),
        "{result:?}"
    );
}

#[test]
fn failed_command_without_stderr_reports_stdout_then_the_status() {
    let with_stdout = poll_to_completion(ShellService::start("echo out; exit 1").unwrap()).unwrap();
    let silent = poll_to_completion(ShellService::start("exit 7").unwrap()).unwrap();

    assert!(
        matches!(with_stdout, Err(RunError::Runtime(ref details)) if details == "out\n"),
        "{with_stdout:?}"
    );
    assert!(
        matches!(silent, Err(RunError::Runtime(ref details)) if details.contains('7')),
        "{silent:?}"
    );
}

#[test]
fn leading_dash_is_part_of_the_command() {
    let result = poll_to_completion(ShellService::start("-n").unwrap()).unwrap();

    assert!(matches!(result, Err(RunError::Runtime(_))), "{result:?}");
}

#[test]
fn running_command_polls_as_running_until_it_finishes() {
    let running = ShellService::start("sleep 0.3").unwrap();

    let Poll::Running(running) = running.poll() else {
        panic!("the command finished before it could be polled");
    };
    let result = poll_to_completion(running).unwrap();

    assert_eq!(result.unwrap(), "");
}

#[test]
fn large_output_does_not_stall_the_command() {
    let running = ShellService::start("head -c 1000000 /dev/zero | tr '\\0' a").unwrap();

    let result = poll_to_completion(running).unwrap();

    assert_eq!(result.unwrap().len(), 1_000_000);
}

#[test]
fn kill_ends_every_process_the_command_spawned() {
    let marker = marker_path("group");
    let script = format!(
        "sleep 30 & echo $! > '{m}'; sleep 30 & echo $! >> '{m}'; wait",
        m = marker.display()
    );
    let running = ShellService::start(&script).unwrap();
    let pids = wait_for_lines(&marker, 2).unwrap();

    running.kill().unwrap();

    assert!(wait_until_dead(&pids), "processes still alive: {pids:?}");
    fs::remove_file(marker).unwrap();
}

#[test]
fn kill_ends_a_command_that_ignores_sigterm() {
    let marker = marker_path("sigterm");
    let script = format!("trap '' TERM; echo $$ > '{}'; sleep 30", marker.display());
    let running = ShellService::start(&script).unwrap();
    let pids = wait_for_lines(&marker, 1).unwrap();

    let asked = Instant::now();
    running.kill().unwrap();

    assert!(
        asked.elapsed() < Duration::from_secs(3),
        "kill took {:?}",
        asked.elapsed()
    );
    assert!(wait_until_dead(&pids), "process still alive: {pids:?}");
    fs::remove_file(marker).unwrap();
}

#[test]
fn kill_of_an_already_exited_command_succeeds() {
    let running = ShellService::start("true").unwrap();
    std::thread::sleep(Duration::from_millis(200));

    running.kill().unwrap();
}

#[test]
fn kill_ends_a_spawned_process_that_ignores_sigterm_after_the_shell_exits() {
    let marker = marker_path("orphan");
    let script = format!(
        "sh -c 'trap \"\" TERM; echo $$ > \"{}\"; sleep 30' & wait",
        marker.display()
    );
    let running = ShellService::start(&script).unwrap();
    let pids = wait_for_lines(&marker, 1).unwrap();

    running.kill().unwrap();

    assert!(wait_until_dead(&pids), "process still alive: {pids:?}");
    fs::remove_file(marker).unwrap();
}
