extern crate clap;
extern crate termcolor;

mod search;
use search::{cli::InvokeOptions, walk};

fn main() {
    let result = InvokeOptions::parse_cli()
        .and_then(|opts| walk::find_git_repositories(&opts).map_err(|err| err.to_string()));

    if let Err(error) = result {
        eprintln!("{}", error);
        std::process::exit(-1);
    }
}
