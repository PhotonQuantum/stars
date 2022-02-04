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

pub struct Logger {
    target: RefCell<LogTarget>,
    quiet: bool
}

impl Logger {
    pub fn new(quiet: bool) -> Self {
        Self {
            target: RefCell::new(Default::default()),
            quiet
        }
    }
    pub fn set_target(&self, target: LogTarget) {
        *self.target.borrow_mut() = target;
    }
    pub fn debug(&self, msg: impl Display) {
        self.println(format!("{} {}", style("DEBUG").cyan(), msg));
    }
    pub fn info(&self, msg: impl Display) {
        self.println(format!("{}  {}", style("INFO").green(), msg));
    }
    pub fn warn(&self, msg: impl Display) {
        self.println(format!("{}  {}", style("WARN").yellow(), msg));
    }
    pub fn error(&self, msg: impl Display) {
        self.println(format!("{} {}", style("ERROR").red(), msg));
    }
    pub fn println(&self, msg: impl Display) {
        if !self.quiet {
            match &*self.target.borrow() {
                LogTarget::Plain => println!("{}", msg),
                LogTarget::Progress(pb) => pb.println(msg.to_string()),
            }
        }
    }
    /// Pause background tick of progress bar.
    pub fn pause_tick(&self) {
        if let LogTarget::Progress(pb) = &*self.target.borrow() {
            pb.disable_steady_tick();
        }
    }
    /// Resume background tick of progress bar.
    pub fn resume_tick(&self) {
        if let LogTarget::Progress(pb) = &*self.target.borrow() {
            pb.enable_steady_tick(100);
        }
    }
}
