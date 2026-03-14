use ratatui::style::{Color, Modifier, Style};

pub const CYAN: Style = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);
pub const GREEN: Style = Style::new().fg(Color::Green).add_modifier(Modifier::BOLD);
pub const DIM: Style = Style::new().add_modifier(Modifier::DIM);
pub const HIGHLIGHT: Style = Style::new().fg(Color::Cyan);
pub const NORMAL: Style = Style::new();
pub const ERROR: Style = Style::new().fg(Color::Red);

pub const HELP_WIZARD: &str = "Enter confirm  ·  Esc back  ·  Ctrl+C cancel";
pub const HELP_SELECT: &str = "↑↓ navigate  ·  type to filter  ·  Enter select  ·  Esc cancel";
pub const HELP_CONFIRM: &str = "←→ toggle  ·  y/n  ·  Enter confirm  ·  Esc back";
