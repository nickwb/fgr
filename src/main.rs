extern crate clap;

mod search;
use search::cli::FgrRun;
use search::walk;

fn main() {
    let mut maybe_run = FgrRun::parse_cli();
    match &mut maybe_run {
        Ok(run) => walk::find_git_repositories(run),
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(-1);
        }
    }
}
