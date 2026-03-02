use std::collections::HashMap;

use ios_backup_core::FileRecord;
use web_sys::{File, FileList};

pub fn is_file_visible(file: &FileRecord, extensions: &[String]) -> bool {
    extensions.is_empty() || extensions.contains(&file.inferred_extension.extension)
}

pub fn index_files_by_name(files: &FileList) -> HashMap<String, File> {
    let mut file_map = HashMap::new();
    for i in 0..files.length() {
        let file = files.get(i).unwrap();
        file_map.insert(file.name(), file);
    }
    file_map
}
