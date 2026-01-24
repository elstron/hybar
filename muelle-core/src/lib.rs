use async_trait::async_trait;

#[async_trait]
pub trait Widget: Send + Sync {
    fn id(&self) -> &'static str;
    fn render(&self) -> String;
    fn needs_thread(&self) -> bool {
        false
    }

    fn start(&mut self) {}
}
