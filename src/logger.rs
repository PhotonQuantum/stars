use std::cell::RefCell;
use std::fmt::Display;

use console::style;
use indicatif::ProgressBar;

pub enum LogTarget {
    Plain,
    Progress(ProgressBar),
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
    /// Sets the logger's target.
    /// If `target` is pointed to a `ProgressBar`, the logger will print messages above it and avoid
    /// them from being overwritten by the bar.
    pub fn set_target(&self, target: LogTarget) {
        *self.target.borrow_mut() = target;
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
                LogTarget::Plain => println!("{}", msg),
                LogTarget::Progress(pb) => pb.println(msg.to_string()),
            }
        }
    }
    /// Pause background tick of progress bar.
    /// This is useful when you want to pause the progressbar redrawing and resume it later.
    pub fn pause_tick(&self) {
        if let LogTarget::Progress(pb) = &*self.target.borrow() {
            pb.disable_steady_tick();
        }
    }
    /// Resume background tick of progress bar.
    /// This is useful when you want to resume the progressbar redrawing.
    pub fn resume_tick(&self) {
        if let LogTarget::Progress(pb) = &*self.target.borrow() {
            pb.enable_steady_tick(100);
        }
    }
}
