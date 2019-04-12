use std::ffi::OsStr;
use std::fs;
use std::fs::{DirEntry, ReadDir};
use std::path::Path;
use std::path::PathBuf;
use std::vec::Vec;

extern crate clap;

mod search;
use search::candidate::SearchCandidate;
use search::cli::RunOptions;
use search::symlink::SymlinkResolveOutcome;

fn main() {
    let mut run = RunOptions::parse_cli();
    match &mut run {
        Ok(options) => do_perform_walk(options),
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(-1);
        }
    }
}

fn do_perform_walk(options: &mut RunOptions) {
    let mut to_walk = Vec::new();
    to_walk.push(SearchCandidate::from_path(options.search_root().clone(), 0));

    while let Some(mut search_path) = to_walk.pop() {
        // Handle symlinks
        {
            match search_path.resolve_symlinks(options.symlink_behaviour()) {
                SymlinkResolveOutcome::AlreadyTraversed => {
                    if options.verbose() {
                        eprintln!(
                            "Skipping: {}, the directory has already been traversed.",
                            search_path.to_path().display()
                        );
                    }
                    continue;
                }
                SymlinkResolveOutcome::SkipSymlink => {
                    if options.verbose() {
                        eprintln!(
                            "Skipping {}, because it is a symlink",
                            search_path.to_path().display()
                        );
                    }
                    continue;
                }
                SymlinkResolveOutcome::CanonicalizeFailed(error_message) => {
                    if options.verbose() {
                        eprintln!(
                            "Tried to follow symlink: {}, but there was an error resolving the link target => {}.",
                            search_path.to_path().display(),
                            error_message
                        );
                    }
                    continue;
                }
                SymlinkResolveOutcome::ReadLinkFailed(error_message) => {
                    if options.verbose() {
                        eprintln!(
                            "Skipping: {}, Could not determine if this was a symlink or not => {}.",
                            search_path.to_path().display(),
                            error_message
                        );
                    }
                    continue;
                }
                SymlinkResolveOutcome::FollowSymlink => (),
                SymlinkResolveOutcome::NotSymlink => (),
            }
        }

        let dir = search_path.to_path();
        let start_from = to_walk.len();

        match fs::read_dir(dir) {
            Err(e) => {
                eprintln!("Can't walk directory {}. {}", dir.display(), e);
                continue;
            }
            Ok(entries) => {
                let new_depth = search_path.depth() + 1;
                let mut add_child = |dir: DirEntry| {
                    to_walk.push(SearchCandidate::from_dir_entry(dir, new_depth));
                };

                if handle_children(dir, entries, options.show_all(), &mut add_child) {
                    println!("{}", dir.display());

                    // Backtrack, we don't need to scan any of the children of this directory
                    while to_walk.len() > start_from {
                        to_walk.pop();
                    }
                }
            }
        }
    }
}

fn handle_children<AddChild: FnMut(DirEntry)>(
    dir: &Path,
    entries: ReadDir,
    show_all: bool,
    add_child: &mut AddChild,
) -> bool {
    for entry in entries {
        match entry {
            Err(e) => {
                eprintln!("Error while walking directory {}. {}", dir.display(), e);
                return false;
            }
            Ok(entry) => {
                let path = entry.path();

                if path.is_dir() {
                    if let Some(name) = path.file_name() {
                        if is_git_repo(&path, name) {
                            return true;
                        }

                        if should_skip_directory(&path, name, show_all) {
                            continue;
                        }

                        add_child(entry);
                    }
                }
            }
        }
    }

    false
}

fn is_git_repo(_dir: &PathBuf, file_name: &OsStr) -> bool {
    file_name == ".git"
}

fn should_skip_directory(_dir: &PathBuf, file_name: &OsStr, show_all: bool) -> bool {
    !show_all
        && match file_name.to_str() {
            Some(str) => str.starts_with("."),
            None => true, // If we can't even decode the file name...
        }
}
