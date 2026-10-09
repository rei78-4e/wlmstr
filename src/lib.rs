use crate::config::Config;
use crate::status::Status;
use clap::ValueEnum;
use easy_storage::Storeable;
use std::fmt::Display;
use std::io;
use std::path::PathBuf;

pub const PROGRAM_NAME: &str = "wlmstr";

pub mod config;
pub mod status;

#[derive(Debug)]
pub enum Error {
    FailedProcess(bool, String, String),
    Io(std::io::Error),
    NotFoundXDGDATAPATH,
    ValueNotSet,
    InsufficientValues,
    SerializeErr(serde_json::Error),
    FileIo(easy_storage::Error),
    NoWallpaperFound,
    NotSupportFile(String),
    ProcessFindErr(procfs::ProcError),
    StoreErr(easy_storage::Error),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<easy_storage::Error> for Error {
    fn from(value: easy_storage::Error) -> Self {
        Self::StoreErr(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO Error: {}", e),
            Error::NotFoundXDGDATAPATH => write!(
                f,
                "XDG_DATA_HOME is not set.\n\nThis program uses XDG_DATA_HOME/{}/ to store its data.\nIf you are not using Linux or a standard XDG-compatible environment,please set the XDG_DATA_HOME environment variable manually.Not dound XDG_DATA_HOME.\nThis Program use XDG_DATA_HOME/{}/ to keep path data.\nIf you don't use Linux or standard envrinment",
                PROGRAM_NAME, PROGRAM_NAME
            ),
            Error::ValueNotSet => write!(
                f,
                "The wallpaper directory path has not been set.\nPlease run `set` subcommand\nwlmstr set -d <dir-path> -p <start-img-path>)"
            ),
            Error::InsufficientValues => write!(
                f,
                "Insufficient values; values for dir and start img path are all required for initialization."
            ),
            Error::SerializeErr(e) => write!(f, "Serialize Error: {}", e),
            Error::FailedProcess(is_awww, v, p) => write!(
                f,
                "failed to process {}: {}\n{}",
                if *is_awww { "awww" } else { "mpbpaper" },
                v,
                p
            ),
            Error::FileIo(e) => write!(f, "File IO Error: {}", e),
            Error::NoWallpaperFound => write!(f, "no wallpaper found in the directory"),
            Error::NotSupportFile(p) => write!(f, "not supports file type: {}", p),
            Error::ProcessFindErr(proc_error) => write!(f, "{}", proc_error),
            Error::StoreErr(error) => write!(f, "{}", error),
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Derection {
    #[value(name = "seq")]
    Sequence,

    #[value(name = "pre")]
    Previous,

    #[value(name = "rnd")]
    Random,
}

impl Display for Derection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Derection::Sequence => "seq",
                Derection::Previous => "pre",
                Derection::Random => "rnd",
            }
        )
    }
}

pub fn resolve_data_path() -> Result<PathBuf, Error> {
    std::env::var_os("XDG_DATA_HOME")
        .map(|f| {
            PathBuf::from(f.to_string_lossy().to_string())
                .join("wlmstr")
                .join("data.json")
        })
        .ok_or(Error::NotFoundXDGDATAPATH)
}

pub fn resolve_config() -> Result<Config, Error> {
    let config_path =
        xdg::BaseDirectories::with_prefix("wlmstr").place_config_file("config.toml")?;

    Ok(Config::load_by_extension(config_path)?)
}

pub fn load_paths(dir: PathBuf) -> Result<Vec<PathBuf>, Error> {
    Ok(std::fs::read_dir(dir)
        .map_err(Error::Io)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect())
}

/// load saved status. returns `Error::ValueNotSet` when nothing has been saved yet.
pub fn load_status() -> Result<Status, Error> {
    let data_path = resolve_data_path()?;
    Status::load_by_extension(&data_path).map_err(|_| Error::ValueNotSet)
}

pub fn save_status(status: &Status) -> Result<(), Error> {
    let data_path = resolve_data_path()?;
    status
        .save_by_extension(data_path, true)
        .map_err(Error::FileIo)
}

/// move to the next wallpaper by `derection`, apply it and save the new status.
pub fn step(derection: Derection, awww_args: Vec<String>) -> Result<Status, Error> {
    let loaded_status = load_status()?;
    let loaded_path_lst = load_paths(loaded_status.get_dir_path())?;
    let new_status = loaded_status.next(derection, loaded_path_lst)?;
    new_status.apply(awww_args)?;
    save_status(&new_status)?;
    Ok(new_status)
}

/// update (or initialize) the wallpaper dir / start path and save the new status.
pub fn set(dir: Option<PathBuf>, paper_path: Option<PathBuf>) -> Result<Status, Error> {
    let new_status = match load_status() {
        Ok(loaded_status) => loaded_status.set(dir, paper_path)?,
        // when not found XDG_DATA_HOME/wlmstr/data.json
        Err(_) => match (dir, paper_path) {
            (Some(dir), Some(p_path)) => Status::new(dir, p_path),
            (Some(dir), None) => {
                let d = Status::get_first_paper_in_dir(&dir)?;
                Status::new(dir, d)
            }
            _ => return Err(Error::InsufficientValues),
        },
    };
    save_status(&new_status)?;
    Ok(new_status)
}
