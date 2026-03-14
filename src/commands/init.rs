use std::path::Path;

use console::style;
use crossterm::event::{self, Event};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::config::{GrovConfig, WorktreeConfig, write_config};
use crate::git::executor::run_git_ok;
use crate::git::worktree::add_worktree;
use crate::paths::{relative_from, repo_name_from_url, worktree_dir};
use crate::tui::FlowOutcome;
use crate::tui::confirm::{self, Confirm};
use crate::tui::step_bar::StepBar;
use crate::tui::terminal::run_tui;
use crate::tui::text_input::{self, TextInput};
use crate::tui::theme;

/// Detect the default branch from a remote URL via `git ls-remote`.
fn detect_default_branch_remote(url: &str) -> Option<String> {
    let output = run_git_ok(None, &["ls-remote", "--symref", url, "HEAD"]).ok()?;
    for line in output.lines() {
        if let Some(rest) = line.strip_prefix("ref: refs/heads/") {
            return rest.split('\t').next().map(|s| s.to_string());
        }
    }
    None
}

const STEPS: &[&str] = &["URL", "Name", "Prefix", "Branch", "Confirm"];

#[derive(Debug, PartialEq)]
pub(crate) struct WizardResult {
    pub url: String,
    pub name: String,
    pub prefix: String,
    pub branch: String,
}

pub(crate) struct InitWizard {
    pub step: usize,
    pub url: String,
    pub name: String,
    pub prefix: String,
    pub branch: String,
    pub detected_branch: Option<String>,
    pub skip_detection: bool,
    pub input: TextInput,
    pub confirm: Confirm,
}

impl InitWizard {
    pub(crate) fn new() -> Self {
        let mut wizard = Self {
            step: 0,
            url: String::new(),
            name: String::new(),
            prefix: String::new(),
            branch: String::new(),
            detected_branch: None,
            skip_detection: false,
            input: TextInput::new("Repository URL"),
            confirm: Confirm::new("Proceed with initialization?"),
        };
        wizard.setup_step();
        wizard
    }

    fn setup_step(&mut self) {
        match self.step {
            0 => {
                let mut input = TextInput::new("Repository URL");
                if !self.url.is_empty() {
                    input = input.with_initial(&self.url);
                }
                self.input = input;
            }
            1 => {
                let derived = repo_name_from_url(&self.url);
                let mut input = TextInput::new("Project name").with_default(derived);
                if !self.name.is_empty() {
                    input = input.with_initial(&self.name);
                }
                self.input = input;
            }
            2 => {
                let mut input =
                    TextInput::new("Worktree prefix (e.g. short alias, blank for none)")
                        .with_default("");
                if !self.prefix.is_empty() {
                    input = input.with_initial(&self.prefix);
                }
                self.input = input;
            }
            3 => {
                let default = self
                    .detected_branch
                    .as_deref()
                    .unwrap_or("main")
                    .to_string();
                let mut input = TextInput::new("Default branch").with_default(default);
                if !self.branch.is_empty() {
                    input = input.with_initial(&self.branch);
                }
                self.input = input;
            }
            4 => {
                self.confirm = Confirm::new("Proceed with initialization?");
            }
            _ => {}
        }
    }

    pub(crate) fn handle_event(
        &mut self,
        event: &Event,
    ) -> anyhow::Result<FlowOutcome<WizardResult>> {
        if self.step < 4 {
            match self.input.handle_event(event) {
                text_input::Action::Submit(val) => match self.step {
                    0 => {
                        if val.is_empty() {
                            return Ok(FlowOutcome::Continue);
                        }
                        self.url = val;
                        if !self.skip_detection {
                            self.detected_branch = detect_default_branch_remote(&self.url);
                        }
                        self.step = 1;
                        self.setup_step();
                    }
                    1 => {
                        self.name = val;
                        self.step = 2;
                        self.setup_step();
                    }
                    2 => {
                        self.prefix = val;
                        self.step = 3;
                        self.setup_step();
                    }
                    3 => {
                        self.branch = val;
                        self.step = 4;
                        self.setup_step();
                    }
                    _ => {}
                },
                text_input::Action::Cancel => {
                    if self.step == 0 {
                        anyhow::bail!("cancelled");
                    }
                    self.step -= 1;
                    self.setup_step();
                }
                text_input::Action::Continue => {}
            }
        } else {
            match self.confirm.handle_event(event) {
                confirm::Action::Confirmed(true) => {
                    return Ok(FlowOutcome::Done(WizardResult {
                        url: self.url.clone(),
                        name: self.name.clone(),
                        prefix: self.prefix.clone(),
                        branch: self.branch.clone(),
                    }));
                }
                confirm::Action::Confirmed(false) => {
                    anyhow::bail!("cancelled");
                }
                confirm::Action::Cancel => {
                    self.step = 3;
                    self.setup_step();
                }
                confirm::Action::Continue => {}
            }
        }
        Ok(FlowOutcome::Continue)
    }

