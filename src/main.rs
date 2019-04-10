use std::env;
use std::ffi::OsStr;
use std::fs;
use std::fs::{DirEntry, ReadDir};
use std::path::Path;
use std::path::PathBuf;
use std::vec::Vec;

#[macro_use]
extern crate clap;

use clap::{App, Arg};

mod paths;
mod symlinks;

use paths::SearchPath;
use symlinks::{FollowState, SymlinkBehaviour, SymlinkOption};

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
            Arg::with_name("match-name")
                .short("n")
                .long("match-name")
                .takes_value(true)
                .value_name("PATTERN")
                .help("Exclude repositories that match this pattern"),
        )
        .arg(
            Arg::with_name("symlinks")
                .short("s")
                .long("symlinks")
                .takes_value(true)
                .value_name("STRATEGY")
                .possible_values(&SymlinkOption::variants())
                .case_insensitive(true)
                .default_value("skip")
                .help("Strategy for handling symlinks"),
        );

    let matches = app.get_matches();
    let symlink_option =
        value_t!(matches, "symlinks", SymlinkOption).expect("Invalid value for -s/--symlinks.");

    let mut symlink_behaviour = match symlink_option {
        SymlinkOption::Skip => SymlinkBehaviour::Skip,
        SymlinkOption::Follow => SymlinkBehaviour::Follow(FollowState::new()),
    };

    match get_search_root(matches.value_of("search-root")) {
        None => {
            eprintln!("Directory is invalid or does not exist.");
            std::process::exit(-1);
        }
        Some(search_root) => {
            if search_root.is_dir() {
                do_perform_walk(search_root, &mut symlink_behaviour);
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

fn do_perform_walk(root_dir: PathBuf, symlink_behaviour: &mut SymlinkBehaviour) {
    let mut to_walk = Vec::new();
    to_walk.push(SearchPath::from_path(root_dir, 0));

    while let Some(mut search_path) = to_walk.pop() {
        // Either skip symlinks, or resolve the actual path
        {
            if !search_path.resolve_symlinks(symlink_behaviour) {
                eprintln!(
                    "Ignoring {}, because it is a symlink",
                    search_path.to_path().display()
                );
                continue;
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
                let mut add_child =
                    |dir: DirEntry| to_walk.push(SearchPath::from_dir_entry(dir, new_depth));

                if handle_children(dir, entries, &mut add_child) {
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

                        add_child(entry);
                    }
                }
            }
        }
    }

    false
}

fn is_git_repo(_dir: &PathBuf, file_name: &OsStr) -> bool {
    return file_name == ".git";
}
