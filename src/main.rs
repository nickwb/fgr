use std::env;
use std::ffi::OsStr;
use std::fs;
use std::fs::ReadDir;
use std::path::Path;
use std::path::PathBuf;
use std::vec::Vec;

extern crate clap;
use clap::{App, Arg};

fn main() {
    let app = App::new("fgr")
        .version("0.1")
        .author("Nick Young")
        .about("A simple utility for finding git repositories.")
        .arg(
            Arg::with_name("search-root")
                .takes_value(true)
                .value_name("PATH")
                .help("The root directory where the search will begin. Defaults to current working directory."),
        )
        .arg(
            Arg::with_name("match-name")
                .short("n")
                .long("match-name")
                .takes_value(true)
                .value_name("PATTERN")
                .help("Exclude repositories where the directory name does not match this pattern."),
        )
        .arg(
            Arg::with_name("follow-symlinks")
                .short("s")
                .long("follow-symlinks")
                .help("Follow symlinks when traversing directories."),
        );

    let matches = app.get_matches();

    match get_search_root(matches.value_of("search-root")) {
        None => {
            eprintln!("Directory is invalid or does not exist.");
            std::process::exit(-1);
        }
        Some(search_root) => {
            if search_root.is_dir() {
                let follow_symlinks = matches.is_present("follow-symlinks");
                do_perform_walk(search_root, follow_symlinks);
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

fn do_perform_walk(root_dir: PathBuf, _follow_symlinks: bool) {
    let mut to_walk = Vec::new();
    to_walk.push(root_dir);

    while let Some(dir_buf) = to_walk.pop() {
        let start_from = to_walk.len();
        let dir = dir_buf.as_path();
        match fs::read_dir(dir) {
            Err(e) => {
                eprintln!("Can't walk directory {}. {}", dir.display(), e);
                continue;
            }
            Ok(entries) => {
                let mut add_child = |dir: PathBuf| to_walk.push(dir);
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

fn handle_children<AddChild: FnMut(PathBuf)>(
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

                        add_child(path);
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
