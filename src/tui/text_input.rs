use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::theme;

pub enum Action {
    Submit(String),
    Cancel,
    Continue,
}

pub struct TextInput {
    pub value: String,
    pub cursor: usize,
    pub default: Option<String>,
    pub label: String,
}

impl TextInput {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            value: String::new(),
            cursor: 0,
            default: None,
            label: label.into(),
        }
    }

    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }

    pub fn with_initial(mut self, initial: impl Into<String>) -> Self {
        let val: String = initial.into();
        self.cursor = val.len();
        self.value = val;
        self
    }

    pub fn result(&self) -> String {
        if self.value.is_empty() {
            self.default.clone().unwrap_or_default()
        } else {
            self.value.clone()
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.value.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    pub fn delete_char(&mut self) {
        if self.cursor > 0 {
            let prev = self.value[..self.cursor]
                .chars()
                .last()
                .map(|c| c.len_utf8())
                .unwrap_or(0);
            self.cursor -= prev;
            self.value.remove(self.cursor);
        }
    }

    pub fn delete_forward(&mut self) {
        if self.cursor < self.value.len() {
            self.value.remove(self.cursor);
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            let prev = self.value[..self.cursor]
                .chars()
                .last()
                .map(|c| c.len_utf8())
                .unwrap_or(0);
            self.cursor -= prev;
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.value.len() {
            let next = self.value[self.cursor..]
                .chars()
                .next()
                .map(|c| c.len_utf8())
                .unwrap_or(0);
            self.cursor += next;
        }
    }

    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor = self.value.len();
    }

    pub fn handle_event(&mut self, event: &Event) -> Action {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            if *modifiers == KeyModifiers::CONTROL && *code == KeyCode::Char('c') {
                std::process::exit(130);
            }
            match code {
                KeyCode::Enter => return Action::Submit(self.result()),
                KeyCode::Esc => return Action::Cancel,
                KeyCode::Char(c) => self.insert_char(*c),
                KeyCode::Backspace => self.delete_char(),
                KeyCode::Delete => self.delete_forward(),
                KeyCode::Left => self.move_left(),
                KeyCode::Right => self.move_right(),
                KeyCode::Home => self.move_home(),
                KeyCode::End => self.move_end(),
                _ => {}
            }
        }
        Action::Continue
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let mut y = area.y;

        // Label
        let label_line = Line::from(vec![
            Span::styled("  ? ", theme::CYAN),
            Span::styled(&self.label, theme::CYAN),
        ]);
        frame.render_widget(
            Paragraph::new(label_line),
            Rect::new(area.x, y, area.width, 1),
        );
        y += 1;

        // Input line
        let display = if self.value.is_empty() {
            if let Some(ref def) = self.default {
                format!("  > {def}")
            } else {
                "  > ".to_string()
            }
        } else {
            format!("  > {}", self.value)
        };

        let style = if self.value.is_empty() && self.default.is_some() {
            theme::DIM
        } else {
            theme::NORMAL
        };
        let input_line = Line::from(Span::styled(&display, style));
        frame.render_widget(
            Paragraph::new(input_line),
            Rect::new(area.x, y, area.width, 1),
        );

        // Cursor position: "  > " is 4 chars, then cursor offset into value
        let cursor_col = 4 + self.value[..self.cursor].chars().count() as u16;
        frame.set_cursor_position((area.x + cursor_col, y));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_empty_input() {
        let input = TextInput::new("test");
        assert_eq!(input.value, "");
        assert_eq!(input.cursor, 0);
        assert_eq!(input.label, "test");
    }

    #[test]
    fn insert_and_cursor() {
        let mut input = TextInput::new("test");
        input.insert_char('a');
        input.insert_char('b');
        assert_eq!(input.value, "ab");
        assert_eq!(input.cursor, 2);
    }

    #[test]
    fn delete_char_removes_before_cursor() {
        let mut input = TextInput::new("test").with_initial("abc");
        input.delete_char();
        assert_eq!(input.value, "ab");
        assert_eq!(input.cursor, 2);
    }

    #[test]
    fn move_left_right() {
        let mut input = TextInput::new("test").with_initial("abc");
        input.move_left();
        assert_eq!(input.cursor, 2);
        input.move_right();
        assert_eq!(input.cursor, 3);
    }

    #[test]
    fn home_end() {
        let mut input = TextInput::new("test").with_initial("abc");
        input.move_home();
        assert_eq!(input.cursor, 0);
        input.move_end();
        assert_eq!(input.cursor, 3);
    }

    #[test]
    fn result_returns_default_when_empty() {
        let input = TextInput::new("test").with_default("fallback");
        assert_eq!(input.result(), "fallback");
    }

    #[test]
    fn result_returns_value_when_not_empty() {
        let input = TextInput::new("test")
            .with_default("fallback")
            .with_initial("typed");
        assert_eq!(input.result(), "typed");
    }

    #[test]
    fn delete_at_start_is_noop() {
        let mut input = TextInput::new("test");
        input.delete_char();
        assert_eq!(input.value, "");
        assert_eq!(input.cursor, 0);
    }

    #[test]
    fn move_left_at_start_is_noop() {
        let mut input = TextInput::new("test");
        input.move_left();
        assert_eq!(input.cursor, 0);
    }

    #[test]
    fn move_right_at_end_is_noop() {
        let mut input = TextInput::new("test").with_initial("ab");
        input.move_right();
        assert_eq!(input.cursor, 2);
    }

    #[test]
    fn renders_label() {
        use crate::tui::test_helpers::{buffer_text, test_terminal};
        use ratatui::layout::Rect;

        let mut terminal = test_terminal(60, 5);
        let input = TextInput::new("Repository URL");
        terminal
            .draw(|frame| {
                input.render(frame, Rect::new(0, 0, 60, 5));
            })
            .unwrap();
        let text = buffer_text(&terminal);
        assert!(text.contains("Repository URL"), "expected label in: {text}");
    }

    #[test]
    fn renders_value() {
        use crate::tui::test_helpers::{buffer_text, test_terminal};
        use ratatui::layout::Rect;

        let mut terminal = test_terminal(60, 5);
        let input = TextInput::new("test").with_initial("hello world");
        terminal
            .draw(|frame| {
                input.render(frame, Rect::new(0, 0, 60, 5));
            })
            .unwrap();
        let text = buffer_text(&terminal);
        assert!(text.contains("> hello world"), "expected value in: {text}");
    }

    #[test]
    fn renders_default_when_empty() {
        use crate::tui::test_helpers::{buffer_text, test_terminal};
        use ratatui::layout::Rect;

        let mut terminal = test_terminal(60, 5);
        let input = TextInput::new("test").with_default("main");
        terminal
            .draw(|frame| {
                input.render(frame, Rect::new(0, 0, 60, 5));
            })
            .unwrap();
        let text = buffer_text(&terminal);
        assert!(text.contains("> main"), "expected default in: {text}");
    }
}
