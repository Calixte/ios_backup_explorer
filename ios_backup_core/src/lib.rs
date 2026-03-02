use core::cmp::Ordering;
use std::collections::HashMap;

use ::infer::MatcherType;
use anyhow::Result;
use futures::future::join_all;
use indexmap::IndexMap;
use tokio::io::AsyncRead;

use crate::infer::{FileHeaderReader, InferredExtension, extension_type::ExtensionType, file_type};

pub mod infer;
pub mod mbdb;

#[derive(Debug, Clone)]
pub struct FileRecord {
    pub inferred_extension: InferredExtension,
    pub mbdb: mbdb::FileRecord,
}

pub async fn parse<T: AsyncRead + Unpin, R: FileHeaderReader>(
    reader: &mut T,
    file_header_reader: &R,
) -> Result<Vec<FileRecord>> {
    Ok(join_all(
        mbdb::parse(reader)
            .await?
            .into_iter()
            .map(async |mbdb_file_record| FileRecord {
                inferred_extension: file_type(&mbdb_file_record, file_header_reader).await,
                mbdb: mbdb_file_record,
            }),
    )
    .await)
}

#[must_use]
pub fn gather_extensions(
    file_records: &[FileRecord],
) -> IndexMap<ExtensionType, (u32, Vec<(String, u32)>)> {
    let mut indexed_file_types = file_records
        .iter()
        .map(|file_record| &file_record.inferred_extension)
        .fold(HashMap::new(), |mut acc, inferred_extension| {
            acc.entry(inferred_extension.extension_type)
                .and_modify(
                    |&mut (ref mut total_count, ref mut extensions): &mut (
                        u32,
                        HashMap<String, u32>,
                    )| {
                        *total_count += 1;
                        extensions
                            .entry(inferred_extension.extension.clone())
                            .and_modify(|count| *count += 1)
                            .or_insert(1);
                    },
                )
                .or_insert_with(move || {
                    (
                        1,
                        HashMap::from([(inferred_extension.extension.clone(), 1)]),
                    )
                });
            acc
        })
        .into_iter()
        .map(|(extension_type, (total_count, extensions))| {
            let mut sorted_extensions = extensions.into_iter().collect::<Vec<_>>();
            sorted_extensions.sort_by_key(|&(_, count)| count);
            sorted_extensions.reverse();
            (extension_type, (total_count, sorted_extensions))
        })
        .collect::<IndexMap<_, _>>();

    let priority = |extension_type: &ExtensionType| match *extension_type {
        ExtensionType(MatcherType::Custom) => 1u8,
        _ => 0u8,
    };

    indexed_file_types.sort_by(
        |&extension_type_a, &(total_count_a, _), &extension_type_b, &(total_count_b, _)| {
            match priority(&extension_type_a).cmp(&priority(&extension_type_b)) {
                Ordering::Equal => total_count_b.cmp(&total_count_a),
                other => other,
            }
        },
    );

    indexed_file_types
}
