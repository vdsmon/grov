use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState, Paragraph};

use super::theme;

#[derive(Debug, PartialEq, Eq)]
pub enum SelectResult {
    /// User selected a regular item (index into the original `items` slice).
    Item(usize),
    /// User selected an extra option (index into the `extra_options` slice).
    Extra(usize),
}

pub enum Action {
    Selected(SelectResult),
    Cancel,
    Continue,
}

pub struct SelectList {
    pub label: String,
    pub items: Vec<String>,
    pub extra_options: Vec<String>,
    pub filter: String,
    pub filtered_indices: Vec<usize>,
    pub list_state: ListState,
}

impl SelectList {
    pub fn new(label: impl Into<String>, items: Vec<String>, extra_options: Vec<String>) -> Self {
        let filtered_indices: Vec<usize> = (0..items.len()).collect();
        let mut list_state = ListState::default();
        if !filtered_indices.is_empty() || !extra_options.is_empty() {
            list_state.select(Some(0));
        }
        Self {
            label: label.into(),
            items,
            extra_options,
            filter: String::new(),
            filtered_indices,
            list_state,
        }
    }

    fn total_visible(&self) -> usize {
        self.filtered_indices.len() + self.extra_options.len()
    }

    fn update_filter(&mut self) {
        let lower = self.filter.to_lowercase();
        self.filtered_indices = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.to_lowercase().contains(&lower))
            .map(|(i, _)| i)
            .collect();

