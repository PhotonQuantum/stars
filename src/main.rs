use clap::Parser;
use console::style;
use crate::args::Args;

mod args;
mod common;
mod homebrew;

fn main() {
    let args = Args::parse();
    println!("{:?}", args);
    println!("{}", style("Hello, world!").green());
}