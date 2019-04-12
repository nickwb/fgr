use crate::search::symlink::{FollowState, SymlinkBehaviour};
use clap::{App, Arg};
use std::env;
use std::error::Error;
use std::path::PathBuf;

pub struct RunOptions {
    search_root: PathBuf,
    symlink_behaviour: SymlinkBehaviour,
    show_all: bool,
    verbose: bool,
}

impl RunOptions {
    pub fn search_root(&self) -> &PathBuf {
        &self.search_root
    }

    pub fn symlink_behaviour(&mut self) -> &mut SymlinkBehaviour {
        &mut self.symlink_behaviour
    }

    pub fn show_all(&self) -> bool {
        self.show_all
    }

    pub fn verbose(&self) -> bool {
        self.verbose
    }

    pub fn parse_cli() -> Result<RunOptions, String> {
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
        let verbose = matches.is_present("verbose");

        let symlink_behaviour = match follow_symlinks {
            false => SymlinkBehaviour::Skip,
            true => SymlinkBehaviour::Follow(FollowState::new()),
        };

        match get_search_root(matches.value_of("search-root")) {
            Err(error) => Err(error),
            Ok(search_root) => Ok(RunOptions {
                search_root,
                symlink_behaviour,
                show_all,
                verbose,
            }),
        }
    }
}

fn get_search_root(cfg: Option<&str>) -> Result<PathBuf, String> {
    match cfg {
        None => match env::current_dir() {
            Ok(path) => Ok(path),
            Err(error) => Err(format!(
                "Could not get current directory => {}",
                error.description()
            )),
        },
        Some(path_str) => match PathBuf::from(path_str).canonicalize() {
            Ok(path) => {
                if path.is_dir() {
                    Ok(path)
                } else {
                    Err(format!("{} is not a directory.", path_str))
                }
            }
            Err(error) => Err(format!(
                "Directory `{}` is invalid or does not exist => {}",
                path_str,
                error.description()
            )),
        },
    }
}