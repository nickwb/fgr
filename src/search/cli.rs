use clap::{App, Arg};
use std::io::Write;
use std::{env, fmt::Arguments};
use std::{io, path::PathBuf};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub struct InvokeOptions {
    search_root: PathBuf,
    follow_symlinks: bool,
    show_all: bool,
    paranoid: bool,
    verbose: bool,
    max_depth: usize,
}

impl InvokeOptions {
    pub fn search_root(&self) -> &PathBuf {
        &self.search_root
    }

    pub fn follow_symlinks(&self) -> bool {
        self.follow_symlinks
    }

    pub fn show_all(&self) -> bool {
        self.show_all
    }

    pub fn paranoid(&self) -> bool {
        self.paranoid
    }

    pub fn verbose(&self) -> bool {
        self.verbose
    }

    pub fn max_depth(&self) -> usize {
        self.max_depth
    }

    pub fn parse_cli() -> Result<InvokeOptions, String> {
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
                Arg::with_name("paranoid")
                    .takes_value(false)
                    .short("p")
                    .long("paranoid")
                    .help("Be extra certain that a directory is a git repository."),
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
            )
            .arg(
                Arg::with_name("max-depth")
                    .short("d")
                    .long("max-depth")
                    .takes_value(true)
                    .default_value("10")
                    .conflicts_with("any-depth")
                    .help("Sets the maximum depth when recursively scanning subdirectories"),
            )
            .arg(
                Arg::with_name("any-depth")
                    .long("any-depth")
                    .conflicts_with("max-depth")
                    .help("Drops the default max-depth limit, allowing unlimited depth"),
            );

        let matches = app.get_matches();

        let follow_symlinks = matches.is_present("symlinks");
        let show_all = matches.is_present("all");
        let paranoid = matches.is_present("paranoid");
        let verbose = matches.is_present("verbose");

        let max_depth = if matches.is_present("any-depth") {
            usize::MAX
        } else {
            matches
                .value_of("max-depth")
                .and_then(|n| n.parse::<usize>().ok())
                .unwrap_or(10)
        };

        match get_search_root(matches.value_of("search-root")) {
            Err(error) => Err(error),
            Ok(search_root) => Ok(InvokeOptions {
                search_root,
                follow_symlinks,
                show_all,
                paranoid,
                verbose,
                max_depth,
            }),
        }
    }
}

fn get_search_root(cfg: Option<&str>) -> Result<PathBuf, String> {
    match cfg {
        None => match env::current_dir() {
            Ok(path) => Ok(path),
            Err(error) => Err(format!("Could not get current directory => {}", error)),
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
                path_str, error
            )),
        },
    }
}

pub struct MessageOutput {
    is_verbose: bool,
    stderr: StandardStream,
}

impl MessageOutput {
    pub fn new(opts: &InvokeOptions) -> Self {
        Self {
            is_verbose: opts.verbose(),
            stderr: StandardStream::stderr(ColorChoice::Auto),
        }
    }

    pub fn log_info(&mut self, message: Arguments) -> io::Result<()> {
        if self.is_verbose {
            self.stderr
                .set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))?;
            writeln!(&mut self.stderr, "{}", message)?;
            self.stderr.reset()?;
        }
        Ok(())
    }

    pub fn log_warning(&mut self, message: Arguments) -> io::Result<()> {
        if self.is_verbose {
            self.stderr
                .set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
            writeln!(&mut self.stderr, "{}", message)?;
            self.stderr.reset()?;
        }
        Ok(())
    }

    pub fn log_error(&mut self, message: Arguments) -> io::Result<()> {
        self.stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
        writeln!(&mut self.stderr, "{}", message)?;
        self.stderr.reset()?;
        Ok(())
    }
}
