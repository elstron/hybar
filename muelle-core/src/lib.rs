use async_trait::async_trait;
use std::sync::atomic::AtomicBool;

#[async_trait]
pub trait Widget: Send + Sync {
    fn id(&self) -> &'static str;
    fn render(&self) -> String;
    fn needs_thread(&self) -> bool {
        false
    }

    fn start(&mut self) {}
}
pub trait HasPending: Send + Sync {
    fn pending_workspace(&self) -> &AtomicBool;
    fn pending_workspace_urgent(&self) -> &parking_lot::Mutex<Option<String>>;
    fn pending_reload(&self) -> &AtomicBool;
}
