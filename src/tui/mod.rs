pub mod confirm;
pub mod select_list;
pub mod step_bar;
pub mod terminal;
pub mod text_input;
pub mod theme;

#[cfg(test)]
pub(crate) mod test_helpers;

pub use confirm::Confirm;
pub use select_list::{SelectList, SelectResult};
pub use terminal::run_tui;
pub use text_input::TextInput;

#[derive(Debug)]
pub enum FlowOutcome<T> {
    Continue,
    Done(T),
}
