use std::fs::{self, DirEntry, Metadata};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use crate::symlinks::{FollowState, SymlinkBehaviour};

pub struct SearchPath {
    depth: u32,
    path: PathBuf,
    entry: Option<DirEntry>,
}

pub enum SymlinkResolveOutcome {
    NotSymlink,
    SkipSymlink,
    FollowSymlink,
    AlreadyTraversed,
    CanonicalizeFailed,
    ReadLinkFailed,
}

impl SearchPath {
    pub fn to_path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn depth(&self) -> u32 {
        self.depth
    }

    pub fn resolve_symlinks(
        &mut self,
        symlink_behaviour: &mut SymlinkBehaviour,
    ) -> SymlinkResolveOutcome {
        match symlink_behaviour {
            SymlinkBehaviour::Skip => {
                let is_symlink = match &self.entry {
                    Some(entry) => SearchPath::is_metadata_symlink(entry.metadata()),
                    None => SearchPath::is_metadata_symlink(fs::symlink_metadata(self.to_path())),
                };

                if is_symlink {
                    SymlinkResolveOutcome::SkipSymlink
                } else {
                    SymlinkResolveOutcome::NotSymlink
                }
            }
            SymlinkBehaviour::Follow(follow_state) => match fs::read_link(self.to_path()) {
                Ok(path) => {
                    if let Ok(absolute) = path.canonicalize() {
                        self.path = absolute;
                        if SearchPath::check_for_cycles(follow_state, &self.path) {
                            SymlinkResolveOutcome::AlreadyTraversed
                        } else {
                            SymlinkResolveOutcome::FollowSymlink
                        }
                    } else {
                        SymlinkResolveOutcome::CanonicalizeFailed
                    }
                }
                Err(error) => match error.kind() {
                    // InvalidInput == "That's not a symlink."
                    ErrorKind::InvalidInput => {
                        if SearchPath::check_for_cycles(follow_state, &self.path) {
                            SymlinkResolveOutcome::AlreadyTraversed
                        } else {
                            SymlinkResolveOutcome::NotSymlink
                        }
                    }
                    _ => SymlinkResolveOutcome::ReadLinkFailed,
                },
            },
        }
    }

    fn check_for_cycles(follow_state: &mut FollowState, path: &Path) -> bool {
        if follow_state.is_seen(path) {
            return true;
        }

        follow_state.mark_seen(path);
        false
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
