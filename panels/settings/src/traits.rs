pub trait HasSettingsEvent: Send + Sync {
    fn pending_reload(&self);

    fn pending_theme_change(&self, theme: String);
    fn get_default_theme(&self) -> String;

    fn enable_autohide(&self, enable: bool);
    fn get_autohide(&self) -> bool;

    fn set_bar_position(&self, position: String);
    fn get_bar_position(&self) -> String;
}
