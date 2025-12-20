//! Child process tracking for cleanup
//!
//! Tracks spawned child processes to ensure they can be cleaned up
//! when the shell exits or receives a signal.

use std::cell::RefCell;
use std::rc::Rc;

/// Tracks child processes spawned by the shell.
///
/// This is used to ensure all child processes are properly cleaned up
/// when the shell exits.
#[derive(Debug, Clone)]
pub struct ChildProcessTracker {
    inner: Rc<RefCell<ChildProcessTrackerInner>>,
}

#[derive(Debug, Default)]
struct ChildProcessTrackerInner {
    /// Process IDs of tracked children
    pids: Vec<u32>,
}

impl Default for ChildProcessTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ChildProcessTracker {
    /// Create a new process tracker.
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(ChildProcessTrackerInner::default())),
        }
    }

    /// Track a child process.
    pub fn track(&self, child: &tokio::process::Child) {
        if let Some(pid) = child.id() {
            self.inner.borrow_mut().pids.push(pid);
        }
    }

    /// Untrack a process by PID.
    pub fn untrack(&self, pid: u32) {
        self.inner.borrow_mut().pids.retain(|&p| p != pid);
    }

    /// Get all tracked PIDs.
    pub fn pids(&self) -> Vec<u32> {
        self.inner.borrow().pids.clone()
    }

    /// Kill all tracked processes.
    #[cfg(unix)]
    pub fn kill_all(&self) {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        let pids = self.pids();
        for pid in pids {
            let _ = kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
        }
    }

    /// Kill all tracked processes.
    #[cfg(windows)]
    pub fn kill_all(&self) {
        // On Windows, we rely on job objects or explicit termination
        // This is a simplified implementation
        let pids = self.pids();
        for pid in pids {
            // Windows process termination would go here
            // Using windows-sys TerminateProcess
            let _ = pid; // Suppress warning
        }
    }

    /// Kill all tracked processes (non-Unix/non-Windows fallback).
    #[cfg(not(any(unix, windows)))]
    pub fn kill_all(&self) {
        // No-op on unsupported platforms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_operations() {
        let tracker = ChildProcessTracker::new();
        assert!(tracker.pids().is_empty());

        // Can't easily test track() without spawning a real process,
        // but we can test the internal state
        tracker.inner.borrow_mut().pids.push(12345);
        assert_eq!(tracker.pids(), vec![12345]);

        tracker.untrack(12345);
        assert!(tracker.pids().is_empty());
    }

    #[test]
    fn test_tracker_clone() {
        let tracker1 = ChildProcessTracker::new();
        tracker1.inner.borrow_mut().pids.push(100);

        let tracker2 = tracker1.clone();
        tracker2.inner.borrow_mut().pids.push(200);

        // Both should see the same data (Rc sharing)
        assert_eq!(tracker1.pids(), vec![100, 200]);
        assert_eq!(tracker2.pids(), vec![100, 200]);
    }
}
