mod app;
mod input;
mod theme_bridge;
mod ui;

use std::io;

use anyhow::Result;
use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, Event},
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
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new vault at the given path
    New {
        /// Path for the new vault file
        path: String,
    },
    /// Open an existing vault and enter the TUI
    Open {
        /// Path to the vault file
        path: String,
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
        Commands::New { path } => {
            println!("Creating new vault at: {path}");
            println!("(vault creation will be implemented in a future release)");
        }
        Commands::Open { path } => {
            run_tui(path)?;
        }
        Commands::Gen {
            length,
            no_symbols,
            no_numbers,
            no_uppercase,
        } => {
            let password = sifr_core::crypto::generate_password(
                length,
                !no_uppercase,
                !no_numbers,
                !no_symbols,
            );
            println!("{}", &*password);
        }
    }

    Ok(())
}

fn run_tui(vault_path: String) -> Result<()> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(vault_path);

    let result = run_loop(&mut terminal, &mut app);

    // Restore terminal regardless of result
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;
        if let Event::Key(key) = event::read()? {
            input::handle_key(app, key);
        }
        if !app.running {
            break;
        }
    }
    Ok(())
}
