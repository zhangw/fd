#![allow(unused)]
use std::path::{Path, PathBuf};

use anyhow::Result;
use cli::Opts;
use compose::{
    build_pattern_regex, build_regex, construct_config, ensure_search_pattern_is_not_a_path,
    ensure_use_hidden_option_for_leading_dot_pattern, set_working_dir,
};
use dir_entry::DirEntry;
use regex::bytes::Regex;
mod compose;
mod config;
mod dir_entry;
mod error;
mod exec;
mod exit_codes;
mod filesystem;
mod filetypes;
mod fmt;
mod output;
mod regex_helper;
mod walk;

pub mod cli;
pub mod filter;

pub fn scan(opts: Opts) -> Result<Vec<PathBuf>> {
    set_working_dir(&opts).expect("Failed to set working directory");
    let search_paths = opts.search_paths().expect("Failed to get search paths");
    ensure_search_pattern_is_not_a_path(&opts)
        .expect("Failed to ensure search pattern is not a path");
    let pattern = &opts.pattern;
    let exprs = &opts.exprs;
    let empty = Vec::new();

    let pattern_regexps = exprs
        .as_ref()
        .unwrap_or(&empty)
        .iter()
        .chain([pattern])
        .map(|pat| build_pattern_regex(pat, &opts))
        .collect::<Result<Vec<String>>>()
        .expect("Failed to build pattern regex");

    let config = construct_config(opts, &pattern_regexps).expect("Failed to construct config");

    ensure_use_hidden_option_for_leading_dot_pattern(&config, &pattern_regexps)
        .expect("Failed to ensure use hidden option for leading dot pattern");

    let regexps = pattern_regexps
        .into_iter()
        .map(|pat| build_regex(pat, &config))
        .collect::<Result<Vec<Regex>>>()
        .expect("Failed to build regex");

    let result = walk::scan_and_collect(&search_paths, regexps, config);
    match result {
        Ok(entries) => Ok(entries
            .into_iter()
            .map(|arg0: dir_entry::DirEntry| DirEntry::path(&arg0).to_path_buf())
            .collect()),
        Err(err) => Err(err),
    }
}
