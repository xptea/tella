use colored::*;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{enable_raw_mode, disable_raw_mode},
    cursor::{Hide, Show},
    execute,
};
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

pub struct MenuSelector {
    options: Vec<(String, String)>,
}

impl MenuSelector {
    pub fn new() -> Self {
        MenuSelector {
            options: Vec::new(),
        }
    }

    pub fn add_option(mut self, title: &str, description: &str) -> Self {
        self.options.push((title.to_string(), description.to_string()));
        self
    }

    pub fn show(&self) -> io::Result<usize> {
        let mut selected = 0;
        let option_count = self.options.len();

        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, Hide)?;

        thread::sleep(Duration::from_millis(200));

        while event::poll(Duration::from_millis(0))? {
            let _ = event::read();
        }

        let result = loop {
            print!("\r");
            for (i, (title, _)) in self.options.iter().enumerate() {
                if i > 0 {
                    print!(" | ");
                }
                if i == selected {
                    print!("{}", format!("[{}]", title).green().bold());
                } else {
                    print!("{}", title.dimmed());
                }
            }
            io::stdout().flush()?;

            if let Ok(true) = event::poll(Duration::from_millis(50)) {
                if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
                    match code {
                        KeyCode::Left | KeyCode::Up => {
                            selected = if selected == 0 { option_count - 1 } else { selected - 1 };
                            thread::sleep(Duration::from_millis(150));
                            while event::poll(Duration::from_millis(0))? {
                                let _ = event::read();
                            }
                        }
                        KeyCode::Right | KeyCode::Down => {
                            selected = (selected + 1) % option_count;
                            thread::sleep(Duration::from_millis(150));
                            while event::poll(Duration::from_millis(0))? {
                                let _ = event::read();
                            }
                        }
                        KeyCode::Enter => {
                            break Ok(selected);
                        }
                        KeyCode::Esc => {
                            break Ok(option_count);
                        }
                        KeyCode::Char('1') if option_count >= 1 => {
                            break Ok(0);
                        }
                        KeyCode::Char('2') if option_count >= 2 => {
                            break Ok(1);
                        }
                        KeyCode::Char('3') if option_count >= 3 => {
                            break Ok(2);
                        }
                        _ => {}
                    }
                }
            }
        };

        execute!(stdout, Show)?;
        disable_raw_mode()?;
        println!();
        result
    }
}

impl Default for MenuSelector {
    fn default() -> Self {
        Self::new()
    }
}

