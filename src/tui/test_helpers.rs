use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

pub fn key_event(code: KeyCode) -> Event {
    Event::Key(KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

pub fn key_char(c: char) -> Event {
    key_event(KeyCode::Char(c))
}

pub fn type_string(s: &str) -> Vec<Event> {
    s.chars().map(key_char).collect()
}

pub fn enter() -> Event {
    key_event(KeyCode::Enter)
}

pub fn esc() -> Event {
    key_event(KeyCode::Esc)
}

pub fn test_terminal(w: u16, h: u16) -> Terminal<TestBackend> {
    Terminal::new(TestBackend::new(w, h)).expect("failed to create test terminal")
}

/// Join all cells in the backend buffer into a single string (rows joined by newlines).
pub fn buffer_text(terminal: &Terminal<TestBackend>) -> String {
    let buf = terminal.backend().buffer();
    let area = *buf.area();
    let mut lines = Vec::new();
    for y in area.y..area.y + area.height {
        let mut line = String::new();
        for x in area.x..area.x + area.width {
            let cell = &buf[(x, y)];
            line.push_str(cell.symbol());
        }
        lines.push(line);
    }
    lines.join("\n")
}
