use std::fs::{self, DirEntry, Metadata};
use std::io;
use std::path::{Path, PathBuf};

use crate::symlinks::SymlinkBehaviour;

pub struct SearchPath {
    depth: u32,
    path: PathBuf,
    entry: Option<DirEntry>,
}

impl SearchPath {
    pub fn to_path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn depth(&self) -> u32 {
        self.depth
    }

    pub fn resolve_symlinks(&mut self, symlink_behaviour: &SymlinkBehaviour) -> bool {
        match symlink_behaviour {
            SymlinkBehaviour::Skip => {
                let is_symlink = match &self.entry {
                    Some(entry) => SearchPath::is_metadata_symlink(entry.metadata()),
                    None => SearchPath::is_metadata_symlink(fs::symlink_metadata(self.to_path())),
                };
                !is_symlink
            }
            SymlinkBehaviour::Follow => {
                if let Ok(path) = fs::read_link(self.to_path()) {
                    self.path = path;
                }
                true
            }
        }
    }

    fn is_metadata_symlink(maybe_metadata: io::Result<Metadata>) -> bool {
        match maybe_metadata {
            Err(_e) => true,
            Ok(meta) => !meta.is_dir() && !meta.is_file(),
        }
    }

    pub fn from_path(path: PathBuf, depth: u32) -> SearchPath {
        SearchPath {
            path,
            depth,
            entry: None,
        }
    }

    pub fn from_dir_entry(entry: DirEntry, depth: u32) -> SearchPath {
        SearchPath {
            depth,
            path: entry.path(),
            entry: Some(entry),
        }
    }
}
