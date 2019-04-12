use std::fs::DirEntry;
use std::path::{Path, PathBuf};

use crate::search::symlink::{SymlinkBehaviour, SymlinkResolveOutcome};

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

    pub fn dir_entry(&self) -> &Option<DirEntry> {
        &self.entry
    }

    pub fn resolve_symlinks(
        &mut self,
        symlink_behaviour: &mut SymlinkBehaviour,
    ) -> SymlinkResolveOutcome {
        let result = symlink_behaviour.resolve_search_path(self);

        if let Some(updated_path) = result.updated_path {
            self.path = updated_path;
        }

        result.resolution
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
