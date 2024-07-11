use std::fs::read_dir;
use std::path::Path;

fn list_files_in_directory<P: AsRef<Path>>(dir_path: P) {
    if let Ok(entries) = read_dir(dir_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                println!("{}", entry.file_name().into_string().unwrap_or_else(|_| "<non-UTF-8 filename>".to_string()));
            }
        }
    }
}

fn main() {
    let directory_path = "/home/akaman/os_code/os/src";
    list_files_in_directory(directory_path);
}