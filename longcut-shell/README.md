# longcut-shell

Longcut shell command execution. The module has no configuration section.

A synchronous command runs in its own process group with its output collected
in memory, and is polled for its outcome or killed as a whole. A background
command is detached from the longcut process and outlives it.
