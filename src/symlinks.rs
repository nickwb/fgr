use std::collections::HashSet;
use std::path::{Path, PathBuf};

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
