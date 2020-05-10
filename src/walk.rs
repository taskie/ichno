use ignore;
use sha1::Digest;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use crate::object::{blob_from_path, tree_from_entries, FileMode, TreeEntry};

fn resolve<P, F>(resolving_map: &mut BTreeMap<PathBuf, TreeEntry>, parent: P, f: &mut F)
where
    P: AsRef<Path>,
    F: FnMut(&Path, &TreeEntry, bool) -> (),
{
    let mut paths = Vec::new();
    let mut entries = Vec::new();
    for (path, entry) in resolving_map.range(parent.as_ref().to_owned()..).into_iter() {
        if !path.starts_with(parent.as_ref()) {
            break;
        }
        paths.push(path.clone());
        entries.push(entry.clone());
    }
    entries.sort_by_key(|e| {
        let mut bs = e.name.as_bytes().to_vec();
        if e.file_mode == FileMode::DIR {
            bs.push('/' as u8);
        }
        bs
    });
    let mut sha1 = sha1::Sha1::new();
    tree_from_entries(&mut sha1, entries.iter()).unwrap();
    let digest = sha1.result().to_vec();
    for path in paths.iter() {
        resolving_map.remove(path);
    }
    let name = parent.as_ref().file_name().unwrap_or_default();
    let parent_entry = TreeEntry::new(FileMode::DIR, name.to_str().unwrap().to_owned(), digest);
    f(parent.as_ref(), &parent_entry, true);
    resolving_map.insert(parent.as_ref().to_owned(), parent_entry);
}

pub fn walk<P: AsRef<Path>, F>(path: P, walk: ignore::Walk, f: &mut F)
where
    F: FnMut(&Path, &TreeEntry, bool) -> (),
{
    let mut resolving_map = BTreeMap::<PathBuf, TreeEntry>::new();
    let mut walk_state = WalkState::new(path.as_ref().to_owned());
    for result in walk {
        match result {
            Ok(entry) => {
                let file_mode = FileMode::from(entry.metadata().unwrap());
                if file_mode != FileMode::DIR {
                    let mut sha1 = sha1::Sha1::new();
                    blob_from_path(&mut sha1, entry.path()).unwrap();
                    let digest = sha1.result().to_vec();
                    let path = entry.path();
                    let name = path.file_name().unwrap().to_str().unwrap().to_owned();
                    let te = TreeEntry::new(file_mode, name, digest);
                    f(path, &te, false);
                    resolving_map.insert(path.to_owned(), te);
                    walk_state.process(Some(&path), &mut |p| resolve(&mut resolving_map, p, f));
                }
            }
            Err(err) => eprintln!("ERROR: {}", err),
        }
    }
    walk_state.process::<&Path, _>(None, &mut |p| resolve(&mut resolving_map, p, f));
}

struct WalkState<T> {
    root: T,
    parent_stack: Vec<T>,
}

impl WalkState<PathBuf> {
    pub fn new(root: PathBuf) -> WalkState<PathBuf> {
        let parent_stack = vec![root.clone()];
        WalkState { root, parent_stack }
    }

    pub fn process<P, F>(&mut self, item: Option<P>, f: &mut F)
    where
        P: AsRef<Path>,
        F: FnMut(&Path) -> (),
    {
        let parent = item.and_then(|p| p.as_ref().parent().map(|p| p.to_owned()));
        if let Some(parent) = parent {
            while !self.parent_stack.is_empty() {
                let last = self.parent_stack.last().unwrap();
                if &parent == last || parent.starts_with(last) {
                    break;
                }
                f(&last);
                self.parent_stack.pop();
            }
            let last = self.parent_stack.last();
            let mut parents = Vec::new();
            let mut cur = parent;
            loop {
                if last.is_some() && &cur == last.unwrap() {
                    break;
                }
                if !cur.starts_with(&self.root) {
                    break;
                }
                parents.push(cur.clone());
                if let Some(next) = cur.parent() {
                    cur = next.to_owned();
                } else {
                    break;
                }
            }
            for pb in parents.iter().rev() {
                self.parent_stack.push(pb.to_owned());
            }
        } else {
            while !self.parent_stack.is_empty() {
                let last = self.parent_stack.pop().unwrap();
                f(&last);
            }
        }
    }
}
