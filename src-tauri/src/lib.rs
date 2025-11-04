use serde::{Serialize};
use specta::Type;
use specta_typescript::Typescript;
use tauri_specta::{collect_commands, Builder, collect_events, Event};
use tauri::{AppHandle, ipc::Channel};

/// Greets the user with a personalized message from Rust.
///
/// # Arguments
/// * `name` - A string slice representing the user's name.
///
/// # Returns
/// A `String` containing the greeting message.
#[tauri::command]
#[specta::specta]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Clone, Serialize, Event, Type)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase", tag = "event", content = "data")]
enum DownloadEvent<'a> {
  Started {
    url: &'a str,
    download_id: usize,
    content_length: usize,
  },
  Progress {
    download_id: usize,
    chunk_length: usize,
  },
  Finished {
    download_id: usize,
  },
}

#[tauri::command]
#[specta::specta]
fn download(_app: AppHandle, url: String, on_event: Channel<DownloadEvent>) {
    let content_length = 1000;
    let download_id = 1;

    println!("Download started for URL: {}, ID: {}, Content Length: {}", url, download_id, content_length);
    on_event.send(DownloadEvent::Started {
        url: &url,
        download_id,
        content_length,
    }).unwrap();

    for chunk_length in [15, 150, 35, 500, 300] {
        println!("Progress: Downloaded chunk of {} bytes for ID: {}", chunk_length, download_id);
        on_event.send(DownloadEvent::Progress {
            download_id,
            chunk_length,
        }).unwrap();
    }

    println!("Download finished for ID: {}", download_id);
    on_event.send(DownloadEvent::Finished { download_id }).unwrap();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if std::env::args().any(|arg| arg == "--specta") {
        export_bindings();
        return;
    }

    let builder = export_bindings();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


fn export_bindings() -> Builder<tauri::Wry> {
    let builder = Builder::<tauri::Wry>::new()
        .commands(collect_commands![greet, download])
        .events(collect_events![DownloadEvent]);

    builder
        .export(Typescript::default()
        .bigint(specta_typescript::BigIntExportBehavior::Number), "../src/bindings.ts")
        .expect("error while exporting typescript bindings");

    builder
}
