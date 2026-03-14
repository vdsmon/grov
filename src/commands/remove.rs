use std::io::{self, Write};

use console::style;
use crossterm::event::{self, Event};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::cli::RemoveMatchMode;
use crate::git::repo::find_bare_repo;
use crate::git::status::is_dirty;
use crate::git::worktree::{
    WorktreeInfo, delete_branch, list_worktrees, matches_branch_name, matches_dir_name,
    remove_worktree, safe_delete_branch, worktree_dir_name,
};
use crate::tui::FlowOutcome;
use crate::tui::confirm::{self, Confirm};
use crate::tui::select_list::{self, SelectList, SelectResult};
use crate::tui::terminal::run_tui;
use crate::tui::theme;

#[derive(Debug, PartialEq)]
pub(crate) struct RemoveChoice {
    pub worktree_index: usize,
    pub delete_branch: bool,
}

pub fn execute(
    name: Option<&str>,
    match_mode: RemoveMatchMode,
    do_delete_branch: bool,
    force: bool,
) -> anyhow::Result<()> {
    use std::io::IsTerminal;

    let cwd = std::env::current_dir()?;
    let repo = find_bare_repo(&cwd)?;
    let worktrees = list_worktrees(&repo)?;
    let is_tty = std::io::stdin().is_terminal();

    // Resolve which worktree to remove
    let (wt_index, should_delete_branch) = match name {
        Some(name) => (
            resolve_by_name(&worktrees, name, match_mode)?,
            do_delete_branch,
        ),
        None => {
            if !is_tty {
                anyhow::bail!("worktree name is required when stdin is not a terminal");
            }
            let choice = resolve_by_picker(&worktrees, do_delete_branch)?;
            (choice.worktree_index, choice.delete_branch)
        }
    };
    let wt = &worktrees[wt_index];

    if wt.is_bare {
        anyhow::bail!("cannot remove the bare repository entry");
    }

    // Check for dirty state
    if !force && is_dirty(&wt.path).unwrap_or(false) {
        anyhow::bail!("worktree has uncommitted changes (use --force to override)");
    }

    let branch_name = wt.branch.clone();
    let wt_path = wt.path.clone();

    remove_worktree(&repo, &wt_path, force)?;

    println!(
        "{} Removed worktree at {}",
        style("\u{2713}").green().bold(),
        style(wt_path.display()).bold(),
    );

    // Branch deletion logic
    if let Some(ref branch) = branch_name
        && should_delete_branch
    {
        match safe_delete_branch(&repo, branch) {
            Ok(()) => {
                println!(
                    "{} Deleted branch {}",
                    style("\u{2713}").green().bold(),
                    style(branch).cyan().bold(),
                );
            }
            Err(e) => {
                let msg = format!("{e:#}");
                if msg.contains("not fully merged") {
                    // Inline y/n prompt (no full TUI — this is a follow-up to an error)
                    eprint!(
                        "{} Branch has unmerged changes. Force delete? [y/N] ",
                        style("!").yellow().bold()
                    );
                    io::stderr().flush()?;
                    let mut answer = String::new();
                    io::stdin().read_line(&mut answer)?;
                    if answer.trim().eq_ignore_ascii_case("y") {
                        delete_branch(&repo, branch)?;
                        println!(
                            "{} Deleted branch {}",
                            style("\u{2713}").green().bold(),
                            style(branch).cyan().bold(),
                        );
                    }
                } else {
                    eprintln!(
                        "{} Could not delete branch: {e:#}",
                        style("!").yellow().bold()
                    );
                }
            }
        }
    }

    Ok(())
}

