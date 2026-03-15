use std::str::FromStr;

#[derive(Debug)]
pub enum BarPosition {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug)]
pub enum BarLayout {
    FullWidth,
    Centered,
    Floating,
}

impl FromStr for BarPosition {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "top" => Ok(BarPosition::Top),
            "bottom" => Ok(BarPosition::Bottom),
            "right" => Ok(BarPosition::Right),
            "left" => Ok(BarPosition::Left),
            _ => Err(()),
        }
    }
}

impl FromStr for BarLayout {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "fullwidth" | "full" => Ok(BarLayout::FullWidth),
            "centered" => Ok(BarLayout::Centered),
            "floating" => Ok(BarLayout::Floating),
            _ => Err(()),
        }
    }
}
