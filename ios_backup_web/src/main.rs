mod directory;
mod export;
mod extensions;
mod intro;
mod utils;

use std::collections::HashMap;

use ios_backup_core::infer::FileHeaderReader;
use ios_backup_core::parse;
use leptos::reactive::spawn_local;
use leptos::{prelude::*, view};
use tokio::io::{AsyncReadExt, BufReader};
use tokio_util::compat::FuturesAsyncReadCompatExt;
use wasm_bindgen_futures::JsFuture;
use wasm_streams::ReadableStream;
use web_sys::File;

use crate::intro::Intro;
use crate::utils::is_file_visible;
use crate::{directory::HtmlDirectory, export::export, extensions::ExtensionFilter};

#[component]
fn App() -> impl IntoView {
    let (files, set_files) = signal_local::<Option<HashMap<String, File>>>(None);
    let (selected_extensions, set_selected_extensions) = signal_local::<Vec<String>>(vec![]);

    let records = LocalResource::new(move || async move {
        match files
            .read()
            .as_ref()
            .and_then(|f_map| f_map.get("Manifest.mbdb"))
        {
            Some(manifest) => {
                let mut reader = BufReader::new(
                    ReadableStream::from_raw(manifest.stream())
                        .into_async_read()
                        .compat(),
                );
                parse(
                    &mut reader,
                    &WebFileHeaderReader {
                        files: files.read().as_ref().unwrap(),
                    },
                )
                .await
                .ok()
            }
            None => None,
        }
    });

    let export = move |_| {
        spawn_local(async move {
            export(
                records
                    .read_untracked()
                    .as_ref()
                    .flatten()
                    .unwrap()
                    .iter()
                    .filter(|file_record| {
                        is_file_visible(file_record, &selected_extensions.read_untracked())
                    }),
                files.read_untracked().as_ref().unwrap(),
            )
            .await;
        });
    };

    let header = move || {
        view! {
            <header>
                <div class="btn-group">
                    <strong>"iOS Backup Explorer"</strong>
                    <div class="file-type-filter">
                        <strong class="btn">"Filter by file type"</strong>
                        <div>
                            <ExtensionFilter
                                file_records=records
                                selected_extensions=selected_extensions
                                set_selected_extensions=set_selected_extensions
                            />
                        </div>
                    </div>
                    <div>
                        <strong class="btn" on:click=export>
                            Export
                        </strong>
                    </div>
                </div>
                <strong class="btn" on:click=move |_| set_files.set(None)>
                    "Close"
                </strong>
            </header>
        }
    };

    view! {
        <div>
            <Show when=move || files.read().is_none() fallback=header>
                <Intro set_files />
            </Show>
            <Show when=move || {
                files.read().is_some() && records.read().as_ref().flatten().is_none()
            }>
                <p>"Loading and parsing Manifest.mbdb..."</p>
            </Show>
            <Show when=move || records.read().as_ref().flatten().is_some()>
                <div id="file-explorer">
                    <HtmlDirectory mbdb_signal=records selected_extensions=selected_extensions />
                </div>
            </Show>
        </div>
    }
}

struct WebFileHeaderReader<'a> {
    files: &'a HashMap<String, web_sys::File>,
}

impl FileHeaderReader for WebFileHeaderReader<'_> {
    fn read_header(&self, file_name: &str) -> impl std::future::Future<Output = Vec<u8>> {
        let file = self.files.get(file_name).unwrap();
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let limit = (file.size()).min(8192f64) as usize;
        let mut buf = vec![0; limit];
        let mut reader = ReadableStream::from_raw(file.stream())
            .into_async_read()
            .compat();
        async move {
            reader.read_exact(&mut buf).await.unwrap();
            buf
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| {
        spawn_local(async {
            JsFuture::from(window().navigator().service_worker().register("./sw.js"))
                .await
                .unwrap();
        });
        view! { <App /> }
    });
}
