use crate::api::get_command_suggestion;
use crate::ui::MenuSelector;
use crate::command_executor;
use colored::*;
use std::io;
use std::thread;
use std::time::Duration;

pub async fn handle_ask_command(question: &str) -> io::Result<()> {
    let dot_handle = print_animated_dots();

    let suggestion = match get_command_suggestion(question).await {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("{}", format!("\n❌ Error: {}", e).red());
            return Err(io::Error::new(io::ErrorKind::Other, e));
        }
    };

    drop(dot_handle);
    print!("\r                    \r");
    io::Write::flush(&mut io::stdout())?;

    if suggestion.command == "ERROR" {
        eprintln!("{}", suggestion.description.red());
        eprintln!("{}", suggestion.explanation.yellow());
        return Ok(());
    }

    println!("{}", suggestion.command.bold().yellow());
    
    let severity_display = match suggestion.severity.as_str() {
        "safe" => "🟢 SAFE".green(),
        "warning" => "🟡 WARNING".yellow(),
        "dangerous" => "🔴 DANGEROUS".red(),
        _ => "⚪ UNKNOWN".normal(),
    };
    
    println!("{}", format!("{} - {}", severity_display, suggestion.description).dimmed());

    println!();
    loop {
        let selected = MenuSelector::new()
            .add_option("Run", "")
            .add_option("Explain", "")
            .add_option("Stop", "")
            .show()?;

        match selected {
            0 => {
                match command_executor::execute_command(&suggestion.command).await {
                    Ok(output) => {
                        if !output.trim().is_empty() {
                            println!("\n{}", output);
                        } else {
                            println!("{}", "✅ Done!".green());
                        }
                    }
                    Err(e) => {
                        eprintln!("{}", format!("❌ Error: {}", e).red());
                    }
                }
                break;
            }
            1 => {
                println!("\n{}", suggestion.explanation);
                println!();
            }
            2 => {
                println!("{}", "Goodbye!".yellow());
                break;
            }
            _ => {
                break;
            }
        }
    }

    Ok(())
}

fn print_animated_dots() -> std::sync::Arc<std::sync::atomic::AtomicBool> {
    let stop_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stop_flag_clone = stop_flag.clone();

    std::thread::spawn(move || {
        let mut count = 0;
        while !stop_flag_clone.load(std::sync::atomic::Ordering::Relaxed) {
            print!(".");
            io::Write::flush(&mut io::stdout()).ok();
            thread::sleep(Duration::from_millis(300));
            count += 1;

            if count > 10 {
                print!("\r");
                io::Write::flush(&mut io::stdout()).ok();
                count = 0;
            }
        }
    });

    stop_flag
}

