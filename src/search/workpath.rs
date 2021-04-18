use super::cli::MessageOutput;
use std::{
    cell::RefCell,
    ffi::OsStr,
    fmt::{self, Display},
    path::PathBuf,
};
use std::{mem, path::Path};

#[cfg(windows)]
static WIN_EXTENDED_PATH_PREFIX: &str = r"\\?\";

// https://docs.microsoft.com/en-us/windows/desktop/FileIO/naming-a-file#maximum-path-length-limitation
#[cfg(windows)]
static WIN_STANDARD_MAX_PATH_LEN: usize = 160;

pub enum WorkPath<'a> {
    Unresolved(&'a Path),
    Canonical(PathBuf),
}

impl WorkPath<'_> {
    pub fn new(path: &Path) -> WorkPath {
        WorkPath::Unresolved(path)
    }

    pub fn file_name(&self) -> &OsStr {
        let path = self.as_maybe_unresolved_path();
        path.file_name().unwrap_or_else(|| path.as_os_str())
    }

    pub fn as_maybe_unresolved_path(&self) -> &Path {
        match self {
            WorkPath::Unresolved(p) => p,
            WorkPath::Canonical(p) => p.as_path(),
        }
    }

    pub fn resolve_canonical(&mut self, output: &RefCell<MessageOutput>) -> &Self {
        if let WorkPath::Unresolved(path) = self {
            let canonical = path.canonicalize();

            if let Err(e) = &canonical {
                let output = &mut output.borrow_mut();
                output
                    .log_warning(format_args!(
                        "Could not determine the canonical path for: {}",
                        path.display()
                    ))
                    .unwrap_or_default();
                output
                    .log_warning(format_args!("{}", e))
                    .unwrap_or_default();
            }

            #[cfg(windows)]
            {
                let probably_canonical = canonical.as_deref().unwrap_or(path);

                if let Some(s) = probably_canonical.to_str() {
                    if s.starts_with(WIN_EXTENDED_PATH_PREFIX)
                        && s.len() < (WIN_STANDARD_MAX_PATH_LEN + WIN_EXTENDED_PATH_PREFIX.len())
                    {
                        let non_extended = &s[WIN_EXTENDED_PATH_PREFIX.len()..];
                        let _ =
                            mem::replace(self, WorkPath::Canonical(PathBuf::from(non_extended)));
                        return self;
                    }
                }
            }

            let probably_canonical = canonical.unwrap_or_else(|_err| path.to_owned());
            let _ = mem::replace(self, WorkPath::Canonical(probably_canonical));
        }

        self
    }
}

impl Display for WorkPath<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WorkPath::Unresolved(_entry) => panic!("We should always resolve the path."),
            WorkPath::Canonical(path) => path.display().fmt(f),
        }
    }
}
