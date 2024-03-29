use super::{cli::InvokeOptions, cli::MessageOutput, workpath::WorkPath};
use std::{
    cell::RefCell,
    io,
    iter::Iterator,
    process::{Command, Stdio},
};
use walkdir::WalkDir;

pub fn find_git_repositories(opts: &InvokeOptions) -> io::Result<()> {
    let output = &RefCell::new(MessageOutput::new(opts));

    let walk = WalkDir::new(opts.search_root())
        .follow_links(opts.follow_symlinks())
        .min_depth(0)
        .max_depth(opts.max_depth());

    let maybe_error = walk
        .into_iter()
        .filter_entry(|e| {
            let is_dir = e.file_type().is_dir();
            let mut path = WorkPath::new(e.path());

            if is_dir && is_git_repo(&mut path, opts, &output).unwrap_or(false) {
                path.resolve_canonical(&output);
                println!("{}", path);
                false
            } else if is_dir && should_skip_directory(&path, opts) {
                path.resolve_canonical(&output);
                output
                    .borrow_mut()
                    .log_info(format_args!("Skipping directory: {}", path))
                    .unwrap_or_default();
                false
            } else {
                true
            }
        })
        // Stop on the first error, or process the whole tree
        .filter(|x| match x {
            Ok(_) => false, // Normal directory entries which are not git repositories
            Err(e) => match (e.loop_ancestor(), e.io_error()) {
                // Loop detected, keep scanning
                (Some(cycle), _) => {
                    let mut workpath = WorkPath::new(cycle);
                    workpath.resolve_canonical(&output);

                    let output = &mut output.borrow_mut();
                    output
                        .log_warning(format_args!(
                            "A symlink cycle was detected at: {}",
                            workpath
                        ))
                        .unwrap_or_default();
                    false
                }
                (_, Some(io)) => match io.kind() {
                    io::ErrorKind::PermissionDenied => {
                        if let Some(path) = e.path() {
                            let mut workpath = WorkPath::new(path);
                            workpath.resolve_canonical(&output);

                            let output = &mut output.borrow_mut();
                            output
                                .log_warning(format_args!(
                                    "Insufficient permissions to traverse: {}",
                                    workpath
                                ))
                                .unwrap_or_default();
                        } else {
                            output
                                .borrow_mut()
                                .log_warning(format_args!(
                                    "Permission denied while scanning for repositories"
                                ))
                                .unwrap_or_default();
                        }
                        false
                    }
                    _ => true,
                },
                // Probably impossible, but some other combination of error
                _ => true,
            },
        })
        .next();

    if let Some(Err(error)) = maybe_error {
        let output = &mut output.borrow_mut();
        output
            .log_error(format_args!(
                "An error occurred while scanning for git repositories"
            ))
            .unwrap_or_default();
        output
            .log_error(format_args!("{}", error))
            .unwrap_or_default();
    }

    Ok(())
}

fn should_skip_directory(path: &WorkPath, opts: &InvokeOptions) -> bool {
    !opts.show_all()
        && path
            .file_name()
            .to_str()
            .map(|s| s.starts_with("."))
            .unwrap_or(true) // If we can't decode the name, we should probably skip it
}

fn is_git_repo(
    path: &mut WorkPath,
    opts: &InvokeOptions,
    output: &RefCell<MessageOutput>,
) -> io::Result<bool> {
    // See if we have a .git directory
    let dot_git_path = path.as_maybe_unresolved_path().join(".git");
    let has_dot_git = dot_git_path.metadata().map(|m| m.is_dir());

    let has_dot_git = match has_dot_git {
        Ok(x) => x,
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => false,
            io::ErrorKind::PermissionDenied => {
                let mut workpath = WorkPath::new(dot_git_path.as_path());
                workpath.resolve_canonical(output);

                output
                    .borrow_mut()
                    .log_error(format_args!(
                        "Insufficient permissions to traverse: {}",
                        workpath
                    ))
                    .unwrap_or_default();
                false
            }
            _ => return Err(err),
        },
    };

    Ok(has_dot_git && (!opts.paranoid() || is_git_repo_paranoid(path, output)?))
}

fn is_git_repo_paranoid(path: &mut WorkPath, output: &RefCell<MessageOutput>) -> io::Result<bool> {
    path.resolve_canonical(output);

    let output = &mut output.borrow_mut();
    output
        .log_info(format_args!("Paranoid: Checking {}", path))
        .unwrap_or_default();

    // We expect `git rev-parse HEAD` to complete with exit code 0
    let test = Command::new("git")
        .current_dir(path.as_maybe_unresolved_path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .args(&["rev-parse", "HEAD"])
        .status();

    match test {
        Ok(status) => {
            if !status.success() {
                output
                    .log_warning(format_args!("Paranoid check failed for: {}", path))
                    .unwrap_or_default();
                Ok(false)
            } else {
                Ok(true)
            }
        }
        Err(error) => {
            output
                .log_error(format_args!("Failed to run --paranoid repository check. Is git installed and configured correctly?"))
                .unwrap_or_default();
            output
                .log_error(format_args!("{}", error))
                .unwrap_or_default();
            Err(error)
        }
    }
}
