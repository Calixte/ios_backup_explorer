use std::collections::HashMap;

use async_zip::{Compression, ZipEntryBuilder, base::write::ZipFileWriter};
use futures_util::io::copy_buf;
use gloo_timers::callback::{Interval, Timeout};
use ios_backup_core::FileRecord;
use js_sys::{Array, Math};
use leptos::prelude::*;
use serde::Serialize;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use wasm_streams::ReadableStream;
use web_sys::wasm_bindgen::JsCast;
use web_sys::{File, HtmlIFrameElement};

#[derive(Serialize)]
#[serde(tag = "type")]
enum SwMessage {
    #[serde(rename = "PORT_TRANSFER")]
    PortTransfer {
        id: String,
        #[serde(with = "serde_wasm_bindgen::preserve")]
        stream: web_sys::ReadableStream,
    },
    #[serde(rename = "HEARTBEAT")]
    Heartbeat,
}

pub async fn export<'a, I>(file_records: I, files: &HashMap<String, File>)
where
    I: Iterator<Item = &'a FileRecord>,
{
    let (writer, reader) = tokio::io::duplex(1024 * 1024);

    let stream = ReadableStream::from_async_read(reader.compat(), 1024 * 1024).into_raw();

    let sw = window().navigator().service_worker().controller().unwrap();
    let id = Math::random().to_string();
    let message = SwMessage::PortTransfer {
        id: id.clone(),
        stream: stream.clone(),
    };

    let js_message = serde_wasm_bindgen::to_value(&message).unwrap();

    sw.post_message_with_transferable(&js_message, &Array::of1(&stream))
        .unwrap();

    let iframe: HtmlIFrameElement = document()
        .create_element("iframe")
        .unwrap()
        .dyn_into()
        .unwrap();
    iframe.set_attribute("style", "display:none").unwrap();
    iframe.set_src(&format!("./download-stream?id={id}"));
    document().body().unwrap().append_child(&iframe).unwrap();

    let heartbeat = serde_wasm_bindgen::to_value(&SwMessage::Heartbeat).unwrap();
    let heartbeat_interval = Interval::new(10_000, move || {
        sw.post_message(&heartbeat).unwrap();
    });

    let mut zip = ZipFileWriter::new(writer.compat_write());

    for file_record in file_records {
        let file_stream = files.get(&file_record.mbdb.file_id).unwrap().stream();
        let reader = futures_util::io::BufReader::with_capacity(
            1024 * 1024,
            ReadableStream::from_raw(file_stream).into_async_read(),
        );

        let mut entry_writer = zip
            .write_entry_stream(ZipEntryBuilder::new(
                file_record.mbdb.path.to_str().unwrap().into(),
                Compression::Stored,
            ))
            .await
            .unwrap();
        copy_buf(reader, &mut entry_writer).await.unwrap();
        entry_writer.close().await.unwrap();
    }
    zip.close().await.unwrap();
    drop(heartbeat_interval);
    Timeout::new(10_000, move || {
        iframe.remove();
    })
    .forget();
}
