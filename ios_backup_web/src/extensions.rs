use std::str::FromStr as _;

use ios_backup_core::{FileRecord, gather_extensions, infer::extension_type::ExtensionType};
use leptos::{prelude::*, view};
use web_sys::{HtmlCollection, HtmlOptionElement, wasm_bindgen::JsCast};

#[component]
pub fn ExtensionFilter(
    file_records: LocalResource<Option<Vec<FileRecord>>>,
    selected_extensions: ReadSignal<Vec<String>, LocalStorage>,
    set_selected_extensions: WriteSignal<Vec<String>, LocalStorage>,
) -> impl IntoView {
    let extensions = Memo::new(move |_| {
        file_records
            .read()
            .as_ref()
            .flatten()
            .map(|file_records| gather_extensions(file_records))
    });

    let on_change_extensions = move |selected_options: HtmlCollection| {
        let mut selected = vec![];
        for i in 0..selected_options.length() {
            let option: HtmlOptionElement = selected_options.item(i).unwrap().dyn_into().unwrap();
            match option.value().chars().next() {
                Some('.') => selected.push({
                    let mut extension = option.value();
                    extension.remove(0);
                    extension
                }),
                _ => {
                    extensions
                        .read()
                        .as_ref()
                        .unwrap()
                        .get(&ExtensionType::from_str(&option.value()).unwrap())
                        .unwrap()
                        .1
                        .iter()
                        .for_each(|(ext, _)| selected.push(ext.clone()));
                }
            }
        }
        selected.sort();
        selected.dedup();
        set_selected_extensions.set(selected);
    };

    view! {
        <Show when=move || extensions.read().is_some()>
            <select
                multiple
                on:change:target=move |ev| {
                    on_change_extensions(ev.target().selected_options());
                }
            >
                <label>"Filter by file type:"</label>
                <For each=move || extensions.get().unwrap() key=|group| group.0 let(group)>
                    <option
                        value=group.0.to_string()
                        prop:selected=move || selected_extensions.with(move |_| false)
                    >
                        {move || format!("{} ({} files)", group.0, group.1.0)}
                    </option>
                    <For each=move || group.1.1.clone() key=|ext| ext.0.clone() let(ext)>
                        <option
                            value=format!(".{}", ext.0)
                            prop:selected=move || selected_extensions.read().contains(&ext.0)
                        >
                            {format!("-> {} ({} files)", ext.0, ext.1)}
                        </option>
                    </For>
                </For>
            </select>
        </Show>
    }
}
