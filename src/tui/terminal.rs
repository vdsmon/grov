use std::io::{self, Stdout};
use std::panic;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::prelude::CrosstermBackend;

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

fn init_terminal() -> io::Result<Tui> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn restore_terminal(terminal: &mut Tui) {
    let _ = disable_raw_mode();
    let _ = execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    );
    let _ = terminal.show_cursor();
}

/// Run a closure with a fully initialized TUI terminal.
///
/// Enters raw mode + alternate screen, calls the closure, then restores
/// the terminal on return (success or error). Also installs a panic hook
/// that restores the terminal before unwinding.
pub fn run_tui<T>(f: impl FnOnce(&mut Tui) -> anyhow::Result<T>) -> anyhow::Result<T> {
    let mut terminal = init_terminal()?;

    // Install panic hook that restores the terminal
    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        prev_hook(info);
    }));

    let result = f(&mut terminal);

    restore_terminal(&mut terminal);

    // Restore the default panic hook
    let _ = panic::take_hook();

    result
}
