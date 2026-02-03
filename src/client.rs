use crate::DEBOUNCE_MS;
use crate::EventState;
use crate::HYPRLAND_SUBSCRIPTION;
use crate::UiEvent;
use crate::get_hypr_socket_path;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

pub async fn hyprland_event_listener(
    event_state: Arc<EventState>,
    sender: async_channel::Sender<UiEvent>,
) {
    let path = match get_hypr_socket_path() {
        Some(p) => p,
        None => {
            eprintln!("Could not determine Hyprland socket path");
            return;
        }
    };

    loop {
        match UnixStream::connect(&path).await {
            Ok(stream) => {
                if let Err(e) =
                    connect_to_hyprland_socket(stream, event_state.clone(), sender.clone()).await
                {
                    eprintln!("Connection to Hyprland socket lost: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Failed to connect to Hyprland socket: {}", e);
            }
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn connect_to_hyprland_socket(
    mut stream: UnixStream,
    event_state: Arc<EventState>,
    sender: async_channel::Sender<UiEvent>,
) -> std::io::Result<UnixStream> {
    println!("Connected to Hyprland socket");

    if let Err(e) = stream.write_all(HYPRLAND_SUBSCRIPTION.as_bytes()).await {
        eprintln!("Failed to subscribe to Hyprland events: {}", e);
        tokio::time::sleep(Duration::from_secs(1)).await;
        return Err(e);
    }

    let reader = BufReader::new(stream);
    let mut lines = reader.lines();

    let mut debounce = tokio::time::interval(Duration::from_millis(DEBOUNCE_MS));
    debounce.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let mut has_workspace_update = false;
    let mut is_fullscreen = false;
    let mut has_fullscreen_update = false;
    let mut latest_title: Option<String> = None;
    let mut has_workspace_urgent: Option<String> = None;
    let mut has_open_window: Option<String> = None;
    loop {
        tokio::select! {
            line_result = lines.next_line() => {
                match line_result {
                    Ok(Some(line)) => {
                        println!("Hyprland event: {}", line);
                        if line.contains("\"change\":") || line.contains("workspace") {
                            has_workspace_update = true;
                        }

                        if line.contains("fullscreen"){
                            has_fullscreen_update = true;
                            is_fullscreen = line.contains(">>1");
                        }

                        if line.contains("activewindow>>")
                            && let Some(title_start) = line.find(">>") {
                            latest_title = Some(line[title_start + 2..].trim().to_string());
                        }

                        if line.contains("urgent>>")
                            && let Some(start) = line.find("urgent>>") {
                            let urgent_id = line[start + 9..].trim().to_string();
                            has_workspace_urgent = Some(urgent_id);
                            has_workspace_update = true;
                        }

                        if line.contains("openwindow>>") {
                            let window_name = line.split(",").nth(2).unwrap_or("").trim().to_string();
                            has_open_window = Some(window_name);
                        }
                    }
                    Ok(None) => {
                        eprintln!("Hyprland socket closed");
                        return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Socket closed"));
                    }
                    Err(e) => {
                        eprintln!("Error reading from socket: {}", e);
                        return Err(e);
                    }
                }
            }

            _ = debounce.tick() => {
                let mut needs_gtk_update = false;

                if has_workspace_update {
                    sender.send(UiEvent::WorkspaceChanged).await.ok();
                    has_workspace_update = false;
                    needs_gtk_update = true;
                }

                if has_fullscreen_update {
                    sender.send(UiEvent::FullscreenChanged(is_fullscreen)).await.ok();
                    has_fullscreen_update = false;
                    needs_gtk_update = true;
                }

                if let Some(title) = latest_title.take() {
                    sender.send(UiEvent::TitleChanged(title.clone())).await.ok();
                    *event_state.pending_title.lock() = Some(title);
                    needs_gtk_update = true;
                }

                if let Some(urgent_id) = has_workspace_urgent.take() {
                    sender.send(UiEvent::WorkspaceUrgent(urgent_id.clone())).await.ok();
                    has_workspace_update = false;
                    needs_gtk_update = true;
                }

                if let Some(_window_name) = has_open_window.take() {
                    sender.send(UiEvent::WindowOpened(_window_name)).await.ok();
                    needs_gtk_update = true;
                }

                if needs_gtk_update {
                    glib::idle_add_once(|| {});
                }
            }
        }
    }
}
