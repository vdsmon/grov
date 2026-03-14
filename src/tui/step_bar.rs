use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::theme;

pub struct StepBar<'a> {
    pub steps: &'a [&'a str],
    pub current: usize,
}

impl<'a> StepBar<'a> {
    pub fn new(steps: &'a [&'a str], current: usize) -> Self {
        Self { steps, current }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let mut spans: Vec<Span> = Vec::new();
        spans.push(Span::raw("  "));

        for (i, &step) in self.steps.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled("  \u{203a}  ", theme::DIM)); // ›
            }
            if i < self.current {
                spans.push(Span::styled("\u{2713} ", theme::GREEN)); // ✓
                spans.push(Span::styled(step, theme::GREEN));
            } else if i == self.current {
                spans.push(Span::styled("\u{25cf} ", theme::CYAN)); // ●
                spans.push(Span::styled(step, theme::CYAN));
            } else {
                spans.push(Span::styled(step, theme::DIM));
            }
        }

        let line = Line::from(spans);
        frame.render_widget(Paragraph::new(line), area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::test_helpers::{buffer_text, test_terminal};
    use ratatui::layout::Rect;

    #[test]
    fn renders_step_names() {
        let mut terminal = test_terminal(80, 3);
        let step_bar = StepBar::new(&["URL", "Name", "Prefix", "Branch", "Confirm"], 2);
        terminal
            .draw(|frame| {
                step_bar.render(frame, Rect::new(0, 0, 80, 1));
            })
            .unwrap();
        let text = buffer_text(&terminal);
        assert!(text.contains("URL"), "expected 'URL' in: {text}");
        assert!(text.contains("Name"), "expected 'Name' in: {text}");
        assert!(text.contains("Prefix"), "expected 'Prefix' in: {text}");
        assert!(text.contains("Branch"), "expected 'Branch' in: {text}");
        assert!(text.contains("Confirm"), "expected 'Confirm' in: {text}");
    }

    #[test]
    fn renders_completed_marker() {
        let mut terminal = test_terminal(80, 3);
        // Step 2 means steps 0,1 are completed (✓), step 2 is current (●)
        let step_bar = StepBar::new(&["URL", "Name", "Prefix", "Branch", "Confirm"], 2);
        terminal
            .draw(|frame| {
                step_bar.render(frame, Rect::new(0, 0, 80, 1));
            })
            .unwrap();
        let text = buffer_text(&terminal);
        // Completed steps should have ✓ marker
        assert!(text.contains("\u{2713}"), "expected checkmark in: {text}");
        // Current step should have ● marker
        assert!(
            text.contains("\u{25cf}"),
            "expected bullet marker in: {text}"
        );
    }
}
