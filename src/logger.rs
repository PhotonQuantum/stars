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

#[derive(Default)]
pub struct Logger {
    target: RefCell<LogTarget>,
}

impl Logger {
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
        self.println(format!("{}  {}", style("WARN").green(), msg));
    }
    pub fn error(&self, msg: impl Display) {
        self.println(format!("{} {}", style("ERROR").green(), msg));
    }
    pub fn println(&self, msg: impl Display) {
        match &*self.target.borrow() {
            LogTarget::Plain => println!("{}", msg),
            LogTarget::Progress(pb) => pb.println(msg.to_string()),
        }
    }
}