/// Resolve the worktree by name/match-mode. Returns the index into `worktrees`.
fn resolve_by_name(
    worktrees: &[WorktreeInfo],
    name: &str,
    match_mode: RemoveMatchMode,
) -> anyhow::Result<usize> {
    let matches: Vec<usize> = worktrees
        .iter()
        .enumerate()
        .filter(|(_, worktree)| match match_mode {
            RemoveMatchMode::Auto => {
                matches_branch_name(worktree, name) || matches_dir_name(worktree, name)
            }
            RemoveMatchMode::Branch => matches_branch_name(worktree, name),
            RemoveMatchMode::Dir => matches_dir_name(worktree, name),
        })
        .map(|(i, _)| i)
        .collect();

    if matches.is_empty() {
        anyhow::bail!("worktree not found: {name}");
    }
    if matches.len() > 1 {
        let candidates = matches
            .iter()
            .map(|&i| {
                let worktree = &worktrees[i];
                let branch = worktree.branch.as_deref().unwrap_or("<none>");
                let dir = worktree_dir_name(worktree);
                format!(
                    "  - branch={branch} dir={dir} path={}",
                    worktree.path.display()
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        anyhow::bail!(
            "ambiguous worktree name '{name}' matched multiple worktrees:\n{candidates}\nrerun with --match branch or --match dir"
        );
    }
    Ok(matches[0])
}

#[derive(Debug)]
enum PickerStep {
    SelectWorktree,
    ConfirmDeleteBranch { wt_index: usize },
}

pub(crate) struct RemovePicker {
    step: PickerStep,
    candidates: Vec<(usize, String)>,
    worktrees: Vec<WorktreeInfo>,
    flag_delete_branch: bool,
    select: SelectList,
    confirm: Confirm,
}

impl RemovePicker {
    pub(crate) fn new(worktrees: Vec<WorktreeInfo>, flag_delete_branch: bool) -> Self {
        let candidates: Vec<(usize, String)> = worktrees
            .iter()
            .enumerate()
            .filter(|(_, wt)| !wt.is_bare)
            .map(|(i, wt)| {
                let branch = wt.branch.as_deref().unwrap_or("<detached>");
                let dir = worktree_dir_name(wt);
                (i, format!("{branch} ({dir})"))
            })
            .collect();

        let display: Vec<String> = candidates.iter().map(|(_, label)| label.clone()).collect();
        let select = SelectList::new("Select a worktree to remove", display, vec![]);
        let confirm = Confirm::new("");

        Self {
            step: PickerStep::SelectWorktree,
            candidates,
            worktrees,
            flag_delete_branch,
            select,
            confirm,
        }
    }

    pub(crate) fn handle_event(
        &mut self,
        event: &Event,
    ) -> anyhow::Result<FlowOutcome<RemoveChoice>> {
        match &self.step {
            PickerStep::SelectWorktree => match self.select.handle_event(event) {
                select_list::Action::Selected(SelectResult::Item(i)) => {
                    let wt_index = self.candidates[i].0;
                    let wt = &self.worktrees[wt_index];

                    // If --delete-branch flag was passed, skip confirm
                    if self.flag_delete_branch {
                        return Ok(FlowOutcome::Done(RemoveChoice {
                            worktree_index: wt_index,
                            delete_branch: true,
                        }));
                    }

                    // If worktree has a branch, ask about deletion
                    if let Some(ref branch) = wt.branch {
                        self.confirm = Confirm::new(format!("Delete branch '{branch}' too?"));
                        self.step = PickerStep::ConfirmDeleteBranch { wt_index };
                    } else {
                        return Ok(FlowOutcome::Done(RemoveChoice {
                            worktree_index: wt_index,
                            delete_branch: false,
                        }));
                    }
                }
                select_list::Action::Selected(SelectResult::Extra(_)) => unreachable!(),
                select_list::Action::Cancel => anyhow::bail!("cancelled"),
                select_list::Action::Continue => {}
            },
            PickerStep::ConfirmDeleteBranch { wt_index } => {
                let wt_index = *wt_index;
                match self.confirm.handle_event(event) {
                    confirm::Action::Confirmed(yes) => {
                        return Ok(FlowOutcome::Done(RemoveChoice {
                            worktree_index: wt_index,
                            delete_branch: yes,
                        }));
                    }
                    confirm::Action::Cancel => {
                        let display: Vec<String> = self
                            .candidates
                            .iter()
                            .map(|(_, label)| label.clone())
                            .collect();
                        self.select =
                            SelectList::new("Select a worktree to remove", display, vec![]);
                        self.step = PickerStep::SelectWorktree;
                    }
                    confirm::Action::Continue => {}
                }
            }
        }
        Ok(FlowOutcome::Continue)
    }

    pub(crate) fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        let widget_area = Rect::new(0, 1, area.width, area.height.saturating_sub(3));

        match &self.step {
            PickerStep::SelectWorktree => {
                self.select.render(frame, widget_area);
            }
            PickerStep::ConfirmDeleteBranch { .. } => {
                self.confirm.render(frame, widget_area);
            }
        }

        let help = match &self.step {
            PickerStep::SelectWorktree => theme::HELP_SELECT,
            PickerStep::ConfirmDeleteBranch { .. } => theme::HELP_CONFIRM,
        };
        let help_y = area.height.saturating_sub(1);
        let help_line = Line::from(Span::styled(format!("  {help}"), theme::DIM));
        frame.render_widget(
            Paragraph::new(help_line),
            Rect::new(0, help_y, area.width, 1),
        );
    }
}

/// Show an interactive TUI picker for worktree selection + optional branch deletion confirm.
fn resolve_by_picker(
    worktrees: &[WorktreeInfo],
    flag_delete_branch: bool,
) -> anyhow::Result<RemoveChoice> {
    if worktrees.iter().filter(|wt| !wt.is_bare).count() == 0 {
        anyhow::bail!("no worktrees to remove");
    }

    run_tui(|terminal| {
        let mut picker = RemovePicker::new(worktrees.to_vec(), flag_delete_branch);

        loop {
            terminal.draw(|frame| picker.render(frame))?;

            if event::poll(std::time::Duration::from_millis(100))? {
                match picker.handle_event(&event::read()?)? {
                    FlowOutcome::Continue => {}
                    FlowOutcome::Done(result) => return Ok(result),
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::tui::test_helpers::*;

    fn feed_picker(
        picker: &mut RemovePicker,
        events: &[Event],
    ) -> anyhow::Result<FlowOutcome<RemoveChoice>> {
        for ev in events {
            match picker.handle_event(ev)? {
                FlowOutcome::Done(r) => return Ok(FlowOutcome::Done(r)),
                FlowOutcome::Continue => {}
            }
        }
        Ok(FlowOutcome::Continue)
    }

    fn make_worktrees() -> Vec<WorktreeInfo> {
        vec![
            WorktreeInfo {
                path: PathBuf::from("/project/repo.git"),
                head: "abc1234".to_string(),
                branch: None,
                is_bare: true,
            },
            WorktreeInfo {
                path: PathBuf::from("/project/dev_main"),
                head: "def5678".to_string(),
                branch: Some("main".to_string()),
                is_bare: false,
            },
            WorktreeInfo {
                path: PathBuf::from("/project/dev_feature"),
                head: "ghi9012".to_string(),
                branch: Some("feature".to_string()),
                is_bare: false,
            },
            WorktreeInfo {
                path: PathBuf::from("/project/dev_detached"),
                head: "jkl3456".to_string(),
                branch: None,
                is_bare: false,
            },
        ]
    }

    #[test]
    fn select_with_branch_shows_confirm() {
        let mut picker = RemovePicker::new(make_worktrees(), false);
        // First non-bare is index 0 in candidates (worktree index 1, "main")
        // Press Enter to select it
        let result = feed_picker(&mut picker, &[enter()]).unwrap();
        // Should move to confirm step, not return Done yet
        assert!(matches!(result, FlowOutcome::Continue));
        assert!(matches!(
            picker.step,
            PickerStep::ConfirmDeleteBranch { .. }
        ));
    }

    #[test]
    fn select_without_branch_returns() {
        let mut picker = RemovePicker::new(make_worktrees(), false);
        // Navigate down to the detached worktree (3rd non-bare = index 2 in candidates)
        let events = vec![
            key_event(crossterm::event::KeyCode::Down), // feature
            key_event(crossterm::event::KeyCode::Down), // detached
            enter(),
        ];

        let result = feed_picker(&mut picker, &events).unwrap();
        match result {
            FlowOutcome::Done(choice) => {
                assert_eq!(choice.worktree_index, 3);
                assert!(!choice.delete_branch);
            }
            FlowOutcome::Continue => panic!("expected Done"),
        }
    }

    #[test]
    fn flag_delete_branch_skips_confirm() {
        let mut picker = RemovePicker::new(make_worktrees(), true);
        // Select first (main) — should skip confirm because flag is set
        let result = feed_picker(&mut picker, &[enter()]).unwrap();
        match result {
            FlowOutcome::Done(choice) => {
                assert_eq!(choice.worktree_index, 1);
                assert!(choice.delete_branch);
            }
            FlowOutcome::Continue => panic!("expected Done"),
        }
    }

    #[test]
    fn confirm_yes() {
        let mut picker = RemovePicker::new(make_worktrees(), false);
        // Select main, then confirm with 'y'
        let events = vec![
            enter(),       // select main
            key_char('y'), // confirm yes
        ];

        let result = feed_picker(&mut picker, &events).unwrap();
        match result {
            FlowOutcome::Done(choice) => {
                assert_eq!(choice.worktree_index, 1);
                assert!(choice.delete_branch);
            }
            FlowOutcome::Continue => panic!("expected Done"),
        }
    }

    #[test]
    fn confirm_no() {
        let mut picker = RemovePicker::new(make_worktrees(), false);
        // Select main, then confirm with 'n'
        let events = vec![
            enter(),       // select main
            key_char('n'), // confirm no
        ];

        let result = feed_picker(&mut picker, &events).unwrap();
        match result {
            FlowOutcome::Done(choice) => {
                assert_eq!(choice.worktree_index, 1);
                assert!(!choice.delete_branch);
            }
            FlowOutcome::Continue => panic!("expected Done"),
        }
    }

    #[test]
    fn esc_from_confirm_returns_to_list() {
        let mut picker = RemovePicker::new(make_worktrees(), false);
        // Select main, then Esc from confirm
        let events = vec![
            enter(), // select main
            esc(),   // back from confirm
        ];

        let result = feed_picker(&mut picker, &events).unwrap();
        assert!(matches!(result, FlowOutcome::Continue));
        assert!(matches!(picker.step, PickerStep::SelectWorktree));
    }
}
