use crate::execute::diagnostics::ResultExt;
use crate::execute::process_file::workspace_file::WorkspaceFile;
use crate::execute::process_file::{FileResult, FileStatus, Message, SharedTraversalOptions};
use crate::CliDiagnostic;
use biome_console::fmt::Formatter;
use biome_diagnostics::serde::Diagnostic;
use biome_diagnostics::{category, DiagnosticExt, Error, PrintDiagnostic};
use biome_service::workspace::RuleCategories;
use crossterm::event::{KeyCode, KeyEventKind};
use crossterm::{
    event,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::{prelude::*, widgets::*};
use std::io::{stdout, Result};
use std::path::Path;
use std::sync::atomic::Ordering;

/// Lints a single file and returns a [FileResult]
pub(crate) fn lint<'ctx>(ctx: &'ctx SharedTraversalOptions<'ctx, '_>, path: &Path) -> FileResult {
    let mut workspace_file = WorkspaceFile::new(ctx, path)?;
    lint_with_guard(ctx, &mut workspace_file)
}

pub(crate) fn lint_with_guard<'ctx>(
    ctx: &'ctx SharedTraversalOptions<'ctx, '_>,
    workspace_file: &mut WorkspaceFile,
) -> FileResult {
    tracing::info_span!("Processes linting", path =? workspace_file.path.display()).in_scope(
        move || {
            let mut errors = 0;
            let mut input = workspace_file.input()?;

            if let Some(fix_mode) = ctx.execution.as_fix_file_mode() {
                if ctx.execution.interactive() {
                    let result = workspace_file
                        .guard()
                        .pull_diagnostics(RuleCategories::LINT, u64::MAX, true)
                        .with_file_path_and_code(
                            workspace_file.path.display().to_string(),
                            category!("lint"),
                        )?;

                    choose_action(result.diagnostics).with_file_path_and_code(
                        workspace_file.path.display().to_string(),
                        category!("lint"),
                    )?;
                } else {
                    let fixed = workspace_file
                        .guard()
                        .fix_file(*fix_mode, false)
                        .with_file_path_and_code(
                            workspace_file.path.display().to_string(),
                            category!("lint"),
                        )?;

                    ctx.push_message(Message::SkippedFixes {
                        skipped_suggested_fixes: fixed.skipped_suggested_fixes,
                    });

                    if fixed.code != input {
                        workspace_file.update_file(fixed.code)?;
                        input = workspace_file.input()?;
                    }
                    errors = fixed.errors;
                }
            }

            let max_diagnostics = ctx.remaining_diagnostics.load(Ordering::Relaxed);
            let pull_diagnostics_result = workspace_file
                .guard()
                .pull_diagnostics(RuleCategories::LINT, max_diagnostics.into(), false)
                .with_file_path_and_code(
                    workspace_file.path.display().to_string(),
                    category!("lint"),
                )?;

            let no_diagnostics = pull_diagnostics_result.diagnostics.is_empty()
                && pull_diagnostics_result.skipped_diagnostics == 0;
            errors += pull_diagnostics_result.errors;

            if !no_diagnostics {
                ctx.push_message(Message::Diagnostics {
                    name: workspace_file.path.display().to_string(),
                    content: input,
                    diagnostics: pull_diagnostics_result
                        .diagnostics
                        .into_iter()
                        .map(Error::from)
                        .collect(),
                    skipped_diagnostics: pull_diagnostics_result.skipped_diagnostics,
                });
            }

            if errors > 0 {
                if ctx.execution.is_check_apply() || ctx.execution.is_check_apply_unsafe() {
                    Ok(FileStatus::Message(Message::ApplyError(
                        CliDiagnostic::file_check_apply_error(
                            workspace_file.path.display().to_string(),
                            category!("lint"),
                        ),
                    )))
                } else {
                    Ok(FileStatus::Message(Message::ApplyError(
                        CliDiagnostic::file_check_error(
                            workspace_file.path.display().to_string(),
                            category!("lint"),
                        ),
                    )))
                }
            } else {
                Ok(FileStatus::Success)
            }
        },
    )
}

fn choose_action(diagnostics: Vec<Diagnostic>) -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let total = diagnostics.len();
    let current_count = 1;
    let mut diagnostics = diagnostics.iter();
    'actions: while let Some(diagnostics) = diagnostics.next() {
        loop {
            terminal.draw(|frame| {
                let outer_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![Constraint::Length(3), Constraint::Percentage(70)])
                    .split(frame.size());

                let inner_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![Constraint::Percentage(70), Constraint::Percentage(30)])
                    .split(outer_layout[1]);

                frame.render_widget(
                    Paragraph::new("Review diagnostics")
                        .block(Block::new().title("Biome interactive mode")),
                    outer_layout[0],
                );

                frame.render_widget(
                    Paragraph::new("Bottom").block(Block::new().borders(Borders::TOP)),
                    inner_layout[0],
                );
                let mut lines = vec![];
                let l: Line = ratatui::text::Line::from("str".into());
                frame.render_widget(
                    Paragraph::new("Bottom").block(Block::new().borders(Borders::TOP)),
                    inner_layout[1],
                );
            })?;

            if event::poll(std::time::Duration::from_millis(16))? {
                if let event::Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                        break 'actions;
                    }
                }
            }
        }
    }
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

pub fn diagnostic_to_string(name: &str, source: &str, diag: Error) -> String {
    let error = diag.with_file_path(name).with_file_source_code(source);
    let text = markup_to_string(biome_console::markup! {
        {PrintDiagnostic::verbose(&error)}
    });

    text
}

fn markup_to_string(markup: biome_console::Markup) -> String {
    let mut buffer = Vec::new();
    let mut write =
        biome_console::fmt::Termcolor(biome_diagnostics::termcolor::NoColor::new(&mut buffer));
    let mut fmt = Formatter::new(&mut write);
    fmt.write_markup(markup).unwrap();

    String::from_utf8(buffer).unwrap()
}
