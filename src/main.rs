mod domain;
mod service;

use std::os::unix::net::UnixStream;

use clap::{command, Parser, Subcommand};
use domain::Result;
use serde_json::json;

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,

    #[arg(default_value = "/tmp/mpvsocket")]
    socket_path: String,
}

#[derive(Debug, Subcommand)]
enum CliCommand {
    Status,
    NextChapter,
    PrevChapter,
    VolumeUp { value: i32 },
    VolumeDown { value: i32 },
    TogglePause,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut stream = UnixStream::connect(cli.socket_path)?;

    match cli.command {
        CliCommand::Status => {
            let status = json!({
                "title": service::get_clean_media_title(&mut stream)?,
                "is_paused": service::is_paused(&mut stream)?,
                "current_chapter": service::get_current_chapter(&mut stream).map(|v| v + 1),
                "total_chapters": service::get_total_chapters(&mut stream)
            });
            println!("{}", status)
        }
        CliCommand::NextChapter => service::play_next_chapter(&mut stream)?,
        CliCommand::PrevChapter => service::play_previous_chapter(&mut stream)?,
        CliCommand::VolumeUp { value } => service::increase_volume(&mut stream, value)?,
        CliCommand::VolumeDown { value } => service::decrease_volume(&mut stream, value)?,
        CliCommand::TogglePause => service::toggle_pause(&mut stream)?,
    }

    Ok(())
}
