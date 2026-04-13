mod app;
mod config;
mod input;
mod theme_bridge;
mod ui;

use std::io;

use anyhow::Result;
use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;

#[derive(Parser)]
#[command(
    name = "sifr",
    about = "Sifr — beautiful TUI password manager",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new vault at the given path
    New {
        /// Path for the new vault file
        path: String,
    },
    /// Open an existing vault and enter the TUI (omit path to open vault picker)
    Open {
        /// Path to the vault file (optional; opens file picker if omitted)
        path: Option<String>,
    },
    /// Generate a random password
    Gen {
        /// Password length
        #[arg(long, default_value_t = 16)]
        length: usize,
        /// Exclude symbols
        #[arg(long)]
        no_symbols: bool,
        /// Exclude numbers
        #[arg(long)]
        no_numbers: bool,
        /// Exclude uppercase letters
        #[arg(long)]
        no_uppercase: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::New { path }) => {
            use std::io::Write;
            print!("Master password: ");
            std::io::stdout().flush()?;
            let password = rpassword::read_password()?;
            print!("Confirm password: ");
            std::io::stdout().flush()?;
            let confirm = rpassword::read_password()?;
            if password != confirm {
                eprintln!("Passwords do not match.");
                std::process::exit(1);
            }
            if password.is_empty() {
                eprintln!("Password cannot be empty.");
                std::process::exit(1);
            }
            match sifr_core::Vault::create(&path, &password) {
                Ok(vault) => {
                    println!("Vault created at: {path}");
                    config::save_last_vault(&path);
                    // Launch TUI with vault already unlocked
                    run_tui_inner(Some(path), Some(vault))?;
                }
                Err(e) => {
                    eprintln!("Failed to create vault: {e}");
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Open { path }) => {
            run_tui(path)?;
        }
        Some(Commands::Gen {
            length,
            no_symbols,
            no_numbers,
            no_uppercase,
        }) => {
            let password = sifr_core::crypto::generate_password(
                length,
                !no_uppercase,
                !no_numbers,
                !no_symbols,
            );
            println!("{}", &*password);
        }
        None => {
            // Bare `sifr` → last vault if available, otherwise picker
            let last = config::load_last_vault();
            run_tui(last)?;
        }
    }

    Ok(())
}

fn restore_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
}

fn run_tui(vault_path: Option<String>) -> Result<()> {
    run_tui_inner(vault_path, None)
}

fn run_tui_inner(vault_path: Option<String>, vault: Option<sifr_core::Vault>) -> Result<()> {
    // Install panic hook so terminal is always restored, even on panic
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        original_hook(info);
    }));

    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let path = vault_path.unwrap_or_default();
    let started_from_picker = path.is_empty();
    let mut app = App::new(path.clone());
    app.started_from_picker = started_from_picker;

    if let Some(v) = vault {
        // Pre-opened vault (e.g. from `sifr new`)
        app.vault = Some(v);
        app.refresh_entries();
        app.screen = app::Screen::EntryList;
    } else if path.is_empty() {
        app.screen = app::Screen::VaultPicker;
    }

    let result = run_loop(&mut terminal, &mut app);

    // Clear clipboard if timer is still active
    if app.clipboard_clear_at.is_some() {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(String::new());
        }
    }

    // Zeroize password input before drop
    *app.password_input = String::new();
    *app.password_confirm = String::new();

    // Restore terminal
    restore_terminal();
    terminal.show_cursor()?;

    result
}

fn run_loop<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if crossterm::event::poll(std::time::Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(key) => input::handle_key(app, key),
                Event::Mouse(mouse) => input::handle_mouse(app, mouse),
                _ => {}
            }
        }

        // Check clipboard auto-clear timer
        if let Some(clear_at) = app.clipboard_clear_at {
            if std::time::Instant::now() >= clear_at {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(String::new());
                }
                app.clipboard_clear_at = None;
                app.clipboard_notification = None;
            }
        }

        // Check error auto-clear timer
        if let Some(clear_at) = app.error_clear_at {
            if std::time::Instant::now() >= clear_at {
                app.error_message = None;
                app.error_clear_at = None;
            }
        }

        if !app.running {
            break;
        }
    }
    Ok(())
}
