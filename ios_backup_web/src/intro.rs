use std::collections::HashMap;

use leptos::{IntoView, component, ev::Targeted, html::Input, prelude::*, view};
use web_sys::{Event, File, HtmlInputElement};

use crate::utils::index_files_by_name;

#[component]
pub fn Intro(set_files: WriteSignal<Option<HashMap<String, File>>, LocalStorage>) -> impl IntoView {
    let file_input: NodeRef<Input> = NodeRef::new();

    Effect::new(move |_| {
        if let Some(file_input) = file_input.get() {
            file_input.set_attribute("webkitdirectory", "").unwrap();
        }
    });

    let change_file_input = move |event: Targeted<Event, HtmlInputElement>| {
        if let Some(files) = event.target().files() {
            set_files.set(Some(index_files_by_name(&files)));
        }
    };

    view! {
        <div>
            <h1>"iOS Backup Explorer"</h1>
            <p>
                "This tool allows you to explore the contents of an iOS backup
                by selecting the backup folder from your filesystem." <br />
                "It respects your privacy by processing files locally in your browser.
                It won't upload any data to external servers.
                It doesn't run any server-side code." <br />
                "It's free, there's nothing to download, and no installation is required."
            </p>
            <p class="tip">
                "Tip: on macOS, backup folders are typically located at "
                <code>"~/Library/Application Support/MobileSync/Backup/"</code>.
            </p>
            <div class="file-input-button">
                <label for="file-input">"Select iOS Backup Folder"</label>
                <input
                    id="file-input"
                    type="file"
                    multiple
                    // webkitdirectory
                    node_ref=file_input
                    on:change:target=change_file_input
                />
            </div>
        </div>
    }
}
