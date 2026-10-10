use rayon::prelude::*;
use std::{
    cmp::{Ordering, Reverse},
    collections::{BinaryHeap, HashMap},
    env, io,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Eq, PartialEq)]
struct DirectoryInfo {
    size: u64,
    path: PathBuf,
}

impl Ord for DirectoryInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.size
            .cmp(&other.size)
            .then_with(|| self.path.cmp(&other.path))
    }
}

impl PartialOrd for DirectoryInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn convert(size: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

fn normalize(values: &[u64]) -> Vec<usize> {
    let min = *values.iter().min().unwrap_or(&0);
    let max = *values.iter().max().unwrap_or(&0);
    let range = u128::from(max - min);

    if range == 0 {
        return vec![1; values.len()];
    }

    values
        .iter()
        .map(|&value| {
            let scaled = (u128::from(value - min) * 41 + range / 2) / range + 1;
            scaled.clamp(1, 42) as usize
        })
        .collect()
}

fn add_size(sizes: &mut HashMap<PathBuf, u64>, path: PathBuf, size: u64) -> io::Result<()> {
    let total = sizes.entry(path).or_insert(0);
    *total = total
        .checked_add(size)
        .ok_or_else(|| io::Error::other("directory size exceeds u64"))?;
    Ok(())
}

fn get_sizes(path: &Path) -> io::Result<HashMap<PathBuf, u64>> {
    if !path.metadata()?.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "input path is not a directory",
        ));
    }

    WalkDir::new(path)
        .into_iter()
        .par_bridge()
        .try_fold(HashMap::new, |mut sizes, entry| {
            let entry = entry.map_err(io::Error::other)?;
            if entry.file_type().is_dir() {
                sizes.entry(entry.path().to_path_buf()).or_insert(0);
            } else if entry.file_type().is_file() {
                let size = entry.metadata().map_err(io::Error::other)?.len();
                let mut parent = entry.path().parent();
                while let Some(directory) = parent {
                    if !directory.starts_with(path) {
                        break;
                    }
                    add_size(&mut sizes, directory.to_path_buf(), size)?;
                    if directory == path {
                        break;
                    }
                    parent = directory.parent();
                }
            }
            Ok(sizes)
        })
        .try_reduce(HashMap::new, |mut left, right| {
            for (path, size) in right {
                add_size(&mut left, path, size)?;
            }
            Ok(left)
        })
}

fn main() {
    let mut paths: Vec<_> = env::args_os().skip(1).collect();
    let count = paths
        .pop()
        .and_then(|s| s.to_str().and_then(|s| s.parse().ok()))
        .unwrap_or(10);
    if paths.is_empty() {
        eprintln!("Usage: sugar <PATH> [PATH ...]");
        std::process::exit(2);
    }

    let sizes = paths
        .par_iter()
        .map(|path| {
            get_sizes(Path::new(path)).map_err(|error| {
                io::Error::new(
                    error.kind(),
                    format!("{}: {error}", Path::new(path).display()),
                )
            })
        })
        .try_reduce(HashMap::new, |mut left, right| {
            for (path, size) in right {
                let total = left.entry(path).or_insert(size);
                *total = (*total).max(size);
            }
            Ok(left)
        });

    match sizes {
        Ok(sizes) => {
            let mut directories = BinaryHeap::new();
            for (path, size) in sizes {
                directories.push(Reverse(DirectoryInfo { size, path }));
                if directories.len() > count {
                    directories.pop();
                }
            }
            let mut directories: Vec<_> = directories
                .into_iter()
                .map(|Reverse(directory)| directory)
                .collect();
            directories.sort_by(|left, right| {
                right
                    .size
                    .cmp(&left.size)
                    .then_with(|| left.path.cmp(&right.path))
            });

            let sizes = directories
                .iter()
                .map(|directory| directory.size)
                .collect::<Vec<_>>();
            let mut nsizes = normalize(&sizes);
            nsizes.reverse();

            println!("SIZE \t\t\t\tDIRECTORY");
            for directory in directories {
                let filename = directory.path.to_str().unwrap_or("<invalid UTF-8>");
                let characters: Vec<_> = filename.chars().collect();
                let progressbar = "▓".repeat(nsizes.pop().unwrap_or(1) as usize);
                let slice = if characters.len() > 58 {
                    characters[..23].iter().collect::<String>()
                        + "..."
                        + &characters[(characters.len() - 32)..]
                            .iter()
                            .collect::<String>()
                        + " " + &progressbar
                } else {
                    String::from(filename) + " " + &" ".repeat(58-characters.len()) + &progressbar
                };
                println!("{} \t\t\t{}", convert(directory.size), slice);
            }
        }
        Err(error) => {
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
    }
}
