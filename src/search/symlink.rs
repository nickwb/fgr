use std::collections::HashSet;
use std::error::Error;
use std::fs::{self, Metadata};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use crate::search::candidate::SearchCandidate;

pub enum SymlinkBehaviour {
    Skip,
    Follow(FollowState),
}

pub struct FollowState {
    seen_items: HashSet<PathBuf>,
}

pub struct SymlinkResolveResult {
    pub resolution: SymlinkResolveOutcome,
    pub updated_path: Option<PathBuf>,
}

pub enum SymlinkResolveOutcome {
    NotSymlink,
    SkipSymlink,
    FollowSymlink,
    AlreadyTraversed,
    CanonicalizeFailed(String),
    ReadLinkFailed(String),
}

impl FollowState {
    pub fn new() -> FollowState {
        FollowState {
            seen_items: HashSet::new(),
        }
    }

    pub fn check_already_visited_and_update(&mut self, path: &Path) -> bool {
        if self.is_seen(path) {
            return true;
        }

        self.mark_seen(path);
        false
    }

    fn is_seen(&self, path: &Path) -> bool {
        self.seen_items.contains(path)
    }

    fn mark_seen(&mut self, path: &Path) {
        self.seen_items.insert(path.to_path_buf());
    }
}

impl SymlinkBehaviour {
    pub fn resolve_candidate(&mut self, candidate: &SearchCandidate) -> SymlinkResolveResult {
        match self {
            SymlinkBehaviour::Skip => {
                let is_symlink = match candidate.dir_entry() {
                    Some(entry) => is_metadata_symlink(entry.metadata()),
                    None => is_metadata_symlink(fs::symlink_metadata(candidate.to_path())),
                };

                if is_symlink {
                    SymlinkResolveResult::from_outcome(SymlinkResolveOutcome::SkipSymlink)
                } else {
                    SymlinkResolveResult::from_outcome(SymlinkResolveOutcome::NotSymlink)
                }
            }
            SymlinkBehaviour::Follow(follow_state) => match fs::read_link(candidate.to_path()) {
                Ok(path) => match path.canonicalize() {
                    Ok(absolute) => {
                        if follow_state.check_already_visited_and_update(candidate.to_path()) {
                            SymlinkResolveResult::from_outcome_and_path(
                                SymlinkResolveOutcome::AlreadyTraversed,
                                absolute,
                            )
                        } else {
                            SymlinkResolveResult::from_outcome_and_path(
                                SymlinkResolveOutcome::FollowSymlink,
                                absolute,
                            )
                        }
                    }
                    Err(error) => SymlinkResolveResult::from_outcome(
                        SymlinkResolveOutcome::CanonicalizeFailed(String::from(
                            error.description(),
                        )),
                    ),
                },
                Err(error) => match error.kind() {
                    // InvalidInput == "That's not a symlink."
                    ErrorKind::InvalidInput => {
                        if follow_state.check_already_visited_and_update(candidate.to_path()) {
                            SymlinkResolveResult::from_outcome(
                                SymlinkResolveOutcome::AlreadyTraversed,
                            )
                        } else {
                            SymlinkResolveResult::from_outcome(SymlinkResolveOutcome::NotSymlink)
                        }
                    }
                    _ => SymlinkResolveResult::from_outcome(SymlinkResolveOutcome::ReadLinkFailed(
                        String::from(error.description()),
                    )),
                },
            },
        }
    }
}

impl SymlinkResolveResult {
    fn from_outcome(outcome: SymlinkResolveOutcome) -> SymlinkResolveResult {
        SymlinkResolveResult {
            updated_path: None,
            resolution: outcome,
        }
    }

    fn from_outcome_and_path(
        outcome: SymlinkResolveOutcome,
        updated_path: PathBuf,
    ) -> SymlinkResolveResult {
        SymlinkResolveResult {
            updated_path: Some(updated_path),
            resolution: outcome,
        }
    }
}

fn is_metadata_symlink(maybe_metadata: io::Result<Metadata>) -> bool {
    match maybe_metadata {
        Err(_e) => true,
        Ok(meta) => !meta.is_dir() && !meta.is_file(),
    }
}
