#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, Window};
use zbus::Connection;

#[tauri::command]
fn toggle_window(window: Window) {
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
        println!("Window hidden");
    } else {
        let _ = window.show();
        let _ = window.set_focus();
        println!("Window shown");
    }
}

async fn listen_dbus(window: Window) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::session().await?;
    println!("Connected to D-Bus session bus");

    // zbus v4 SignalStream API
    use zbus::proxy;
    use futures_util::stream::StreamExt;

    #[proxy(
        interface = "io.github.noppomario.uti.DoubleTap",
        default_path = "/io/github/noppomario/uti/DoubleTap"
    )]
    trait DoubleTap {
        #[zbus(signal)]
        fn triggered(&self) -> zbus::Result<()>;
    }

    let proxy = DoubleTapProxy::new(&conn).await?;
    let mut stream = proxy.receive_triggered().await?;
    println!("Listening for D-Bus signals...");

    while let Some(_signal) = stream.next().await {
        println!("D-Bus signal received!");
        let _ = window.emit("double-ctrl-pressed", ());
    }

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![toggle_window])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            let window_clone = window.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = listen_dbus(window_clone).await {
                    eprintln!("D-Bus error: {}", e);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
