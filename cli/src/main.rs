mod app;
mod config;
mod input;
mod theme_bridge;
mod ui;

use std::io::{self, Write};

use anyhow::Result;
use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, FIELD_INDEX_NOTES};

#[derive(Parser)]
#[command(
    name = "sifr",
    about = "Sifr — cross-platform terminal password manager",
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
        /// Suppress newline (quiet output)
        #[arg(long, short)]
        quiet: bool,
    },
    /// Export vault entries to JSON (outputs to stdout or file with -o)
    Export {
        /// Path to the vault file
        path: String,
        /// Output file (stdout if omitted)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Import entries from CSV.
    /// Format: title,username,password,url,notes,totp_secret
    Import {
        /// Path to the vault file
        path: String,
        /// Input CSV file (stdin if omitted)
        #[arg(short, long)]
        input: Option<String>,
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
            quiet,
        }) => {
            let password = sifr_core::crypto::generate_password(
                length,
                !no_uppercase,
                !no_numbers,
                !no_symbols,
            );
            if quiet {
                print!("{}", &*password);
            } else {
                println!("{}", &*password);
            }
        }
        None => {
            // Bare `sifr` → last vault if available, otherwise picker
            let last = config::load_last_vault();
            run_tui(last)?;
        }
        Some(Commands::Export { path, output }) => {
            print!("Master password: ");
            std::io::stdout().flush()?;
            let password = rpassword::read_password()?;
            match sifr_core::Vault::open(&path, &password) {
                Ok(vault) => {
                    match vault.export_json() {
                        Ok(json) => {
                            if let Some(out_path) = output {
                                std::fs::write(&out_path, &json)?;
                                println!("Exported to: {}", out_path);
                            } else {
                                println!("{}", json);
                            }
                        }
                        Err(e) => {
                            eprintln!("Export failed: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to open vault: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Import { path, input }) => {
            print!("Master password: ");
            std::io::stdout().flush()?;
            let password = rpassword::read_password()?;
            match sifr_core::Vault::open(&path, &password) {
                Ok(vault) => {
                    let csv_data = if let Some(input_path) = input {
                        std::fs::read_to_string(&input_path)?
                    } else {
                        use std::io::Read;
                        let mut buf = String::new();
                        std::io::stdin().read_to_string(&mut buf)?;
                        buf
                    };
                    match vault.import_csv(&csv_data) {
                        Ok(count) => {
                            println!("Imported {} entries.", count);
                        }
                        Err(e) => {
                            eprintln!("Import failed: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to open vault: {}", e);
                    std::process::exit(1);
                }
            }
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
        app.record_activity();
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

fn run_loop<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()>
where
    B::Error: Send + Sync + 'static,
{
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

        // Check auto-lock timer (skip if Notes textarea is active)
        if app.form_editing_field != Some(FIELD_INDEX_NOTES) {
            let _ = app.check_auto_lock();
        }

        if !app.running {
            break;
        }
    }
    Ok(())
}
