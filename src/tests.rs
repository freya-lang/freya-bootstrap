use std::fs::{File, read_dir};
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::parser::parse;
use crate::tokenizer::tokenize;

#[test]
fn test_stdlib() {
	let root_path: PathBuf = "./stdlib".into();
	let mut files: Vec<PathBuf> = Vec::new();

	walk_folder(&root_path, &mut files);

	for path in files {
		let mut file = File::open(path).unwrap();
		let mut contents = String::new();
		file.read_to_string(&mut contents).unwrap();

		let tokenization_output = tokenize(&contents).unwrap();
		let parsed = parse(tokenization_output).unwrap();
	}
}

fn walk_folder(folder: &Path, paths: &mut Vec<PathBuf>) {
	let contents = read_dir(folder).unwrap();

	for entry in contents {
		let entry = entry.unwrap();
		let file_type = entry.file_type().unwrap();
		let path = entry.path();

		if file_type.is_dir() {
			walk_folder(&path, paths);
		} else if file_type.is_file() {
			paths.push(path);
		} else {
			panic!("file entry {path:?} neither a directory or a file");
		}
	}
}
