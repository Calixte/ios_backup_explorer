mod directory_node;

use std::iter::once;

use byte_unit::{Byte, UnitType};
use ios_backup_core::FileRecord;
use leptos::{IntoView, component, prelude::*, view};

use crate::directory::directory_node::{DirectoryNode, TogglableFileRecord};

#[component]
pub fn HtmlDirectory(
    mbdb_signal: LocalResource<Option<Vec<FileRecord>>>,
    selected_extensions: ReadSignal<Vec<String>, LocalStorage>,
) -> impl IntoView {
    let directory_tree = Memo::new(move |_| {
        let mut tree = DirectoryNode::new(String::new(), String::new(), vec![], vec![]);
        mbdb_signal
            .read()
            .as_ref()
            .flatten()
            .unwrap()
            .iter()
            .for_each(|file| {
                let mut current_node = &mut tree;
                let mut parts = once(file.mbdb.domain.clone())
                    .chain(
                        file.mbdb
                            .path
                            .iter()
                            .map(|part| part.to_string_lossy().to_string()),
                    )
                    .peekable();
                while let Some(part) = parts.next() {
                    if parts.peek().is_none() {
                        break;
                    }
                    let pos = current_node.children.iter().position(|c| *c.name == *part);
                    current_node = if let Some(index) = pos {
                        &mut current_node.children[index]
                    } else {
                        current_node.children.push(DirectoryNode::new(
                            format!("{}/{}", current_node.full_name, part),
                            part,
                            vec![],
                            vec![],
                        ));
                        current_node.children.last_mut().unwrap()
                    };
                }
                current_node.files.push(TogglableFileRecord {
                    file_record: file.clone(),
                    visible: RwSignal::new(true),
                });
            });
        tree
    });

    let file_count = Memo::new(move |_| mbdb_signal.read().as_ref().flatten().unwrap().len());

    let (filtered_file_count, set_filtered_file_count) = signal_local(None);

    Effect::new(move |_| {
        let file_count = directory_tree
            .read()
            .filter_directory_node(&selected_extensions.read());
        set_filtered_file_count.set(Some(file_count));
    });

    view! {
        <div id="mbdb-entries">
            <p>
                "Displaying "
                <b>
                    {move || filtered_file_count.read().unwrap_or(*file_count.read()).to_string()}
                </b>
                <Show when=move || {
                    filtered_file_count
                        .read()
                        .map(|filtered_file_count| file_count.read() != filtered_file_count)
                        .unwrap_or(false)
                }>" of " <b>{move || file_count.read().to_string()}</b></Show> " files"
            </p>
            <HtmlDirectoryNode node=Signal::derive_local(move || directory_tree.get()) />
        </div>
    }
}

#[component]
fn Listings(node_signal: Signal<DirectoryNode, LocalStorage>) -> impl IntoView {
    view! {
        <ul>
            <For
                each=move || node_signal.get().files
                key=|file| file.file_record.mbdb.file_id.clone()
                children=move |file| {
                    let file_record = file.file_record.clone();
                    view! {
                        <Show when=move || { file.visible.with(|v| *v) }>
                            <li>
                                {file_record
                                    .mbdb
                                    .path
                                    .file_name()
                                    .unwrap_or_else(|| file_record.mbdb.path.as_os_str())
                                    .to_string_lossy()
                                    .to_string()}
                                <span>
                                    {format!(
                                        "{:#.0}",
                                        Byte::from_u64(file_record.mbdb.file_length)
                                            .get_appropriate_unit(UnitType::Decimal),
                                    )}
                                </span>
                            </li>
                        </Show>
                    }
                }
            />
            <For
                each=move || node_signal.get().children
                key=|child| child.full_name.clone()
                children=move |child| {
                    view! {
                        <Show when=move || { child.visible.with(|v| *v) }>
                            <li>
                                <HtmlDirectoryNode node=Signal::stored_local(child.clone()) />
                            </li>
                        </Show>
                    }
                }
            />
        </ul>
    }
}

#[component]
fn HtmlDirectoryNode(node: Signal<DirectoryNode, LocalStorage>) -> impl IntoView {
    view! {
        <Show
            when=move || node.with(|n| n.name.is_empty())
            fallback=move || {
                view! {
                    <details>
                        <summary>{node.read().name.clone()}</summary>
                        <Listings node_signal=node />
                    </details>
                }
            }
        >
            <Listings node_signal=node />
        </Show>
    }
    .into_any()
}
