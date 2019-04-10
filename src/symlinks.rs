use std::collections::HashSet;
use std::path::{Path, PathBuf};

arg_enum! {
    #[derive(PartialEq, Debug)]
    pub enum SymlinkOption {
        Skip,
        Follow
    }
}

pub enum SymlinkBehaviour {
    Skip,
    Follow(FollowState),
}

pub struct FollowState {
    seen_items: HashSet<PathBuf>,
}

impl FollowState {
    pub fn new() -> FollowState {
        FollowState {
            seen_items: HashSet::new(),
        }
    }

    pub fn is_seen(&self, path: &Path) -> bool {
        self.seen_items.contains(path)
    }

    pub fn mark_seen(&mut self, path: &Path) {
        self.seen_items.insert(path.to_path_buf());
    }
}
