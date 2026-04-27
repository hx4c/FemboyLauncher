use std::{
    io::{self, stdout},
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

struct App {
    instance_focus: usize,
    action_row_focus: usize,
    action_col_focus: usize,
    instances: Vec<(&'static str, &'static str)>,
    action_rows: Vec<Vec<&'static str>>,
}

fn main() -> io::Result<()> {
    let mut app = App {
        instance_focus: 0,
        action_row_focus: 0,
        action_col_focus: 0,
        instances: vec![
            ("Prism Vanilla 1.20.4", "2h 40m"),
            ("Prism Fabric 1.21.0", "0h 00m"),
            ("Prism Neoforge 1.21", "11h 18m"),
            ("Nightly Tech Pack", "4h 02m"),
        ],
        action_rows: vec![
            vec!["Launch", "Kill"],
            vec!["Edit", "Folder", "Export"],
            vec!["Copy", "Delete"],
        ],
    };

    let mut terminal = setup_terminal()?;

    loop {
        terminal.draw(|frame| ui(frame, &app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Up => {
                        if app.instance_focus > 0 {
                            app.instance_focus -= 1;
                        } else {
                            app.instance_focus = app.instances.len() - 1;
                        }
                    }
                    KeyCode::Down => {
                        app.instance_focus = (app.instance_focus + 1) % app.instances.len();
                    }
                    KeyCode::Tab => {
                        advance_action_focus(&mut app, 1);
                    }
                    KeyCode::BackTab => {
                        advance_action_focus(&mut app, -1);
                    }
                    KeyCode::Char('h') => {
                        let row_len = app.action_rows[app.action_row_focus].len();
                        if app.action_col_focus > 0 {
                            app.action_col_focus -= 1;
                        } else {
                            app.action_col_focus = row_len.saturating_sub(1);
                        }
                    }
                    KeyCode::Char('l') => {
                        let row_len = app.action_rows[app.action_row_focus].len();
                        app.action_col_focus = (app.action_col_focus + 1) % row_len;
                    }
                    _ => {}
                }
            }
        }
    }

    restore_terminal(terminal)?;
    Ok(())
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(out);
    Ok(Terminal::new(backend)?)
}

fn restore_terminal(mut terminal: Terminal<CrosstermBackend<std::io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn ui(frame: &mut Frame, app: &App) {
    let content = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(42)].as_ref())
        .split(frame.area());

    render_groups_instances(frame, content[0], app);
    render_right_bar(frame, content[1], app);
}

fn render_groups_instances(frame: &mut Frame, area: Rect, app: &App) {
    let mut instance_rows = Vec::new();
    for (i, (name, playtime)) in app.instances.iter().enumerate() {
        let style = if i == app.instance_focus {
            Style::default().bg(Color::Rgb(50, 80, 130)).fg(Color::White).bold()
        } else {
            Style::default()
        };
        let marker = if i == app.instance_focus { "▸" } else { " " };
        instance_rows.push(Row::new(vec![
            Cell::from(format!("{} {}", marker, name)),
            Cell::from(*playtime),
        ]).style(style));
    }
    let inst_table = Table::new(
        instance_rows,
        [
            Constraint::Percentage(55),
            Constraint::Percentage(45),
        ],
    )
    .header(
        Row::new(vec![Cell::from("Instance"), Cell::from("Playtime")])
            .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
    )
    .column_spacing(1);
    frame.render_widget(
        inst_table.block(Block::default().borders(Borders::ALL).title("Instances")),
        area,
    );
}

fn render_right_bar(frame: &mut Frame, area: Rect, app: &App) {
    let (action_area, playtime_area) = {
        let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
        .split(area);
        (outer[0], outer[1])
    };

    let action_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(4), Constraint::Length(3)].as_ref())
        .split(action_area);

    for row in 0..3 {
        render_action_row(frame, action_rows[row], app, row);
    }

    let playtime = Paragraph::new(format!(
        "Playtime: {}",
        app.instances
            .get(app.instance_focus)
            .map_or("0h 00m", |(_, p)| *p)
    ))
    .alignment(Alignment::Left)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Playtime"),
    );
    frame.render_widget(playtime, playtime_area);
}

fn render_action_row(frame: &mut Frame, area: Rect, app: &App, row: usize) {
    if app.action_rows[row].is_empty() {
        return;
    }
    let widths = vec![Constraint::Ratio(1, app.action_rows[row].len() as u32); app.action_rows[row].len()];
    let row_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(widths)
        .split(area);
    for (col, label) in app.action_rows[row].iter().enumerate() {
        let style = if app.action_row_focus == row && app.action_col_focus == col {
            Style::default().fg(Color::Black).bg(Color::Rgb(80, 180, 255)).bold()
        } else {
            Style::default().fg(Color::White)
        };
        let button = Paragraph::new(*label)
            .alignment(Alignment::Center)
            .style(style)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(button, row_layout[col]);
    }
}

fn flatten_action_len(app: &App) -> usize {
    app.action_rows.iter().map(|row| row.len()).sum()
}

fn current_action_index(app: &App) -> usize {
    let mut idx = app.action_col_focus;
    for row in 0..app.action_row_focus {
        idx += app.action_rows[row].len();
    }
    idx
}

fn set_action_index(app: &mut App, index: usize) {
    let mut remaining = index;
    for row in 0..app.action_rows.len() {
        let len = app.action_rows[row].len();
        if remaining < len {
            app.action_row_focus = row;
            app.action_col_focus = remaining;
            return;
        }
        remaining -= len;
    }
    app.action_row_focus = 0;
    app.action_col_focus = 0;
}

fn advance_action_focus(app: &mut App, delta: isize) {
    let total = flatten_action_len(app);
    if total == 0 {
        return;
    }
    let current = current_action_index(app);
    let next = ((current as isize + delta).rem_euclid(total as isize)) as usize;
    set_action_index(app, next);
}
