use std::env;
use std::fs;
use std::path::{Path, PathBuf};
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
                walk_directories(search_root.as_path(), follow_symlinks);
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

fn walk_directories(dir: &Path, follow_symlinks: bool) {
    let mut children = Vec::new();

    match fs::read_dir(dir) {
        Err(e) => {
            eprintln!("Can't walk directory {}. {}", dir.display(), e);
            return;
        }
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Err(e) => {
                        eprintln!("Error while walking directory {}. {}", dir.display(), e);
                        return;
                    }
                    Ok(entry) => {
                        let path = entry.path();
                        if path.is_dir() {
                            if let Some(name) = path.file_name() {
                                if name == ".git" {
                                    println!("{}", dir.display());
                                    return;
                                }
                            }

                            children.push(path);
                        }
                    }
                }
            }
        }
    }

    for sub_directory in children {
        walk_directories(&sub_directory, follow_symlinks);
    }
}
