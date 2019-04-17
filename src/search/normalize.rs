use std::fmt::{self, Display};
use std::path::Path;

#[cfg(windows)]
static WIN_EXTENDED_PATH_PREFIX: &str = r"\\?\";

// https://docs.microsoft.com/en-us/windows/desktop/FileIO/naming-a-file#maximum-path-length-limitation
#[cfg(windows)]
static WIN_STANDARD_MAX_PATH_LEN: usize = 160;

pub enum NormalizedPath<'a> {
    Untouched(&'a Path),

    // Only used by Windows, so a warning would normally be emitted on Unix builds
    #[allow(dead_code)]
    Sliced(&'a str),
}

impl NormalizedPath<'_> {
    pub fn new(path: &Path) -> NormalizedPath {
        // This block is only included in Windows builds.
        #[cfg(windows)]
        {
            if let Some(s) = path.to_str() {
                if s.starts_with(WIN_EXTENDED_PATH_PREFIX)
                    && s.len() < (WIN_STANDARD_MAX_PATH_LEN + WIN_EXTENDED_PATH_PREFIX.len())
                {
                    let non_extended = &s[WIN_EXTENDED_PATH_PREFIX.len()..];
                    return NormalizedPath::Sliced(non_extended);
                }
            }
        }

        NormalizedPath::Untouched(path)
    }
}

impl Display for NormalizedPath<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NormalizedPath::Untouched(path) => path.display().fmt(f),
            NormalizedPath::Sliced(str) => str.fmt(f),
        }
    }
}
