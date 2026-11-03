//! A command run in the background leaves no process behind for this process to reap.
//!
//! This test is alone in its binary because the assertion is about every child of the process:
//! another test spawning its own children in parallel would make it meaningless.

use longcut_shell::ShellService;
use std::fs;
use std::path::Path;

/// Pid, parent pid and state of the process behind a `/proc/<pid>` directory.
fn process_entry(proc_dir: &Path) -> Option<(u32, u32, char)> {
    let pid: u32 = proc_dir.file_name()?.to_str()?.parse().ok()?;
    let stat = fs::read_to_string(proc_dir.join("stat")).ok()?;
    // The command name is parenthesised and may contain spaces, so the fields after it are
    // read from the last closing parenthesis on.
    let (_, fields) = stat.rsplit_once(')')?;
    let mut fields = fields.split_whitespace();
    let state = fields.next()?.chars().next()?;
    let parent_pid = fields.next()?.parse().ok()?;
    Some((pid, parent_pid, state))
}

/// Pid and state of every process whose parent is this process.
fn child_processes() -> Vec<(u32, char)> {
    let own_pid = std::process::id();
    fs::read_dir("/proc")
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| process_entry(&entry.path()))
        .filter(|(_, parent_pid, _)| *parent_pid == own_pid)
        .map(|(pid, _, state)| (pid, state))
        .collect()
}

#[test]
fn background_command_leaves_no_child_process_behind() {
    ShellService::run_async("true").unwrap();

    let children = child_processes();
    assert!(
        children.is_empty(),
        "processes still parented to this process (pid, state): {children:?}"
    );
}
