use ios_backup_core::FileRecord;
use leptos::prelude::{Read, RwSignal, Set};

use crate::utils::is_file_visible;

#[derive(Clone, Debug)]
pub struct TogglableFileRecord {
    pub file_record: FileRecord,
    pub visible: RwSignal<bool>,
}

#[derive(Clone, Debug)]
pub struct DirectoryNode {
    pub full_name: String,
    pub name: String,
    pub children: Vec<Self>,
    pub files: Vec<TogglableFileRecord>,
    pub visible: RwSignal<bool>,
}

impl DirectoryNode {
    pub fn new(
        full_name: String,
        name: String,
        children: Vec<Self>,
        files: Vec<TogglableFileRecord>,
    ) -> Self {
        Self {
            full_name,
            name,
            children,
            files,
            visible: RwSignal::new(true),
        }
    }

    pub fn filter_directory_node(&self, extensions: &Vec<String>) -> usize {
        let mut file_count = 0;
        self.children.iter().for_each(|child| {
            file_count += child.filter_directory_node(extensions);
        });
        self.files.iter().for_each(|file| {
            let visible = is_file_visible(&file.file_record, extensions);
            file.visible.set(visible);
            if visible {
                file_count += 1;
            }
        });
        self.visible.set(
            self.children.iter().any(|child| *child.visible.read())
                || self.files.iter().any(|file| *file.visible.read()),
        );
        file_count
    }
}

impl PartialEq for DirectoryNode {
    fn eq(&self, other: &Self) -> bool {
        self.full_name == other.full_name
    }
}
