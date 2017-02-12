use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn test_files(test_dir: &Path) -> Vec<PathBuf> {
    let mut test_files = vec![];

    for entry in WalkDir::new(&test_dir) {
        let entry = entry.unwrap();
        let path = entry.path();

        if let Some(extension) = path.extension() {
            if path.is_file() && (extension == "jpg" || extension == "jpeg") {
                test_files.push(path.to_owned());
            }
        }
    }

    if let Ok(file) = File::open(test_dir.join("disabled.list")) {
        for line in BufReader::new(file).lines() {
            let line = line.unwrap();

            if line.is_empty() || line.starts_with("#") {
                continue;
            }

            let path = test_dir.join(Path::new(&line));

            if !test_files.contains(&path) {
                panic!("The file {:?} specified in {:?} could not be found among the files being tested", line, test_dir.join("disabled.txt"));
            }

            let position = test_files.iter().position(|p| p == &path).unwrap();
            test_files.remove(position);
        }
    }

    test_files
}
