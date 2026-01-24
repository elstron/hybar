use chrono::Local;
use muelle_core::Widget;

pub struct Clock;

impl Widget for Clock {
    fn id(&self) -> &'static str {
        "clock"
    }

    fn render(&self) -> String {
        format!("{}", Local::now().format("%H:%M:%S"))
    }

    fn needs_thread(&self) -> bool {
        true
    }

    fn start(&mut self) {
        // Lanza un hilo
        std::thread::spawn(|| {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(0));
            }
        });
    }
}