    pub(crate) fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        let mut y = 1u16; // top margin

        // Step bar
        let step_bar = StepBar::new(STEPS, self.step);
        step_bar.render(frame, Rect::new(0, y, area.width, 1));
        y += 2;

        // Completed fields
        let fields: &[(&str, &str)] = &[
            ("URL:", &self.url),
            ("Name:", &self.name),
            ("Prefix:", &self.prefix),
            ("Branch:", &self.branch),
        ];
        for (i, &(label, value)) in fields.iter().enumerate() {
            if i < self.step && !value.is_empty() {
                let line = Line::from(vec![
                    Span::styled(format!("  {label:<12}"), theme::DIM),
                    Span::raw(value),
                ]);
                frame.render_widget(Paragraph::new(line), Rect::new(0, y, area.width, 1));
                y += 1;
            }
        }
        // Show prefix as "(none)" if step past prefix and it's empty
        if self.step > 2 && self.prefix.is_empty() {
            let line = Line::from(vec![
                Span::styled("  Prefix:   ", theme::DIM),
                Span::raw("(none)"),
            ]);
            frame.render_widget(Paragraph::new(line), Rect::new(0, y, area.width, 1));
            y += 1;
        }

        if self.step > 0 {
            y += 1; // gap
        }

        // Current widget
        let widget_area = Rect::new(0, y, area.width, area.height.saturating_sub(y + 2));
        if self.step < 4 {
            self.input.render(frame, widget_area);
        } else {
            self.confirm.render(frame, widget_area);
        }

        // Help text at bottom
        let help = if self.step == 0 {
            "Enter confirm  \u{b7}  Ctrl+C cancel"
        } else {
            theme::HELP_WIZARD
        };
        let help_y = area.height.saturating_sub(1);
        let help_line = Line::from(Span::styled(format!("  {help}"), theme::DIM));
        frame.render_widget(
            Paragraph::new(help_line),
            Rect::new(0, help_y, area.width, 1),
        );
    }
}

fn run_wizard() -> anyhow::Result<WizardResult> {
    run_tui(|terminal| {
        let mut wizard = InitWizard::new();

        loop {
            terminal.draw(|frame| wizard.render(frame))?;

            if event::poll(std::time::Duration::from_millis(100))? {
                match wizard.handle_event(&event::read()?)? {
                    FlowOutcome::Continue => {}
                    FlowOutcome::Done(result) => return Ok(result),
                }
            }
        }
    })
}

pub fn execute(path: Option<&Path>) -> anyhow::Result<()> {
    let result = run_wizard()?;
    execute_clone_and_setup(
        &result.url,
        &result.name,
        &result.prefix,
        &result.branch,
        path,
    )
}