        // Reset selection
        if self.total_visible() > 0 {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }
    }

    fn move_up(&mut self) {
        let total = self.total_visible();
        if total == 0 {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let next = if current == 0 { total - 1 } else { current - 1 };
        self.list_state.select(Some(next));
    }

    fn move_down(&mut self) {
        let total = self.total_visible();
        if total == 0 {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let next = if current + 1 >= total { 0 } else { current + 1 };
        self.list_state.select(Some(next));
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
                KeyCode::Enter => {
                    if let Some(selected) = self.list_state.selected() {
                        let filtered_count = self.filtered_indices.len();
                        if selected < filtered_count {
                            return Action::Selected(SelectResult::Item(
                                self.filtered_indices[selected],
                            ));
                        } else {
                            return Action::Selected(SelectResult::Extra(
                                selected - filtered_count,
                            ));
                        }
                    }
                }
                KeyCode::Esc => return Action::Cancel,
                KeyCode::Up => self.move_up(),
                KeyCode::Down => self.move_down(),
                KeyCode::Backspace => {
                    self.filter.pop();
                    self.update_filter();
                }
                KeyCode::Char(c) => {
                    self.filter.push(*c);
                    self.update_filter();
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
        y += 1;

        // Filter input
        let filter_display = if self.filter.is_empty() {
            Line::from(Span::styled("  > (type to filter)", theme::DIM))
        } else {
            Line::from(vec![Span::raw("  > "), Span::raw(&self.filter)])
        };
        frame.render_widget(
            Paragraph::new(filter_display),
            Rect::new(area.x, y, area.width, 1),
        );
        y += 2; // blank line after filter

        // List items
        let available_height = area.height.saturating_sub(y - area.y + 2) as usize; // reserve 2 for help
        let mut list_items: Vec<ListItem> = Vec::new();

        for &orig_idx in &self.filtered_indices {
            list_items.push(ListItem::new(format!("    {}", self.items[orig_idx])));
        }

        // Separator + extra options
        if !self.extra_options.is_empty() && !self.filtered_indices.is_empty() {
            list_items.push(ListItem::new(Span::styled(
                "    ─────────────────",
                theme::DIM,
            )));
        }

        for opt in &self.extra_options {
            list_items.push(ListItem::new(Span::styled(
                format!("    {opt}"),
                theme::CYAN,
            )));
        }

        // Build the list widget — we need a mutable copy of list_state for rendering
        let total_items = self.filtered_indices.len() + self.extra_options.len();
        let separator_count = if !self.extra_options.is_empty() && !self.filtered_indices.is_empty()
        {
            1
        } else {
            0
        };

        // Map logical selection to visual index (accounting for separator)
        let visual_selection = self.list_state.selected().map(|sel| {
            if sel < self.filtered_indices.len() {
                sel
            } else {
                sel + separator_count
            }
        });

        let mut visual_state = ListState::default();
        visual_state.select(visual_selection);

        let list_height = (list_items.len().min(available_height)) as u16;
        let highlight_symbol = "  \u{276f} "; // ❯

        let list = List::new(list_items)
            .highlight_symbol(highlight_symbol)
            .highlight_style(theme::HIGHLIGHT);

        let list_area = Rect::new(area.x, y, area.width, list_height);
        frame.render_stateful_widget(list, list_area, &mut visual_state);

        // For total_items display (unused but kept for potential status line)
        let _ = total_items;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_list_with_selection() {
        let list = SelectList::new("test", vec!["a".into(), "b".into()], vec![]);
        assert_eq!(list.list_state.selected(), Some(0));
        assert_eq!(list.filtered_indices, vec![0, 1]);
    }

    #[test]
    fn filter_narrows_results() {
        let mut list = SelectList::new(
            "test",
            vec!["alpha".into(), "beta".into(), "gamma".into()],
            vec![],
        );
        list.filter = "al".to_string();
        list.update_filter();
        assert_eq!(list.filtered_indices, vec![0]);
    }

    #[test]
    fn filter_case_insensitive() {
        let mut list = SelectList::new("test", vec!["Alpha".into(), "beta".into()], vec![]);
        list.filter = "ALPHA".to_string();
        list.update_filter();
        assert_eq!(list.filtered_indices, vec![0]);
    }

    #[test]
    fn move_wraps_around() {
        let mut list = SelectList::new("test", vec!["a".into(), "b".into()], vec![]);
        assert_eq!(list.list_state.selected(), Some(0));
        list.move_up();
        assert_eq!(list.list_state.selected(), Some(1));
        list.move_down();
        assert_eq!(list.list_state.selected(), Some(0));
    }

    #[test]
    fn extra_options_included_in_total() {
        let list = SelectList::new("test", vec!["a".into()], vec!["extra".into()]);
        assert_eq!(list.total_visible(), 2);
    }

    #[test]
    fn renders_label_and_items() {
        use crate::tui::test_helpers::{buffer_text, test_terminal};
        use ratatui::layout::Rect;

        let mut terminal = test_terminal(60, 20);
        let list = SelectList::new(
            "Pick a branch",
            vec!["main".into(), "develop".into(), "feature-x".into()],
            vec![],
        );
        terminal
            .draw(|frame| {
                list.render(frame, Rect::new(0, 0, 60, 20));
            })
            .unwrap();
        let text = buffer_text(&terminal);
        assert!(text.contains("Pick a branch"), "expected label in: {text}");
        assert!(text.contains("main"), "expected 'main' in: {text}");
        assert!(text.contains("develop"), "expected 'develop' in: {text}");
        assert!(
            text.contains("feature-x"),
            "expected 'feature-x' in: {text}"
        );
    }

    #[test]
    fn renders_filtered_items() {
        use crate::tui::test_helpers::{buffer_text, test_terminal};
        use ratatui::layout::Rect;

        let mut terminal = test_terminal(60, 20);
        let mut list = SelectList::new(
            "Pick",
            vec!["main".into(), "develop".into(), "feature-x".into()],
            vec![],
        );
        list.filter = "dev".to_string();
        list.update_filter();
        terminal
            .draw(|frame| {
                list.render(frame, Rect::new(0, 0, 60, 20));
            })
            .unwrap();
        let text = buffer_text(&terminal);
        assert!(text.contains("develop"), "expected 'develop' in: {text}");
        // "main" should not appear as a list item (but might appear as part of other UI text)
        // Check that "main" doesn't appear in item lines (lines with 4-space indent)
        assert!(
            !text.contains("    main"),
            "should not have 'main' as item: {text}"
        );
    }

    #[test]
    fn renders_extra_options() {
        use crate::tui::test_helpers::{buffer_text, test_terminal};
        use ratatui::layout::Rect;

        let mut terminal = test_terminal(60, 20);
        let list = SelectList::new(
            "Pick",
            vec!["main".into()],
            vec!["Create a new branch".into()],
        );
        terminal
            .draw(|frame| {
                list.render(frame, Rect::new(0, 0, 60, 20));
            })
            .unwrap();
        let text = buffer_text(&terminal);
        assert!(
            text.contains("Create a new branch"),
            "expected extra option in: {text}"
        );
    }
}
