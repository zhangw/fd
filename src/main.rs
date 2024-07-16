mod cli;
mod compose;
mod config;
mod dir_entry;
mod error;
mod exec;
mod exit_codes;
mod filesystem;
mod filetypes;
mod filter;
mod fmt;
mod output;
mod regex_helper;
mod walk;

use crate::cli::Opts;
use anyhow::{bail, Result};
use clap::{CommandFactory, Parser};
use compose::{
    build_pattern_regex, build_regex, construct_config, ensure_search_pattern_is_not_a_path,
    ensure_use_hidden_option_for_leading_dot_pattern, set_working_dir,
};
use exit_codes::ExitCode;
use regex::bytes::Regex;
use std::env;
use std::path::Path;

// We use jemalloc for performance reasons, see https://github.com/sharkdp/fd/pull/481
// FIXME: re-enable jemalloc on macOS, see comment in Cargo.toml file for more infos
#[cfg(all(
    not(windows),
    not(target_os = "android"),
    not(target_os = "macos"),
    not(target_os = "freebsd"),
    not(target_os = "openbsd"),
    not(all(target_env = "musl", target_pointer_width = "32")),
    not(target_arch = "riscv64"),
    feature = "use-jemalloc"
))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() {
    let result = run();
    match result {
        Ok(exit_code) => {
            exit_code.exit();
        }
        Err(err) => {
            eprintln!("[fd error]: {:#}", err);
            ExitCode::GeneralError.exit();
        }
    }
}

fn run() -> Result<ExitCode> {
    let opts = Opts::parse();

    #[cfg(feature = "completions")]
    if let Some(shell) = opts.gen_completions()? {
        return print_completions(shell);
    }

    set_working_dir(&opts)?;
    let search_paths = opts.search_paths()?;
    if search_paths.is_empty() {
        bail!("No valid search paths given.");
    }

    ensure_search_pattern_is_not_a_path(&opts)?;
    let pattern = &opts.pattern;
    let exprs = &opts.exprs;
    let empty = Vec::new();

    let pattern_regexps = exprs
        .as_ref()
        .unwrap_or(&empty)
        .iter()
        .chain([pattern])
        .map(|pat| build_pattern_regex(pat, &opts))
        .collect::<Result<Vec<String>>>()?;

    let config = construct_config(opts, &pattern_regexps)?;

    ensure_use_hidden_option_for_leading_dot_pattern(&config, &pattern_regexps)?;

    let regexps = pattern_regexps
        .into_iter()
        .map(|pat| build_regex(pat, &config))
        .collect::<Result<Vec<Regex>>>()?;

    walk::scan(&search_paths, regexps, config)
}

#[cfg(feature = "completions")]
#[cold]
fn print_completions(shell: clap_complete::Shell) -> Result<ExitCode> {
    // The program name is the first argument.
    let first_arg = env::args().next();
    let program_name = first_arg
        .as_ref()
        .map(Path::new)
        .and_then(|path| path.file_stem())
        .and_then(|file| file.to_str())
        .unwrap_or("fd");
    let mut cmd = Opts::command();
    cmd.build();
    clap_complete::generate(shell, &mut cmd, program_name, &mut std::io::stdout());
    Ok(ExitCode::Success)
}
