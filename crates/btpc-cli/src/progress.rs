use std::sync::Mutex;
use std::time::{Duration, Instant};

use btpc_core::create::{HashProgress, ProgressSink};
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};

use crate::context::ProgressPolicy;

const UPDATE_INTERVAL: Duration = Duration::from_millis(100);

// Spec: CLI-PROGRESS-001
#[derive(Debug)]
struct ProgressState {
    last_draw: Option<Instant>,
    last_bytes: u64,
}

pub(crate) struct CliProgress {
    bar: ProgressBar,
    enabled: bool,
    state: Mutex<ProgressState>,
}

impl CliProgress {
    pub(crate) fn new(policy: ProgressPolicy, label: &'static str) -> Self {
        Self::with_draw_target(policy, label, ProgressDrawTarget::stderr())
    }

    fn with_draw_target(
        policy: ProgressPolicy,
        label: &'static str,
        draw_target: ProgressDrawTarget,
    ) -> Self {
        let enabled = policy == ProgressPolicy::Enabled;
        let bar = if enabled {
            let bar = ProgressBar::with_draw_target(None, draw_target);
            bar.set_style(
                ProgressStyle::with_template("{spinner:.cyan} {msg} [{bar:32.cyan/dim}] {bytes}/{total_bytes} {bytes_per_sec}")
                    .expect("static progress template is valid")
                    .progress_chars("=>-"),
            );
            bar.set_message(label);
            bar
        } else {
            ProgressBar::hidden()
        };
        Self {
            bar,
            enabled,
            state: Mutex::new(ProgressState {
                last_draw: None,
                last_bytes: 0,
            }),
        }
    }

    fn update(&self, progress: HashProgress, now: Instant) {
        if !self.enabled {
            return;
        }
        let mut state = self.state.lock().expect("progress mutex poisoned");
        let final_update = progress.bytes_hashed() >= progress.total_bytes();
        let due = state
            .last_draw
            .is_none_or(|last| now.duration_since(last) >= UPDATE_INTERVAL);
        if !due && !final_update {
            return;
        }
        state.last_draw = Some(now);
        state.last_bytes = progress.bytes_hashed();
        self.bar.set_length(progress.total_bytes());
        self.bar.set_position(progress.bytes_hashed());
    }

    #[cfg(test)]
    fn last_bytes(&self) -> u64 {
        self.state
            .lock()
            .expect("progress mutex poisoned")
            .last_bytes
    }
}

impl ProgressSink for CliProgress {
    fn on_progress(&self, progress: HashProgress) {
        self.update(progress, Instant::now());
    }
}

impl Drop for CliProgress {
    fn drop(&mut self) {
        self.bar.finish_and_clear();
    }
}

#[cfg(test)]
mod tests {
    use super::{CliProgress, UPDATE_INTERVAL};
    use crate::context::ProgressPolicy;
    use btpc_core::create::HashProgress;
    use indicatif::{InMemoryTerm, ProgressDrawTarget};
    use std::time::Instant;

    #[test]
    fn progress_throttles_intermediate_updates_but_always_draws_completion() {
        let progress = CliProgress::new(ProgressPolicy::Enabled, "testing");
        let start = Instant::now();
        progress.update(HashProgress::new(1, 100, 0), start);
        progress.update(HashProgress::new(2, 100, 0), start + UPDATE_INTERVAL / 2);
        assert_eq!(progress.last_bytes(), 1);
        progress.update(HashProgress::new(100, 100, 1), start + UPDATE_INTERVAL / 2);
        assert_eq!(progress.last_bytes(), 100);
    }

    #[test]
    fn disabled_progress_ignores_callbacks() {
        let progress = CliProgress::new(ProgressPolicy::Disabled, "testing");
        progress.update(HashProgress::new(100, 100, 1), Instant::now());
        assert_eq!(progress.last_bytes(), 0);
    }

    #[test]
    fn enabled_progress_draws_and_clears_on_drop() {
        let terminal = InMemoryTerm::new(3, 80);
        {
            let progress = CliProgress::with_draw_target(
                ProgressPolicy::Enabled,
                "testing",
                ProgressDrawTarget::term_like(Box::new(terminal.clone())),
            );
            progress.update(HashProgress::new(50, 100, 1), Instant::now());
            assert!(terminal.contents().contains("testing"));
        }
        assert_eq!(terminal.contents(), "");
    }
}
