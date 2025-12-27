use evdev::{Device, EventType, Key};
use zbus::Connection;
use std::time::{Duration, Instant};

const DOUBLE_TAP_INTERVAL: Duration = Duration::from_millis(300);

async fn notify_double_ctrl(conn: &Connection) -> zbus::Result<()> {
    conn.emit_signal(
        None::<()>,
        "/io/github/noppomario/uti/DoubleTap",
        "io.github.noppomario.uti.DoubleTap",
        "Triggered",
        &(),
    ).await?;
    println!("D-Bus signal sent: Triggered");
    Ok(())
}

fn find_keyboard_device() -> std::io::Result<std::path::PathBuf> {
    for entry in std::fs::read_dir("/dev/input")? {
        let path = entry?.path();
        if let Some(name) = path.file_name() {
            if name.to_string_lossy().starts_with("event") {
                if let Ok(device) = Device::open(&path) {
                    if device.supported_keys().map_or(false, |keys|
                        keys.contains(Key::KEY_A)) {
                        println!("Found keyboard device: {}", path.display());
                        return Ok(path);
                    }
                }
            }
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No keyboard found"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Double Ctrl daemon starting...");

    let device_path = find_keyboard_device()?;
    let mut device = Device::open(&device_path)?;
    let conn = Connection::session().await?;

    println!("Monitoring device: {}", device.name().unwrap_or("unknown"));
    println!("Connected to D-Bus session bus");
    println!("Waiting for double Ctrl press...");

    let mut last_ctrl_release: Option<Instant> = None;

    loop {
        for event in device.fetch_events()? {
            if event.event_type() != EventType::KEY {
                continue;
            }

            let key = Key::new(event.code());
            if key != Key::KEY_LEFTCTRL && key != Key::KEY_RIGHTCTRL {
                continue;
            }

            if event.value() == 0 { // キーリリース
                let now = Instant::now();

                if let Some(last) = last_ctrl_release {
                    if now.duration_since(last) < DOUBLE_TAP_INTERVAL {
                        println!("Double Ctrl detected!");
                        notify_double_ctrl(&conn).await?;
                        last_ctrl_release = None;
                        continue;
                    }
                }

                last_ctrl_release = Some(now);
            }
        }
    }
}
