mod cli;
mod api;
mod ui;
mod command_executor;
mod settings;
mod updater;

use clap::Parser;
use colored::*;
use std::io;

#[derive(Parser, Debug)]
#[command(name = "tella")]
#[command(about = "Ask about commands - get the best command for your task", long_about = None)]
struct Args {
    #[arg(long, action)]
    settings: bool,

    #[arg(long, action)]
    upgrade: bool,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    question: Vec<String>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    dotenv::dotenv().ok();

    let args = Args::parse();

    if args.upgrade {
        match updater::perform_upgrade().await {
            Ok(_) => return Ok(()),
            Err(e) => {
                eprintln!("{}", format!("❌ Error: {}", e).red());
                return Err(io::Error::new(io::ErrorKind::Other, e));
            }
        }
    }

    if args.settings {
        match settings::Settings::interactive_setup() {
            Ok(_) => return Ok(()),
            Err(e) => {
                eprintln!("{}", format!("❌ Error: {}", e).red());
                return Err(io::Error::new(io::ErrorKind::Other, e));
            }
        }
    }

    tokio::spawn(async {
        updater::check_for_updates().await;
    });

    if !args.question.is_empty() {
        let question = args.question.join(" ");
        cli::handle_ask_command(&question).await?;
    } else {
        println!("{}", "tella - Command Assistant v0.1.19".bold().cyan());
        println!("{}", "━".repeat(50));
        println!("\n{}", "Usage:".bold());
        println!("  {} tella your question here", "$".cyan());
        println!("  {} tella show me the last 5 git commits", "$".cyan());
        println!("  {} tella --settings", "$".cyan());
        println!("  {} tella --upgrade", "$".cyan());
        println!("\n{}", "Examples:".bold());
        println!("  {} tella how to list files in directory", "$".cyan());
        println!("  {} tella find large files on my system", "$".cyan());
        println!("  {} tella create a backup of my files", "$".cyan());
    }

    Ok(())
}
