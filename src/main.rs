use std::env;
use std::ffi::OsStr;
use std::fs;
use std::fs::{DirEntry, ReadDir};
use std::path::Path;
use std::path::PathBuf;
use std::vec::Vec;

extern crate clap;
use clap::{App, Arg};

mod search;

use search::paths::SearchPath;
use search::symlink::{FollowState, SymlinkBehaviour, SymlinkResolveOutcome};

fn main() {
    let app = App::new("fgr")
        .version("0.1")
        .author("Nick Young")
        .about("A simple utility for finding git repositories.")
        .arg(
            Arg::with_name("search-root")
                .takes_value(true)
                .value_name("PATH")
                .help("The directory where the search will begin"),
        )
        .arg(
            Arg::with_name("all")
                .takes_value(false)
                .short("a")
                .long("all")
                .help("Do not ignore directories starting with `.`"),
        )
        .arg(
            Arg::with_name("verbose")
                .takes_value(false)
                .short("v")
                .long("verbose")
                .help("Output detailed messages to standard error"),
        )
        .arg(
            Arg::with_name("symlinks")
                .short("s")
                .long("follow-symlinks")
                .takes_value(false)
                .help("Follow symlinks rather than ignoring them"),
        );

    let matches = app.get_matches();

    let follow_symlinks = matches.is_present("symlinks");
    let show_all = matches.is_present("all");
    let verbose_output = matches.is_present("verbose");

    let mut symlink_behaviour = match follow_symlinks {
        false => SymlinkBehaviour::Skip,
        true => SymlinkBehaviour::Follow(FollowState::new()),
    };

    match get_search_root(matches.value_of("search-root")) {
        None => {
            eprintln!("Directory is invalid or does not exist.");
            std::process::exit(-1);
        }
        Some(search_root) => {
            if search_root.is_dir() {
                do_perform_walk(
                    search_root,
                    &mut symlink_behaviour,
                    show_all,
                    verbose_output,
                );
            } else {
                eprintln!("{} is not a directory.", search_root.display());
            }
        }
    };
}

fn get_search_root(cfg: Option<&str>) -> Option<PathBuf> {
    match cfg {
        None => Some(env::current_dir().expect("Could not get current directory.")),
        Some(path_str) => PathBuf::from(path_str).canonicalize().ok(),
    }
}

fn do_perform_walk(
    root_dir: PathBuf,
    symlink_behaviour: &mut SymlinkBehaviour,
    show_all: bool,
    verbose_output: bool,
) {
    let mut to_walk = Vec::new();
    to_walk.push(SearchPath::from_path(root_dir, 0));

    while let Some(mut search_path) = to_walk.pop() {
        // Handle symlinks
        {
            match search_path.resolve_symlinks(symlink_behaviour) {
                SymlinkResolveOutcome::AlreadyTraversed => {
                    if verbose_output {
                        eprintln!(
                            "Skipping: {}, the directory has already been traversed.",
                            search_path.to_path().display()
                        );
                    }
                    continue;
                }
                SymlinkResolveOutcome::SkipSymlink => {
                    if verbose_output {
                        eprintln!(
                            "Skipping {}, because it is a symlink",
                            search_path.to_path().display()
                        );
                    }
                    continue;
                }
                SymlinkResolveOutcome::CanonicalizeFailed(error_message) => {
                    if verbose_output {
                        eprintln!(
                            "Tried to follow symlink: {}, but there was an error resolving the link target => {}.",
                            search_path.to_path().display(),
                            error_message
                        );
                    }
                    continue;
                }
                SymlinkResolveOutcome::ReadLinkFailed(error_message) => {
                    if verbose_output {
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
                    to_walk.push(SearchPath::from_dir_entry(dir, new_depth));
                };

                if handle_children(dir, entries, show_all, &mut add_child) {
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
