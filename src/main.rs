use clap::{Args, Parser, Subcommand, ValueEnum, error::Result};
use std::path::PathBuf;
use wlmstr::config::Config;
use wlmstr::status::FmtStatus;
use wlmstr::{Derection, Error, PROGRAM_NAME};

///  Stateful wallpaper slideshow CLI for awww
#[derive(Debug, Parser)]
#[command(version, about, name = PROGRAM_NAME)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long = "debug")]
    is_debug_mode: bool,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Next(Next),
    Status(StatusCmd),
    Set(SetCmd),
}

/// next slide by derection [possible derection: seq, pre, rnd] seq is default.
#[derive(Debug, Args)]
struct Next {
    #[arg(default_value_t = Derection::Sequence)]
    derection: Derection,
}

/// show current status(dir and paper path). supports to output in JSON or debug format
#[derive(Debug, Args)]
struct StatusCmd {
    /// Default is Debug
    format: Option<StatusFmt>,
}

#[derive(Debug, Clone, ValueEnum)]
enum StatusFmt {
    Json,
    Debug,
}

/// set config data
#[derive(Debug, Args)]
struct SetCmd {
    #[arg(short = 'd', long = "dir", help = "specify wallpapers dir")]
    dir: Option<PathBuf>,

    #[arg(short = 'p', long = "path", help = "start iamge path of slides")]
    paper_path: Option<PathBuf>,
}

fn run(cli_cmd: Commands, config: Config) -> Result<(), Error> {
    let new_st = match cli_cmd {
        Commands::Next(next_prm) => wlmstr::step(next_prm.derection, config.awww.args)?,
        Commands::Status(sc) => {
            let loaded_status = wlmstr::load_status()?;
            let next_path = {
                let loaded_path_lst = wlmstr::load_paths(loaded_status.get_dir_path())?;
                loaded_status
                    .next(Derection::Sequence, loaded_path_lst)?
                    .get_current_p_path()
            };
            let formatted = FmtStatus::new(&loaded_status, next_path);
            let res = match sc.format.unwrap_or(StatusFmt::Debug) {
                StatusFmt::Json => {
                    serde_json::to_string_pretty(&formatted).map_err(Error::SerializeErr)?
                }
                StatusFmt::Debug => formatted.to_string(),
            };
            println!("{}", res);
            return Ok(());
        }
        Commands::Set(input_set_data) => {
            wlmstr::set(input_set_data.dir, input_set_data.paper_path)?
        }
    };

    println!(
        "change to {}",
        new_st.get_current_p_path().to_string_lossy()
    );
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    let loaded_config = wlmstr::resolve_config().unwrap_or_default();

    if cli.is_debug_mode {
        let config_path = xdg::BaseDirectories::with_prefix("wlmstr")
            .place_config_file("config.toml")
            .unwrap();
        println!("loaded config path: {:?}", config_path);
        println!("config file exists: {:?}", config_path.exists());
        println!("loaded config obj:  {:?}", loaded_config);
    }

    let res = run(cli.command, loaded_config);

    match res {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            eprint!("{}", e);
            std::process::exit(1)
        }
    }
}
