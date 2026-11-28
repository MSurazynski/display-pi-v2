mod api;
mod epd7in3e;
mod errors;
mod parsers;
mod process;
mod render;
mod storage;
mod task;
mod types;

use crate::task::{dashboard::run_dashboard_task, nasa::run_nasa_task};
use clap::{Parser, Subcommand};
use tracing::info;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Fetch today's Astronomy Picture of the Day and display it
    DisplayApod,
    /// Generate and display a dashboard
    #[command(
        group(
            clap::ArgGroup::new("time_of_day")
                .args(["morning", "evening"])
                .required(true)
                .multiple(false),
        ),
        group(
            clap::ArgGroup::new("day")
                .args(["today", "tomorrow"])
                .required(true)
                .multiple(false),
        ),
    )]
    DisplayDashboard {
        /// Show morning option
        #[arg(long, conflicts_with = "evening")]
        morning: bool,

        /// Show evening option
        #[arg(long, conflicts_with = "morning")]
        evening: bool,

        /// Show today option
        #[arg(long, conflicts_with = "tomorrow")]
        today: bool,

        /// Show tomorrow option
        #[arg(long, conflicts_with = "today")]
        tomorrow: bool,
    },
    /// Show one of my images
    DisplayMyPicture,
    /// List available tasks to execute
    List,
}

fn start_logger() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting");
    start_logger();

    match Cli::parse().command {
        Command::DisplayApod => run_nasa_task().await,
        Command::DisplayDashboard {
            morning,
            evening,
            today,
            tomorrow,
        } => {
            run_dashboard_task().await;
        }
        Command::DisplayMyPicture => {
            todo!()
        }
        Command::List => {
            todo!();
        }
    }

    // make_dashboard().await?;

    info!("Finished");
    Ok(())

    // let raw_weather_data = fetch_wather().await.unwrap();
    // let weather_data = parse_weather_data(raw_weather_data).unwrap();
    // render_dashboard_svg();
    // let todoist_url = "https://api.todoist.com/api/v1/tasks";
    // let api_key = "82fe982a94f66c098447ac908a0939e505c24dea";
    // fetch_tasks_due_today(&client, todoist_url, api_key)
    //     .await
    //     .unwrap();
}
