use core::future::Future;

use infer::MatcherType;

use crate::{infer::extension_type::ExtensionType, mbdb::FileRecord};

pub mod extension_type;

#[derive(Debug, Clone)]
pub struct InferredExtension {
    pub extension: String,
    pub extension_type: ExtensionType,
}

pub trait FileHeaderReader {
    fn read_header(&self, file_name: &str) -> impl Future<Output = Vec<u8>>;
}

pub async fn file_type<R: FileHeaderReader>(
    file_record: &FileRecord,
    file_header_reader: &R,
) -> InferredExtension {
    let header = file_header_reader.read_header(&file_record.file_id).await;
    infer::get(&header).map_or_else(
        || {
            let extension = file_record
                .path
                .extension()
                .or_else(|| file_record.path.file_name())
                .map_or_else(
                    || file_record.path.to_string_lossy().to_string(),
                    |ext| ext.to_string_lossy().to_string(),
                );
            InferredExtension {
                extension,
                extension_type: ExtensionType(MatcherType::Custom),
            }
        },
        |ft| InferredExtension {
            extension: ft.extension().to_owned(),
            extension_type: ExtensionType(ft.matcher_type()),
        },
    )
}
