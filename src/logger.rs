use std::cell::RefCell;
use std::fmt::Display;

use console::{style, Term};
use indicatif::{ProgressBar, ProgressStyle};

enum LogTarget {
    Plain,
    Progress(
        ProgressBar,
        /* paused */ bool,
        /* max_message_len */ usize,
    ),
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
    /// Sets the logger's target to plain stdout.
    pub fn set_plain(&self) {
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
                LogTarget::Progress(pb, paused, _) if !paused => pb.println(msg.to_string()),
                _ => println!("{}", msg),
            }
        }
    }
    /// Pause background tick of progress bar.
    /// This is useful when you want to pause the progressbar redrawing and resume it later.
    pub fn pause_progress_bar(&self) {
        if let LogTarget::Progress(pb, paused, _) = &mut *self.target.borrow_mut() {
            pb.disable_steady_tick();
            Term::stderr().clear_last_lines(1).unwrap();
            *paused = true;
        }
    }
    /// Resume background tick of progress bar.
    /// This is useful when you want to resume the progressbar redrawing.
    pub fn resume_progress_bar(&self) {
        if let LogTarget::Progress(pb, paused, _) = &mut *self.target.borrow_mut() {
            println!();
            pb.enable_steady_tick(100);
            *paused = false;
        }
    }

    /// Set progress bar style to determinate.
    /// A new progress bar will be created if one does not exist.
    pub fn set_progress_bar_determinate(&self, max: u64) {
        self.with_progress_bar(|pb| {
            pb.set_length(max);
            pb.set_style(pb_style(0));
            pb.enable_steady_tick(100);
        });
    }

    /// Set progress bar style to spinner.
    /// A new progress bar will be created if one does not exist.
    pub fn set_progress_bar_spinner(&self) {
        self.with_progress_bar(|pb| {
            pb.set_style(
                indicatif::ProgressStyle::default_bar().template("{spinner:.green} {prefix} {msg}"),
            );
            pb.enable_steady_tick(100);
        });
    }

    /// Mutate progress bar.
    /// A new progress bar will be created if one does not exist.
    pub fn with_progress_bar(&self, f: impl FnOnce(&ProgressBar)) {
        if !self.quiet {
            let pb = self.acquire_progress_bar();
            f(&pb);
        }
    }

    pub fn set_prefix(&self, msg: impl Display) {
        self.with_progress_bar(|pb| pb.set_prefix(msg.to_string()));
    }

    pub fn set_message(&self, msg: impl Display) {
        if let LogTarget::Progress(pb, _, max_len) = &mut *self.target.borrow_mut() {
            let msg = msg.to_string();
            if pb.length() > 0 && msg.len() > *max_len {
                *max_len = msg.len();
                pb.set_style(pb_style(*max_len));
            }
            pb.set_message(msg);
        }
    }

    fn acquire_progress_bar(&self) -> ProgressBar {
        if let LogTarget::Progress(pb, _, _) = &*self.target.borrow() {
            return pb.clone();
        }
        let pb = ProgressBar::new(0);
        *self.target.borrow_mut() = LogTarget::Progress(pb.clone(), false, 0);
        pb
    }
}

fn pb_style(max_len: usize) -> ProgressStyle {
    ProgressStyle::default_bar()
        .template(
            format!(
                "{{spinner:.green}} {{prefix}} [{{wide_bar:.cyan/blue}}] {{pos}}/{{len}} ({{eta}}) {{msg:{}}}",
                max_len
            )
            .as_str(),
        )
        .progress_chars("#>-")
}
