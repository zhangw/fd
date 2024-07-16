use std::env;
use std::path::PathBuf;

use clap::Parser;
use fd_lib::cli::{ColorWhen, FileType, Opts};
use fd_lib::filter::SizeFilter;
use fd_lib::scan;

pub fn main() {
    let mut opts = Opts::parse_from(vec![""]);
    // --show-progress
    opts.show_progress = true;
    // -I
    opts.no_ignore = true;
    // --color never
    opts.color = ColorWhen::Never;
    // -g
    opts.glob = true;
    // --size +1m
    let more_than_1m = SizeFilter::from_string("+1m").expect("Failed to parse size filter");
    let size_filters = vec![more_than_1m];
    opts.size = size_filters;
    // --pattern fd
    opts.pattern = "fd".to_string();
    // --type f
    opts.filetype = Some(vec![FileType::File]);
    // the search path
    let cwd = env::current_dir().expect("Failed to get current directory");
    let paths: Vec<PathBuf> = vec![cwd];
    opts.path = paths;
    let result = scan(opts);
    match result {
        Ok(entries) => {
            for entry in entries {
                println!("{}", entry.display());
            }
        }
        Err(err) => {
            eprintln!("Error: {:#}", err);
        }
    }
}
