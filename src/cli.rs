use crate::api::get_command_suggestion;
use crate::ui::MenuSelector;
use crate::command_executor;
use crate::settings::Settings;
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

    // let elapsed = start.elapsed();
    // eprintln!("🔍 Debug: API call took {:?}", elapsed);

    // signal spinner thread to stop
    dot_handle.store(true, std::sync::atomic::Ordering::SeqCst);
    // give spinner a moment to clear the line
    thread::sleep(Duration::from_millis(50));
    print!("\r                    \r");
    io::Write::flush(&mut io::stdout())?;

    if suggestion.command == "ERROR" || suggestion.command == "no command returned" {
        eprintln!("{}", suggestion.description.red());
        eprintln!("{}", suggestion.explanation.yellow());
        return Ok(());
    }

    // Load settings to get output preferences
    let settings = Settings::load().ok();
    let output_settings = settings.as_ref().map(|s| &s.output_settings);

    // Display command if enabled
    if output_settings.map_or(true, |o| o.show_command) {
        println!("{}", suggestion.command.bold().yellow());
    }
    
    // Display severity and description if enabled
    if output_settings.map_or(true, |o| o.show_severity || o.show_description) {
        let severity_display = match suggestion.severity.as_str() {
            "safe" => "🟢 SAFE".green(),
            "warning" => "🟡 WARNING".yellow(),
            "dangerous" => "🔴 DANGEROUS".red(),
            _ => "⚪ UNKNOWN".normal(),
        };
        
        if output_settings.map_or(true, |o| o.show_severity) {
            if output_settings.map_or(true, |o| o.show_description) {
                println!("{}", format!("{} - {}", severity_display, suggestion.description).dimmed());
            } else {
                println!("{}", severity_display);
            }
        } else if output_settings.map_or(true, |o| o.show_description) {
            println!("{}", suggestion.description.dimmed());
        }
    }

    println!();
    loop {
        let mut menu = MenuSelector::new()
            .add_option("Run", "");
        
        // Only add Explain option if explanation is enabled
        let explain_enabled = output_settings.map_or(true, |o| o.show_explanation);
        if explain_enabled {
            menu = menu.add_option("Explain", "");
        }
        
        let menu = menu.add_option("Stop", "");
        let selected = menu.show()?;

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
            1 if explain_enabled => {
                // Show explanation
                println!("\n{}", suggestion.explanation);
                println!();
            }
            _ => {
                println!("{}", "Goodbye!".yellow());
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
        let spinner = ['|', '/', '-', '\\'];
        let mut i = 0;
        while !stop_flag_clone.load(std::sync::atomic::Ordering::Relaxed) {
            print!("\r{} Thinking...", spinner[i]);
            io::Write::flush(&mut io::stdout()).ok();
            thread::sleep(Duration::from_millis(100));
            i = (i + 1) % spinner.len();
        }
        print!("\r                    \r"); // Clear the line
        io::Write::flush(&mut io::stdout()).ok();
    });

    stop_flag
}

