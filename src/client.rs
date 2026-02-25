use crate::DEBOUNCE_MS;
use crate::EventState;
use crate::HYPRLAND_SUBSCRIPTION;
use crate::UiEvent;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

struct States {
    has_workspace_update: bool,
    is_fullscreen: bool,
    has_fullscreen_update: bool,
    latest_title: Option<String>,
    has_workspace_urgent: Option<String>,
    has_open_window: Option<(String, String)>,
    has_close_window: Option<String>,
}

pub struct HyprlandClient {
    event_state: Arc<EventState>,
    sender: async_channel::Sender<UiEvent>,
    states: States,
}

impl HyprlandClient {
    pub fn new(event_state: Arc<EventState>, sender: async_channel::Sender<UiEvent>) -> Self {
        Self {
            event_state,
            sender,
            states: States {
                has_workspace_update: false,
                is_fullscreen: false,
                has_fullscreen_update: false,
                latest_title: None,
                has_workspace_urgent: None,
                has_open_window: None,
                has_close_window: None,
            },
        }
    }

    pub async fn run(&mut self) {
        let path = match self.socket_path() {
            Some(p) => p,
            None => {
                eprintln!("Could not determine Hyprland socket path");
                return;
            }
        };

        loop {
            match UnixStream::connect(&path).await {
                Ok(stream) => {
                    if let Err(e) = self.connect(stream).await {
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

    async fn connect(&mut self, mut stream: UnixStream) -> std::io::Result<UnixStream> {
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

        loop {
            tokio::select! {
                line_result = lines.next_line() => {
                    match line_result {
                        Ok(Some(line)) => self.event_matches(&line),
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
                _ = debounce.tick() => self.debounce().await,
            }
        }
    }

    fn event_matches(&mut self, line: &str) {
        let mut line = line.split(">>");
        let event_name = line.next().unwrap_or("").trim();
        let event_value = line.next().unwrap_or("").trim();

        match event_name {
            "change" | "workspace" => {
                self.states.has_workspace_update = true;
            }
            "fullscreen" => {
                self.states.has_fullscreen_update = true;
                self.states.is_fullscreen = event_value.contains("1");
            }
            "activewindow" => {
                let mut title = event_value.trim().to_string();

                if (title == ",") || (title == " - ") || (title.is_empty()) {
                    title = "".to_string();
                }

                self.states.latest_title = Some(title);
            }
            "urgent" => {
                let urgent_id = event_value.trim().to_string();
                self.states.has_workspace_urgent = Some(urgent_id);
                self.states.has_workspace_update = true;
            }
            "openwindow" => {
                let data = event_value;
                let mut parts = data.split(",");
                let app_id = parts.next().unwrap_or("").to_string();
                let _ = parts.next();
                let window_name = parts.next().unwrap_or("").trim().to_string();
                self.states.has_open_window = Some((window_name.to_lowercase(), app_id));
            }
            "closewindow" => {
                let app_id = event_value.to_string();
                self.states.has_close_window = Some(app_id);
            }
            _ => {}
        }
    }

    fn socket_path(&self) -> Option<PathBuf> {
        let runtime_dir = env::var("XDG_RUNTIME_DIR").ok()?;
        let instance = env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;

        let base = PathBuf::from(runtime_dir).join("hypr").join(instance);

        let candidates = [".socket2.sock", "hyprland.sock2"];

        for name in candidates {
            let path = base.join(name);
            if path.exists() {
                return Some(path);
            }
        }

        None
    }

    async fn debounce(&mut self) {
        let mut needs_gtk_update = false;

        if self.states.has_workspace_update {
            self.sender.send(UiEvent::WorkspaceChanged).await.ok();
            self.states.has_workspace_update = false;
            needs_gtk_update = true;
        }

        if self.states.has_fullscreen_update {
            self.sender
                .send(UiEvent::FullscreenChanged(self.states.is_fullscreen))
                .await
                .ok();
            self.states.has_fullscreen_update = false;
            needs_gtk_update = true;
        }

        if let Some(title) = self.states.latest_title.take() {
            self.sender
                .send(UiEvent::TitleChanged(title.clone()))
                .await
                .ok();
            *self.event_state.pending_title.lock() = Some(title);
            needs_gtk_update = true;
        }

        if let Some(urgent_id) = self.states.has_workspace_urgent.take() {
            self.sender
                .send(UiEvent::WorkspaceUrgent(urgent_id.clone()))
                .await
                .ok();
            self.states.has_workspace_update = false;
            needs_gtk_update = true;
        }

        if let Some((name, id)) = self.states.has_open_window.take() {
            self.sender
                .send(UiEvent::WindowOpened((name, id)))
                .await
                .ok();
            needs_gtk_update = true;
        }

        if let Some(id) = self.states.has_close_window.take() {
            self.sender.send(UiEvent::WindowClosed(id)).await.ok();
            needs_gtk_update = true;
        }

        if needs_gtk_update {
            glib::idle_add_once(|| {});
        }
    }
}
