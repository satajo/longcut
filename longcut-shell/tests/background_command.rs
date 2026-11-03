//! A command run in the background returns control before the whole script has run, and the
//! script runs in the same conditions as a command run to completion.

use longcut_shell::ShellService;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// A file the background command writes its result into. Removed when dropped, so a failed
/// assertion leaves nothing behind.
struct Marker {
    path: PathBuf,
}

impl Marker {
    fn new(test_name: &str) -> Self {
        let file_name = format!("longcut-shell-{}-{test_name}", std::process::id());
        Self {
            path: std::env::temp_dir().join(file_name),
        }
    }

    /// A script that runs the command and writes its output into the marker in one step, so a
    /// reader never sees the output half written.
    fn script_writing_output_of(&self, command: &str) -> String {
        let path = self.path.display();
        format!("{command} > '{path}.part' && mv '{path}.part' '{path}'")
    }

    /// The content of the marker once the background command has written it.
    fn wait_for_content(&self) -> Option<String> {
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(5) {
            if let Ok(content) = fs::read_to_string(&self.path) {
                return Some(content);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        None
    }
}

impl Drop for Marker {
    fn drop(&mut self) {
        // The marker is missing when the command never got to write it, and a panic here
        // while a failed assertion unwinds would abort the whole test binary.
        let _best_effort = fs::remove_file(&self.path);
    }
}

/// The bitmask of the `SigIgn:` line in the content of a `/proc/<pid>/status` file.
fn ignored_signals(status: &str) -> Option<u64> {
    let line = status.lines().find(|line| line.starts_with("SigIgn:"))?;
    let mask = line.split_whitespace().nth(1)?;
    u64::from_str_radix(mask, 16).ok()
}

#[test]
fn background_command_returns_before_the_whole_script_has_run() {
    let marker = Marker::new("multi-line");
    let script = format!("sleep 1\n{}", marker.script_writing_output_of("echo done"));

    let started = Instant::now();
    ShellService::run_async(&script).unwrap();
    let elapsed = started.elapsed();

    assert!(
        elapsed < Duration::from_millis(500),
        "run_async blocked for {elapsed:?}"
    );
    assert_eq!(marker.wait_for_content().unwrap(), "done\n");
}

#[test]
fn background_command_has_no_positional_parameters() {
    let marker = Marker::new("parameters");
    let script = marker.script_writing_output_of("echo \"$#:$1\"");

    ShellService::run_async(&script).unwrap();

    assert_eq!(marker.wait_for_content().unwrap(), "0:\n");
}

#[test]
fn background_command_ignores_no_signal_the_launching_process_does_not() {
    let marker = Marker::new("signals");
    let script = marker.script_writing_output_of("cat /proc/self/status");

    ShellService::run_async(&script).unwrap();

    let own_status = fs::read_to_string("/proc/self/status").unwrap();
    let own_ignores = ignored_signals(&own_status).unwrap();
    let background_status = marker.wait_for_content().unwrap();
    let background_ignores = ignored_signals(&background_status).unwrap();
    assert_eq!(
        background_ignores & !own_ignores,
        0,
        "the background command ignores {background_ignores:#x} while this process ignores {own_ignores:#x}"
    );
}
