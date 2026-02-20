use std::path::Path;

use console::style;
use crossterm::event::{self, Event};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::config::read_config;
use crate::git::executor::run_git_ok;
use crate::git::repo::{current_branch, default_branch, find_bare_repo};
use crate::git::worktree::{
    add_worktree, available_branches, branch_exists_local, branch_exists_remote, list_worktrees,
};
use crate::paths::{relative_from, worktree_dir};
use crate::tui::FlowOutcome;
use crate::tui::select_list::{self, SelectList, SelectResult};
use crate::tui::terminal::run_tui;
use crate::tui::text_input::{self, TextInput};
use crate::tui::theme;

#[derive(Debug, PartialEq)]
enum BaseBranchAction {
    UseBase(String),
    Prompt { default: Option<String> },
    ErrorNotTty,
}

fn resolve_base_branch(
    base_flag: Option<&str>,
    current_branch: Option<&str>,
    is_tty: bool,
) -> BaseBranchAction {
    if let Some(b) = base_flag {
        return BaseBranchAction::UseBase(b.to_string());
    }
    if !is_tty {
        return BaseBranchAction::ErrorNotTty;
    }
    BaseBranchAction::Prompt {
        default: current_branch.map(|s| s.to_string()),
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum BranchChoice {
    Existing(String),
    New { name: String, base: String },
}

#[derive(Debug)]
enum AddStep {
    SelectBranch,
    NewBranchName,
    BaseBranch { new_name: String },
}

pub(crate) struct AddPicker {
    step: AddStep,
    branches: Vec<String>,
    extras: Vec<String>,
    default_base: Option<String>,
    select: SelectList,
    input: TextInput,
}

impl AddPicker {
    pub(crate) fn new(branches: Vec<String>, default_base: Option<String>) -> Self {
        let step = if branches.is_empty() {
            AddStep::NewBranchName
        } else {
            AddStep::SelectBranch
        };

        let extras = if branches.is_empty() {
            vec![]
        } else {
            vec!["Create a new branch".to_string()]
        };
        let select = SelectList::new("Select a branch", branches.clone(), extras.clone());
        let input = TextInput::new("New branch name");

        Self {
            step,
            branches,
            extras,
            default_base,
            select,
            input,
        }
    }

    pub(crate) fn handle_event(
        &mut self,
        event: &Event,
    ) -> anyhow::Result<FlowOutcome<BranchChoice>> {
        match &self.step {
            AddStep::SelectBranch => match self.select.handle_event(event) {
                select_list::Action::Selected(result) => match result {
                    SelectResult::Item(i) => {
                        return Ok(FlowOutcome::Done(BranchChoice::Existing(
                            self.branches[i].clone(),
                        )));
                    }
                    SelectResult::Extra(_) => {
                        self.input = TextInput::new("New branch name");
                        self.step = AddStep::NewBranchName;
                    }
                },
                select_list::Action::Cancel => anyhow::bail!("cancelled"),
                select_list::Action::Continue => {}
            },
            AddStep::NewBranchName => match self.input.handle_event(event) {
                text_input::Action::Submit(name) => {
                    if !name.is_empty() {
                        let base_default =
                            self.default_base.as_deref().unwrap_or("main").to_string();
                        let new_name = name.clone();
                        self.input = TextInput::new(format!("Base branch for '{name}'"))
                            .with_default(base_default);
                        self.step = AddStep::BaseBranch { new_name };
                    }
                }
                text_input::Action::Cancel => {
                    if self.branches.is_empty() {
                        anyhow::bail!("cancelled");
                    }
                    self.select = SelectList::new(
                        "Select a branch",
                        self.branches.clone(),
                        self.extras.clone(),
                    );
                    self.step = AddStep::SelectBranch;
                }
                text_input::Action::Continue => {}
            },
            AddStep::BaseBranch { new_name } => match self.input.handle_event(event) {
                text_input::Action::Submit(base) => {
                    return Ok(FlowOutcome::Done(BranchChoice::New {
                        name: new_name.clone(),
                        base,
                    }));
                }
                text_input::Action::Cancel => {
                    self.input = TextInput::new("New branch name").with_initial(new_name.clone());
                    self.step = AddStep::NewBranchName;
                }
                text_input::Action::Continue => {}
            },
        }
        Ok(FlowOutcome::Continue)
    }

    pub(crate) fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        let widget_area = Rect::new(0, 1, area.width, area.height.saturating_sub(3));

        match &self.step {
            AddStep::SelectBranch => {
                self.select.render(frame, widget_area);
            }
            AddStep::NewBranchName => {
                self.input.render(frame, widget_area);
            }
            AddStep::BaseBranch { new_name } => {
                // Show the chosen new branch name above the base input
                let info = Line::from(vec![
                    Span::styled("  New branch: ", theme::DIM),
                    Span::raw(new_name.as_str()),
                ]);
                frame.render_widget(Paragraph::new(info), Rect::new(0, 1, area.width, 1));
                let base_area = Rect::new(0, 3, area.width, area.height.saturating_sub(5));
                self.input.render(frame, base_area);
            }
        }

        // Help text
        let help = match &self.step {
            AddStep::SelectBranch => theme::HELP_SELECT,
            AddStep::NewBranchName | AddStep::BaseBranch { .. } => theme::HELP_WIZARD,
        };
        let help_y = area.height.saturating_sub(1);
        let help_line = Line::from(Span::styled(format!("  {help}"), theme::DIM));
        frame.render_widget(
            Paragraph::new(help_line),
            Rect::new(0, help_y, area.width, 1),
        );
    }
}

fn run_branch_picker(
    branches: Vec<String>,
    default_base: Option<String>,
) -> anyhow::Result<BranchChoice> {
    run_tui(|terminal| {
        let mut picker = AddPicker::new(branches, default_base);

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

pub fn execute(
    branch: Option<&str>,
    base: Option<&str>,
    custom_path: Option<&Path>,
) -> anyhow::Result<()> {
    use std::io::IsTerminal;

    let cwd = std::env::current_dir()?;
    let repo = find_bare_repo(&cwd)?;
    let config = read_config(&repo);

    // Fetch latest
    if let Err(err) = run_git_ok(Some(&repo), &["fetch", "origin"]) {
        eprintln!(
            "{} could not fetch from origin: {err:#}; continuing with local refs",
            style("warning:").yellow().bold()
        );
    }

    // Resolve branch — use argument or prompt interactively
    let branch = match branch {
        Some(b) => b.to_string(),
        None => {
            if !std::io::stdin().is_terminal() {
                anyhow::bail!("branch argument is required when stdin is not a terminal");
            }

            let worktrees = list_worktrees(&repo)?;
            let branches = available_branches(&repo, &worktrees)?;
            let cur = current_branch(&cwd).unwrap_or(None);

            match run_branch_picker(branches, cur)? {
                BranchChoice::Existing(name) => name,
                BranchChoice::New { name, base: b } => {
                    // Create new branch with base — handle inline
                    let wt_path = match custom_path {
                        Some(p) => p.to_path_buf(),
                        None => worktree_dir(&repo, &name, &config.worktree.prefix),
                    };
                    if wt_path.exists() {
                        anyhow::bail!("worktree directory already exists at {}", wt_path.display());
                    }
                    add_worktree(&repo, &wt_path, Some(&b), &["-b", &name])?;
                    print_success(&name, &wt_path, &cwd);
                    return Ok(());
                }
            }
        }
    };

    // Determine worktree path
    let wt_path = match custom_path {
        Some(p) => p.to_path_buf(),
        None => worktree_dir(&repo, &branch, &config.worktree.prefix),
    };

    // Check if worktree dir already exists
    if wt_path.exists() {
        anyhow::bail!("worktree directory already exists at {}", wt_path.display());
    }

    let remote_ref = format!("origin/{branch}");

    if branch_exists_local(&repo, &branch) {
        // Local branch exists → check it out
        add_worktree(&repo, &wt_path, Some(&branch), &[])?;
    } else if branch_exists_remote(&repo, &branch) {
        // Remote branch exists → git worktree add --track -b <branch> <path> origin/<branch>
        add_worktree(
            &repo,
            &wt_path,
            Some(&remote_ref),
            &["--track", "-b", &branch],
        )?;
    } else {
        // New branch — resolve base via flag, prompt, or non-TTY error
        let current = current_branch(&cwd).unwrap_or(None);
        let is_tty = std::io::stdin().is_terminal();
        let base_branch = match resolve_base_branch(base, current.as_deref(), is_tty) {
            BaseBranchAction::UseBase(b) => b,
            BaseBranchAction::Prompt {
                default: prompt_default,
            } => {
                // Use TUI for base branch prompt
                let fallback;
                let effective_default = match &prompt_default {
                    Some(b) => b.as_str(),
                    None => {
                        fallback = default_branch(&repo)?;
                        fallback.as_str()
                    }
                };
                run_base_branch_prompt(&branch, effective_default)?
            }
            BaseBranchAction::ErrorNotTty => {
                anyhow::bail!("--base is required when stdin is not a terminal");
            }
        };
        add_worktree(&repo, &wt_path, Some(&base_branch), &["-b", &branch])?;
    }

    print_success(&branch, &wt_path, &cwd);

    Ok(())
}

fn run_base_branch_prompt(branch: &str, default: &str) -> anyhow::Result<String> {
    run_tui(|terminal| {
        let mut input =
            TextInput::new(format!("Base branch for new branch '{branch}'")).with_default(default);

        loop {
            terminal.draw(|frame| {
                let area = frame.area();
                let widget_area = Rect::new(0, 1, area.width, area.height.saturating_sub(3));
                input.render(frame, widget_area);

                let help_y = area.height.saturating_sub(1);
                let help_line = Line::from(Span::styled(
                    format!("  {}", theme::HELP_WIZARD),
                    theme::DIM,
                ));
                frame.render_widget(
                    Paragraph::new(help_line),
                    Rect::new(0, help_y, area.width, 1),
                );
            })?;

            if event::poll(std::time::Duration::from_millis(100))? {
                let ev = event::read()?;
                match input.handle_event(&ev) {
                    text_input::Action::Submit(val) => return Ok(val),
                    text_input::Action::Cancel => anyhow::bail!("cancelled"),
                    text_input::Action::Continue => {}
                }
            }
        }
    })
}

fn print_success(branch: &str, wt_path: &Path, cwd: &Path) {
    println!(
        "{} Created worktree at {} on branch {}",
        style("\u{2713}").green().bold(),
        style(wt_path.display()).bold(),
        style(branch).cyan().bold(),
    );

    let rel = relative_from(wt_path, cwd);
    if rel != Path::new(".") {
        let display = rel.display().to_string();
        let cd_arg = if display.contains(' ') {
            format!("\"{}\"", display)
        } else {
            display
        };
        println!(
            "{}",
            style(format!("  To start working:  cd {cd_arg}")).dim()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::test_helpers::*;

    fn feed_picker(
        picker: &mut AddPicker,
        events: &[Event],
    ) -> anyhow::Result<FlowOutcome<BranchChoice>> {
        for ev in events {
            match picker.handle_event(ev)? {
                FlowOutcome::Done(r) => return Ok(FlowOutcome::Done(r)),
                FlowOutcome::Continue => {}
            }
        }
        Ok(FlowOutcome::Continue)
    }

    #[test]
    fn resolve_base_provided() {
        assert_eq!(
            resolve_base_branch(Some("develop"), Some("feat"), true),
            BaseBranchAction::UseBase("develop".to_string())
        );
    }

    #[test]
    fn resolve_base_provided_overrides_non_tty() {
        assert_eq!(
            resolve_base_branch(Some("develop"), None, false),
            BaseBranchAction::UseBase("develop".to_string())
        );
    }

    #[test]
    fn resolve_tty_with_current_branch() {
        assert_eq!(
            resolve_base_branch(None, Some("feat"), true),
            BaseBranchAction::Prompt {
                default: Some("feat".to_string())
            }
        );
    }

    #[test]
    fn resolve_tty_without_current_branch() {
        assert_eq!(
            resolve_base_branch(None, None, true),
            BaseBranchAction::Prompt { default: None }
        );
    }

    #[test]
    fn resolve_non_tty_without_base() {
        assert_eq!(
            resolve_base_branch(None, Some("feat"), false),
            BaseBranchAction::ErrorNotTty
        );
    }

    #[test]
    fn resolve_non_tty_no_base_no_branch() {
        assert_eq!(
            resolve_base_branch(None, None, false),
            BaseBranchAction::ErrorNotTty
        );
    }

    // --- Flow tests ---

    #[test]
    fn select_existing_branch() {
        let mut picker = AddPicker::new(vec!["main".into(), "develop".into()], Some("main".into()));
        // Press Enter on first item (main)
        let result = feed_picker(&mut picker, &[enter()]).unwrap();
        match result {
            FlowOutcome::Done(choice) => {
                assert_eq!(choice, BranchChoice::Existing("main".to_string()));
            }
            FlowOutcome::Continue => panic!("expected Done"),
        }
    }

    #[test]
    fn create_new_branch() {
        let mut picker = AddPicker::new(vec!["main".into(), "develop".into()], Some("main".into()));
        let mut events: Vec<Event> = Vec::new();
        // Navigate down past items to "Create a new branch" extra option
        events.push(key_event(crossterm::event::KeyCode::Down)); // develop
        events.push(key_event(crossterm::event::KeyCode::Down)); // extra: Create a new branch
        events.push(enter()); // select extra
        // Type new branch name
        events.extend(type_string("feature-x"));
        events.push(enter());
        // Accept default base
        events.push(enter());

        let result = feed_picker(&mut picker, &events).unwrap();
        match result {
            FlowOutcome::Done(choice) => {
                assert_eq!(
                    choice,
                    BranchChoice::New {
                        name: "feature-x".to_string(),
                        base: "main".to_string(),
                    }
                );
            }
            FlowOutcome::Continue => panic!("expected Done"),
        }
    }

    #[test]
    fn esc_from_new_name_returns_to_list() {
        let mut picker = AddPicker::new(vec!["main".into(), "develop".into()], Some("main".into()));
        let events = vec![
            key_event(crossterm::event::KeyCode::Down),
            key_event(crossterm::event::KeyCode::Down),
            enter(),
            esc(),
        ];

        let result = feed_picker(&mut picker, &events).unwrap();
        assert!(matches!(result, FlowOutcome::Continue));
        assert!(matches!(picker.step, AddStep::SelectBranch));
    }

    #[test]
    fn empty_branches_starts_at_new_name() {
        let mut picker = AddPicker::new(vec![], Some("main".into()));
        // Should start at NewBranchName step
        assert!(matches!(picker.step, AddStep::NewBranchName));

        // Type a name and submit
        let mut events: Vec<Event> = Vec::new();
        events.extend(type_string("new-feature"));
        events.push(enter());
        events.push(enter()); // accept default base

        let result = feed_picker(&mut picker, &events).unwrap();
        match result {
            FlowOutcome::Done(choice) => {
                assert_eq!(
                    choice,
                    BranchChoice::New {
                        name: "new-feature".to_string(),
                        base: "main".to_string(),
                    }
                );
            }
            FlowOutcome::Continue => panic!("expected Done"),
        }
    }

    #[test]
    fn renders_branch_list() {
        let picker = AddPicker::new(
            vec!["main".into(), "develop".into(), "feature-y".into()],
            Some("main".into()),
        );
        let mut terminal = test_terminal(80, 24);
        terminal.draw(|frame| picker.render(frame)).unwrap();
        let text = buffer_text(&terminal);
        assert!(text.contains("main"), "expected 'main' in: {text}");
        assert!(text.contains("develop"), "expected 'develop' in: {text}");
        assert!(
            text.contains("feature-y"),
            "expected 'feature-y' in: {text}"
        );
    }
}
