use std::cell::RefCell;
use std::fmt::Display;

use console::{style, Term};
use indicatif::ProgressBar;

enum LogTarget {
    Plain,
    Progress(ProgressBar, /* paused */ bool),
}

impl Default for LogTarget {
    fn default() -> Self {
        Self::Plain
    }
}

/// Global logger.
pub struct Logger {
    target: RefCell<LogTarget>,
    quiet: bool,
}

#[allow(dead_code)]
impl Logger {
    /// Creates a new logger. If `quiet` is `true`, the logger will not print anything.
    pub fn new(quiet: bool) -> Self {
        Self {
            target: RefCell::new(Default::default()),
            quiet,
        }
    }
    /// Sets the logger's target to a progressbar.
    ///
    /// Messages will print above the progressbar, avoiding them from being overwritten.
    /// Beware that text may output to stderr together with the bar.
    pub fn progress_bar(&self, pb: ProgressBar) {
        *self.target.borrow_mut() = LogTarget::Progress(pb, false);
    }
    /// Sets the logger's target to plain stdout.
    pub fn plain(&self) {
        *self.target.borrow_mut() = LogTarget::Plain;
    }
    /// Logs a debug message.
    pub fn debug(&self, msg: impl Display) {
        self.println(format!("{} {}", style("DEBUG").cyan(), msg));
    }
    /// Logs an info message.
    pub fn info(&self, msg: impl Display) {
        self.println(format!("{}  {}", style("INFO").green(), msg));
    }
    /// Logs a warning message.
    pub fn warn(&self, msg: impl Display) {
        self.println(format!("{}  {}", style("WARN").yellow(), msg));
    }
    /// Logs an error message.
    pub fn error(&self, msg: impl Display) {
        self.println(format!("{} {}", style("ERROR").red(), msg));
    }
    /// Logs a message.
    pub fn println(&self, msg: impl Display) {
        if !self.quiet {
            match &*self.target.borrow() {
                LogTarget::Progress(pb, paused) if !paused => pb.println(msg.to_string()),
                _ => println!("{}", msg),
            }
        }
    }
    /// Pause background tick of progress bar.
    /// This is useful when you want to pause the progressbar redrawing and resume it later.
    pub fn pause_progressbar(&self) {
        if let LogTarget::Progress(pb, paused) = &mut *self.target.borrow_mut() {
            pb.disable_steady_tick();
            Term::stderr().clear_last_lines(1).unwrap();
            *paused = true;
        }
    }
    /// Resume background tick of progress bar.
    /// This is useful when you want to resume the progressbar redrawing.
    pub fn resume_progressbar(&self) {
        if let LogTarget::Progress(pb, paused) = &mut *self.target.borrow_mut() {
            println!();
            pb.enable_steady_tick(100);
            *paused = false;
        }
    }
}
