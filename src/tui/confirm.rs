use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::theme;

pub enum Action {
    Confirmed(bool),
    Cancel,
    Continue,
}

pub struct Confirm {
    pub label: String,
    pub selected: bool, // false=No (default), true=Yes
}

impl Confirm {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            selected: false,
        }
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
                KeyCode::Enter => return Action::Confirmed(self.selected),
                KeyCode::Esc => return Action::Cancel,
                KeyCode::Char('y' | 'Y') => return Action::Confirmed(true),
                KeyCode::Char('n' | 'N') => return Action::Confirmed(false),
                KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                    self.selected = !self.selected;
                }
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
        y += 2;

        // Buttons
        let (no_style, yes_style) = if self.selected {
            (theme::DIM, theme::HIGHLIGHT)
        } else {
            (theme::HIGHLIGHT, theme::DIM)
        };

        let no_text = if self.selected { "  No  " } else { " [No]  " };
        let yes_text = if self.selected { " [Yes] " } else { "  Yes  " };

        let buttons = Line::from(vec![
            Span::raw("      "),
            Span::styled(no_text, no_style),
            Span::raw("   "),
            Span::styled(yes_text, yes_style),
        ]);
        frame.render_widget(Paragraph::new(buttons), Rect::new(area.x, y, area.width, 1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::test_helpers::key_event;

    #[test]
    fn default_is_no() {
        let confirm = Confirm::new("test?");
        assert!(!confirm.selected);
    }

    #[test]
    fn toggle_with_arrow() {
        let mut confirm = Confirm::new("test?");
        confirm.handle_event(&key_event(KeyCode::Right));
        assert!(confirm.selected);
        confirm.handle_event(&key_event(KeyCode::Left));
        assert!(!confirm.selected);
    }

    #[test]
    fn y_confirms_true() {
        let mut confirm = Confirm::new("test?");
        let action = confirm.handle_event(&key_event(KeyCode::Char('y')));
        assert!(matches!(action, Action::Confirmed(true)));
    }

    #[test]
    fn n_confirms_false() {
        let mut confirm = Confirm::new("test?");
        let action = confirm.handle_event(&key_event(KeyCode::Char('n')));
        assert!(matches!(action, Action::Confirmed(false)));
    }

    #[test]
    fn enter_confirms_current() {
        let mut confirm = Confirm::new("test?");
        // Default is No
        let action = confirm.handle_event(&key_event(KeyCode::Enter));
        assert!(matches!(action, Action::Confirmed(false)));

        // Toggle to Yes, then Enter
        confirm.handle_event(&key_event(KeyCode::Right));
        let action = confirm.handle_event(&key_event(KeyCode::Enter));
        assert!(matches!(action, Action::Confirmed(true)));
    }

    #[test]
    fn esc_cancels() {
        let mut confirm = Confirm::new("test?");
        let action = confirm.handle_event(&key_event(KeyCode::Esc));
        assert!(matches!(action, Action::Cancel));
    }

    #[test]
    fn renders_label_and_buttons() {
        use crate::tui::test_helpers::{buffer_text, test_terminal};
        use ratatui::layout::Rect;

        let mut terminal = test_terminal(60, 10);
        let confirm = Confirm::new("Proceed with init?");
        terminal
            .draw(|frame| {
                confirm.render(frame, Rect::new(0, 0, 60, 10));
            })
            .unwrap();
        let text = buffer_text(&terminal);
        assert!(
            text.contains("Proceed with init?"),
            "expected label in: {text}"
        );
        assert!(text.contains("No"), "expected 'No' in: {text}");
        assert!(text.contains("Yes"), "expected 'Yes' in: {text}");
    }

    #[test]
    fn renders_selected_button() {
        use crate::tui::test_helpers::{buffer_text, test_terminal};
        use ratatui::layout::Rect;

        let mut terminal = test_terminal(60, 10);
        let mut confirm = Confirm::new("test?");
        // Default: No is selected → [No]
        terminal
            .draw(|frame| {
                confirm.render(frame, Rect::new(0, 0, 60, 10));
            })
            .unwrap();
        let text = buffer_text(&terminal);
        assert!(text.contains("[No]"), "expected '[No]' in: {text}");

        // Toggle to Yes → [Yes]
        confirm.handle_event(&key_event(KeyCode::Right));
        terminal
            .draw(|frame| {
                confirm.render(frame, Rect::new(0, 0, 60, 10));
            })
            .unwrap();
        let text = buffer_text(&terminal);
        assert!(text.contains("[Yes]"), "expected '[Yes]' in: {text}");
    }
}
