extern crate clap;

mod search;
use search::cli::RunOptions;
use search::walk;

fn main() {
    let mut run = RunOptions::parse_cli();
    match &mut run {
        Ok(options) => walk::find_git_repositories(options),
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(-1);
        }
    }
}
