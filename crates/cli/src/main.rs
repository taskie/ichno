extern crate treblo;

use std::env;

use std::{
    ffi::{OsStr, OsString},
    path::Path,
};

use crate::treblo::{hex::to_hex_string, walk};

fn main() {
    let args: Vec<OsString> = env::args_os().collect();
    let path = if args.len() > 1 { args[1].to_owned() } else { OsString::from(".") };
    let path = Path::new(&path);
    let w = ignore::WalkBuilder::new(path).hidden(false).filter_entry(|p| p.file_name() != OsStr::new(".git")).build();
    walk::walk(path, w, &mut |p, e, is_tree| {
        let object_type = if is_tree { "tree" } else { "blob" };
        let path = if args.len() > 1 { p } else { p.strip_prefix(path).unwrap() };
        println!(
            "{:06o} {} {}\t{}",
            e.file_mode.as_i32(),
            object_type,
            to_hex_string(e.digest.as_slice()),
            path.display()
        )
    });
}