fn execute_clone_and_setup(
    url: &str,
    project_name: &str,
    prefix: &str,
    branch: &str,
    path: Option<&Path>,
) -> anyhow::Result<()> {
    let parent = match path {
        Some(p) => p.to_path_buf(),
        None => std::env::current_dir()?,
    };

    let project_dir = parent.join(project_name);
    if project_dir.exists() {
        anyhow::bail!("directory already exists: {}", project_dir.display());
    }
    std::fs::create_dir_all(&project_dir)?;

    let bare_path = project_dir.join("repo.git");
    let bare_str = bare_path.to_string_lossy().to_string();
    run_git_ok(None, &["clone", "--bare", url, &bare_str])?;

    let config = GrovConfig {
        worktree: WorktreeConfig {
            prefix: prefix.to_string(),
        },
    };
    write_config(&bare_path, &config)?;

    run_git_ok(
        Some(&bare_path),
        &[
            "config",
            "remote.origin.fetch",
            "+refs/heads/*:refs/remotes/origin/*",
        ],
    )?;

    run_git_ok(Some(&bare_path), &["fetch", "origin"])?;

    let wt_path = worktree_dir(&bare_path, branch, prefix);
    add_worktree(&bare_path, &wt_path, Some(branch), &[])?;

    println!(
        "\n{} Initialized {}/\n\n    {:<12}{}\n    {:<12}{}",
        style("\u{2713}").green().bold(),
        style(project_name).bold(),
        "bare repo",
        style(format!("{}/repo.git", project_name)).dim(),
        "worktree",
        style(
            wt_path
                .file_name()
                .map(|n| format!("{}/{}", project_name, n.to_string_lossy()))
                .expect("worktree path must have a file name")
        )
        .dim(),
    );

    let cwd = std::env::current_dir()?;
    let project_rel = relative_from(&project_dir, &cwd);
    if project_rel != Path::new(".") {
        let display = project_rel.display().to_string();
        let cd_arg = if display.contains(' ') {
            format!("\"{}\"", display)
        } else {
            display
        };
        println!(
            "{}",
            style(format!("  To enter the project:  cd {cd_arg}")).dim()
        );
    }
    let wt_rel = relative_from(&wt_path, &cwd);
    if wt_rel != Path::new(".") {
        let display = wt_rel.display().to_string();
        let cd_arg = if display.contains(' ') {
            format!("\"{}\"", display)
        } else {
            display
        };
        println!(
            "{}",
            style(format!("  To start working:      cd {cd_arg}")).dim()
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::test_helpers::*;

    fn feed_wizard(
        wizard: &mut InitWizard,
        events: &[Event],
    ) -> anyhow::Result<FlowOutcome<WizardResult>> {
        for ev in events {
            match wizard.handle_event(ev)? {
                FlowOutcome::Done(r) => return Ok(FlowOutcome::Done(r)),
                FlowOutcome::Continue => {}
            }
        }
        Ok(FlowOutcome::Continue)
    }

    fn new_test_wizard() -> InitWizard {
        let mut w = InitWizard::new();
        w.skip_detection = true;
        w
    }

    #[test]
    fn wizard_full_flow() {
        let mut wizard = new_test_wizard();

        let mut events: Vec<Event> = Vec::new();
        // Step 0: type URL + Enter
        events.extend(type_string("https://github.com/user/repo.git"));
        events.push(enter());
        // Step 1: type name + Enter
        events.extend(type_string("myproject"));
        events.push(enter());
        // Step 2: type prefix + Enter
        events.extend(type_string("mp"));
        events.push(enter());
        // Step 3: type branch + Enter
        events.extend(type_string("develop"));
        events.push(enter());
        // Step 4: Right (to Yes) + Enter
        events.push(key_event(crossterm::event::KeyCode::Right));
        events.push(enter());

        let result = feed_wizard(&mut wizard, &events).unwrap();
        match result {
            FlowOutcome::Done(r) => {
                assert_eq!(r.url, "https://github.com/user/repo.git");
                assert_eq!(r.name, "myproject");
                assert_eq!(r.prefix, "mp");
                assert_eq!(r.branch, "develop");
            }
            FlowOutcome::Continue => panic!("expected Done"),
        }
    }

    #[test]
    fn wizard_default_branch_used() {
        let mut wizard = new_test_wizard();
        wizard.detected_branch = Some("master".to_string());

        let mut events: Vec<Event> = Vec::new();
        // Step 0: URL
        events.extend(type_string("https://example.com/repo"));
        events.push(enter());
        // Step 1: name (accept default)
        events.push(enter());
        // Step 2: prefix (accept default)
        events.push(enter());
        // Step 3: branch — submit empty to use default (detected "master")
        events.push(enter());
        // Step 4: confirm Yes
        events.push(key_char('y'));

        let result = feed_wizard(&mut wizard, &events).unwrap();
        match result {
            FlowOutcome::Done(r) => {
                assert_eq!(r.branch, "master");
            }
            FlowOutcome::Continue => panic!("expected Done"),
        }
    }

    #[test]
    fn wizard_esc_goes_back() {
        let mut wizard = new_test_wizard();

        let mut events: Vec<Event> = Vec::new();
        // Step 0: URL
        events.extend(type_string("https://example.com/repo"));
        events.push(enter());
        // Step 1: name
        events.extend(type_string("proj"));
        events.push(enter());
        // Now at step 2, press Esc to go back
        events.push(esc());

        feed_wizard(&mut wizard, &events).unwrap();
        assert_eq!(wizard.step, 1);
    }

    #[test]
    fn wizard_esc_at_step0_cancels() {
        let mut wizard = new_test_wizard();
        let result = feed_wizard(&mut wizard, &[esc()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cancelled"));
    }

    #[test]
    fn wizard_confirm_no_cancels() {
        let mut wizard = new_test_wizard();

        let mut events: Vec<Event> = Vec::new();
        events.extend(type_string("https://example.com/repo"));
        events.push(enter());
        events.push(enter()); // name default
        events.push(enter()); // prefix default
        events.push(enter()); // branch default
        // At confirm, press 'n'
        events.push(key_char('n'));

        let result = feed_wizard(&mut wizard, &events);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cancelled"));
    }

    #[test]
    fn wizard_renders_completed_fields() {
        let mut wizard = new_test_wizard();

        // Advance past step 0 (URL)
        let mut events: Vec<Event> = Vec::new();
        events.extend(type_string("https://example.com/repo"));
        events.push(enter());
        feed_wizard(&mut wizard, &events).unwrap();

        // Render and check that URL value appears
        let mut terminal = test_terminal(80, 24);
        terminal.draw(|frame| wizard.render(frame)).unwrap();
        let text = buffer_text(&terminal);
        assert!(
            text.contains("https://example.com/repo"),
            "expected URL in rendered output: {text}"
        );
    }
}
