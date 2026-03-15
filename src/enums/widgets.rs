use std::str::FromStr;

pub enum BarWidget {
    Workspaces,
    Time,
    Separator,
    AppTitle,
    Apps,
    Playback,
    Settings,
    Shutdown,
    Custom(String),
}

impl FromStr for BarWidget {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "workspaces" => Ok(BarWidget::Workspaces),
            "time" | "clock" => Ok(BarWidget::Time),
            "separator" => Ok(BarWidget::Separator),
            "apptitle" | "title" => Ok(BarWidget::AppTitle),
            "apps" => Ok(BarWidget::Apps),
            "playback" | "player" => Ok(BarWidget::Playback),
            "settings" => Ok(BarWidget::Settings),
            "shutdown" => Ok(BarWidget::Shutdown),
            "custom" => Ok(BarWidget::Custom(s.to_string())),
            _ => Err(()),
        }
    }
}
