use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::fs::{DirEntry, ReadDir};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::vec::Vec;

use crate::search::candidate::SearchCandidate;
use crate::search::cli::FgrRun;
use crate::search::symlink::SymlinkResolveOutcome;

pub fn find_git_repositories(run: &mut FgrRun) {
    let mut to_walk = Vec::new();
    to_walk.push(SearchCandidate::from_path(run.search_root().clone(), 0));

    while let Some(mut search_path) = to_walk.pop() {
        // Handle symlinks
        {
            match search_path.resolve_symlinks(run.symlink_behaviour()) {
                SymlinkResolveOutcome::AlreadyTraversed => {
                    run.log_info(format_args!(
                        "Skipping: {}, the directory has already been traversed.",
                        search_path.normal()
                    ));
                    continue;
                }
                SymlinkResolveOutcome::SkipSymlink => {
                    run.log_info(format_args!(
                        "Skipping {}, because it is a symlink",
                        search_path.normal()
                    ));
                    continue;
                }
                SymlinkResolveOutcome::CanonicalizeFailed(error_message) => {
                    run.log_warning(format_args!(
                        "Tried to follow symlink: {}, but there was an error resolving the link target => {}.",
                        search_path.normal(),
                        error_message
                    ));
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
                run.log_error(format_args!(
                    "Can't walk directory {}. {}",
                    search_path.normal(),
                    e
                ));
                continue;
            }
            Ok(entries) => {
                let new_depth = search_path.depth() + 1;
                let mut add_search_candidate = |dir: DirEntry| {
                    to_walk.push(SearchCandidate::from_dir_entry(dir, new_depth));
                };

                if is_git_repo(&search_path, entries, &mut add_search_candidate, run) {
                    // We got a result, write it to stdout
                    run.output_result(search_path.normal());

                    // Backtrack, we don't need to scan any of the children of this directory
                    while to_walk.len() > start_from {
                        to_walk.pop();
                    }
                }
            }
        }
    }
}

fn is_git_repo<FnAddCandidate: FnMut(DirEntry)>(
    search_path: &SearchCandidate,
    entries: ReadDir,
    add_search_candidate: &mut FnAddCandidate,
    run: &mut FgrRun,
) -> bool {
    for entry in entries {
        match entry {
            Err(e) => {
                run.log_error(format_args!(
                    "Error while walking directory {}. {}",
                    search_path.normal(),
                    e
                ));
                return false;
            }
            Ok(entry) => {
                let path = entry.path();

                if path.is_dir() {
                    if let Some(name) = path.file_name() {
                        if is_dot_git_dir(name) && is_git_repo_paranoid(search_path, run) {
                            return true;
                        }

                        if should_skip_directory(&path, name, run) {
                            continue;
                        }

                        add_search_candidate(entry);
                    }
                }
            }
        }
    }

    false
}

fn is_dot_git_dir(file_name: &OsStr) -> bool {
    file_name == ".git"
}

fn is_git_repo_paranoid(search_path: &SearchCandidate, run: &mut FgrRun) -> bool {
    if !run.paranoid() {
        return true;
    }

    run.log_info(format_args!("Paranoid: Checking {}", search_path.normal()));

    // We expect `git rev-parse HEAD` to complete with exit code 0
    let test = Command::new("git")
        .current_dir(search_path.to_path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .args(&["rev-parse", "HEAD"])
        .status();

    match test {
        Ok(status) => status.success(),
        Err(error) => {
            run.log_error(format_args!("Failed to run --paranoid repository check. Is git installed and configured correctly?"));
            run.log_error(format_args!("{}", error.description()));
            false
        }
    }
}

fn should_skip_directory(_dir: &PathBuf, file_name: &OsStr, run: &FgrRun) -> bool {
    !run.show_all()
        && match file_name.to_str() {
            Some(str) => str.starts_with("."),
            None => true, // If we can't even decode the file name...
        }
}
