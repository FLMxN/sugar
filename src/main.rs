use rayon::prelude::*;
use std::{
    cmp::{Ordering, Reverse},
    collections::BinaryHeap,
    env, io,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Eq, PartialEq)]
struct FileInfo {
    size: u64,
    path: PathBuf,
}

impl Ord for FileInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.size
            .cmp(&other.size)
            .then_with(|| self.path.cmp(&other.path))
    }
}

impl PartialOrd for FileInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn add_to_top_files(files: &mut BinaryHeap<Reverse<FileInfo>>, file: FileInfo, count: usize) {
    files.push(Reverse(file));
    if files.len() > count {
        files.pop();
    }
}

fn get_top_files(path: &Path, count: usize) -> io::Result<BinaryHeap<Reverse<FileInfo>>> {
    WalkDir::new(path)
        .into_iter()
        .par_bridge()
        .try_fold(BinaryHeap::new, |mut files, entry| {
            let entry = entry.map_err(io::Error::other)?;
            if !entry.file_type().is_file() {
                return Ok(files);
            }

            let size = entry.metadata().map_err(io::Error::other)?.len();
            add_to_top_files(
                &mut files,
                FileInfo {
                    size,
                    path: entry.into_path(),
                },
                count
            );
            Ok(files)
        })
        .try_reduce(BinaryHeap::new, |mut left, right| {
            for Reverse(file) in right {
                add_to_top_files(&mut left, file, count);
            }
            Ok(left)
        })
}

fn main() {
    let mut paths: Vec<_> = env::args_os().skip(1).collect();
    let count = paths.pop().and_then(|s| s.to_str().and_then(|s| s.parse().ok())).unwrap_or(10);
    if paths.is_empty() {
        eprintln!("Usage: sugar <PATH> [PATH ...]");
        std::process::exit(2);
    }

    let files = paths
        .par_iter()
        .map(|path| {
            get_top_files(Path::new(path), count).map_err(|error| {
                io::Error::new(
                    error.kind(),
                    format!("{}: {error}", Path::new(path).display()),
                )
            })
        })
        .try_reduce(BinaryHeap::new, |mut left, right| {
            for Reverse(file) in right {
                add_to_top_files(&mut left, file, count);
            }
            Ok(left)
        });

    match files {
        Ok(files) => {
            let mut files: Vec<_> = files.into_iter().map(|Reverse(file)| file).collect();
            files.sort_by(|left, right| {
                right
                    .size
                    .cmp(&left.size)
                    .then_with(|| left.path.cmp(&right.path))
            });

            println!("SIZE \t\t\t\tPATH");
            for file in files {
                let filename = file.path.to_str().unwrap_or("<invalid UTF-8>");
                let lenght = filename.len();
                let slice = if lenght > 67 {
                    String::from(&filename[..42]) + "..." + &filename[lenght - 13..]
                } else {
                    String::from(filename)
                };
                println!("{} \t\t\t{}", file.size, slice);
            }
        }
        Err(error) => {
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
    }
}
