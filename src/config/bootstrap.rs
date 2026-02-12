use directories::ProjectDirs;
use include_dir::{Dir, include_dir};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

static DEFAULTS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/defaults");

pub fn bootstrap_config() -> Result<PathBuf, Box<dyn Error>> {
    let proj_dirs = ProjectDirs::from("com", "stron", "hybar")
        .ok_or("The config directory could not be determined.")?;

    let config_dir = proj_dirs.config_dir();

    fs::create_dir_all(config_dir)?;

    extract_if_missing(&DEFAULTS_DIR, config_dir)?;

    Ok(config_dir.to_path_buf())
}

fn extract_if_missing(dir: &Dir, target: &Path) -> Result<(), Box<dyn Error>> {
    for file in dir.files() {
        let relative_path = file.path();
        let destination = target.join(relative_path);

        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }

        if !destination.exists() {
            fs::write(destination, file.contents())?;
        }
    }

    for subdir in dir.dirs() {
        extract_if_missing(subdir, target)?;
    }

    Ok(())
}
