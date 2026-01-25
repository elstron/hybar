use crate::DEBOUNCE_MS;
use crate::EventState;
use crate::HYPRLAND_SUBSCRIPTION;
use crate::get_hypr_socket_path;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

pub async fn hyprland_event_listener(event_state: Arc<EventState>) {
    let path = match get_hypr_socket_path() {
        Some(p) => p,
        None => {
            eprintln!("Could not determine Hyprland socket path");
            return;
        }
    };

    loop {
        match UnixStream::connect(&path).await {
            Ok(mut stream) => {
                println!("Connected to Hyprland socket");

                if let Err(e) = stream.write_all(HYPRLAND_SUBSCRIPTION.as_bytes()).await {
                    eprintln!("Failed to subscribe to Hyprland events: {}", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }

                let reader = BufReader::new(stream);
                let mut lines = reader.lines();

                let mut debounce = tokio::time::interval(Duration::from_millis(DEBOUNCE_MS));
                debounce.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

                let mut has_workspace_update = false;
                let mut has_fullscreen_update = false;
                let mut latest_title: Option<String> = None;
                let mut has_workspace_urgent: Option<String> = None;
                loop {
                    tokio::select! {
                        line_result = lines.next_line() => {
                            match line_result {
                                Ok(Some(line)) => {
                                    if line.contains("\"change\":") || line.contains("workspace") {
                                        has_workspace_update = true;
                                    } else if line.contains("fullscreen") && line.contains("1") {
                                        has_fullscreen_update = true;
                                    } else if line.contains("activewindow>>")
                                        && let Some(title_start) = line.find(">>") {
                                        latest_title = Some(line[title_start + 2..].trim().to_string());
                                    } else if line.contains("urgent>>")
                                        && let Some(start) = line.find("urgent>>") {
                                        let urgent_id = line[start + 9..].trim().to_string();
                                        has_workspace_urgent = Some(urgent_id);
                                        has_workspace_update = true;
                                    }
                                }
                                Ok(None) => {
                                    eprintln!("Hyprland socket closed");
                                    break;
                                }
                                Err(e) => {
                                    eprintln!("Error reading from socket: {}", e);
                                    break;
                                }
                            }
                        }

                        _ = debounce.tick() => {
                            let mut needs_gtk_update = false;

                            if has_workspace_update {
                                event_state.pending_workspace.store(true, Ordering::Relaxed);
                                has_workspace_update = false;
                                needs_gtk_update = true;
                            }

                            if has_fullscreen_update {
                                event_state.pending_fullscreen.store(true, Ordering::Relaxed);
                                has_fullscreen_update = false;
                                needs_gtk_update = true;
                            }

                            if let Some(title) = latest_title.take() {
                                *event_state.pending_title.lock() = Some(title);
                                needs_gtk_update = true;
                            }

                            if let Some(urgent_id) = has_workspace_urgent.take() {
                                *event_state.pending_workspace_urgent.lock() = Some(urgent_id);
                                has_workspace_update = false;
                                needs_gtk_update = true;
                            }

                            if needs_gtk_update {
                                glib::idle_add_once(|| {});
                            }
                        }
                    }
                }

                eprintln!("Connection to Hyprland socket lost, reconnecting...");
            }
            Err(e) => {
                eprintln!("Failed to connect to Hyprland socket: {}", e);
            }
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
