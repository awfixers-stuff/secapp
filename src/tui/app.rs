use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap};
use ratatui::Frame;
use std::io;
use std::sync::Arc;

use crate::config::Config;
use crate::daemon::Daemon;

/// Which tab is currently active.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Tab {
    Directories,
    Logs,
    Policies,
    Status,
}

impl Tab {
    const ALL: [Tab; 4] = [Tab::Directories, Tab::Logs, Tab::Policies, Tab::Status];

    fn title(self) -> &'static str {
        match self {
            Tab::Directories => "Directories",
            Tab::Logs => "Access Logs",
            Tab::Policies => "Policies",
            Tab::Status => "Status",
        }
    }

    fn index(self) -> usize {
        match self {
            Tab::Directories => 0,
            Tab::Logs => 1,
            Tab::Policies => 2,
            Tab::Status => 3,
        }
    }
}

/// The TUI application state.
pub struct App {
    tab: Tab,
    config: Arc<Config>,
    daemon: Arc<Daemon>,
    /// Whether the user is in password input mode.
    password_mode: bool,
    /// Current password input (masked).
    password_input: String,
    /// Status message shown at the bottom.
    status_message: String,
    /// List state for the directories tab.
    dir_list_state: ListState,
    /// List state for the logs tab.
    log_list_state: ListState,
    /// List state for the policies tab.
    policy_list_state: ListState,
    /// Cached protected files.
    protected_files: Vec<String>,
    /// Cached access logs.
    access_logs: Vec<crate::db::AccessLog>,
    /// Cached policies.
    policies: Vec<crate::db::Policy>,
    /// Whether the keystore is unlocked.
    unlocked: bool,
}

impl App {
    pub fn new(config: Config, daemon: Daemon) -> Self {
        Self {
            tab: Tab::Directories,
            config: Arc::new(config),
            daemon: Arc::new(daemon),
            password_mode: false,
            password_input: String::new(),
            status_message: String::new(),
            dir_list_state: ListState::default(),
            log_list_state: ListState::default(),
            policy_list_state: ListState::default(),
            protected_files: Vec::new(),
            access_logs: Vec::new(),
            policies: Vec::new(),
            unlocked: false,
        }
    }

    /// Refresh data from the database.
    async fn refresh_data(&mut self) {
        let db = self.daemon.db();
        if let Ok(files) = db.list_protected_files().await {
            self.protected_files = files.iter().map(|f| {
                let status = if f.is_encrypted { "🔒" } else { "🔓" };
                format!("{} {} ({} bytes)", status, f.path, f.original_size)
            }).collect();
        }
        if let Ok(logs) = db.list_access_logs(100).await {
            self.access_logs = logs;
        }
        if let Ok(policies) = db.list_policies().await {
            self.policies = policies;
        }
        self.unlocked = self.daemon.is_unlocked().await;
    }

    /// Handle a key event. Returns true if the app should quit.
    async fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        // Password input mode takes priority
        if self.password_mode {
            return self.handle_password_key(key).await;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
            KeyCode::Tab => {
                let idx = self.tab.index();
                let next = Tab::ALL[(idx + 1) % Tab::ALL.len()];
                self.tab = next;
            }
            KeyCode::BackTab => {
                let idx = self.tab.index();
                let prev = Tab::ALL[(idx + Tab::ALL.len() - 1) % Tab::ALL.len()];
                self.tab = prev;
            }
            KeyCode::Char('u') => {
                // Enter password mode for unlock
                self.password_mode = true;
                self.password_input.clear();
                self.status_message = "Enter master password:".to_string();
            }
            KeyCode::Char('l') => {
                // Lock the keystore
                self.daemon.lock().await?;
                self.unlocked = false;
                self.status_message = "Keystore locked".to_string();
            }
            KeyCode::Char('r') => {
                // Refresh data
                self.refresh_data().await;
                self.status_message = "Data refreshed".to_string();
            }
            KeyCode::Down => self.select_next(),
            KeyCode::Up => self.select_previous(),
            _ => {}
        }
        Ok(false)
    }

    /// Handle key events in password input mode.
    async fn handle_password_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Enter => {
                let password = std::mem::take(&mut self.password_input);
                self.password_mode = false;
                match self.daemon.unlock(&password).await {
                    Ok(()) => {
                        self.unlocked = true;
                        self.status_message = "Keystore unlocked successfully".to_string();
                        self.refresh_data().await;
                    }
                    Err(e) => {
                        self.status_message = format!("Unlock failed: {}", e);
                    }
                }
            }
            KeyCode::Esc => {
                self.password_mode = false;
                self.password_input.clear();
                self.status_message = "Password entry cancelled".to_string();
            }
            KeyCode::Backspace => {
                self.password_input.pop();
            }
            KeyCode::Char(c) => {
                self.password_input.push(c);
            }
            _ => {}
        }
        Ok(false)
    }

    fn select_next(&mut self) {
        let state = match self.tab {
            Tab::Directories => &mut self.dir_list_state,
            Tab::Logs => &mut self.log_list_state,
            Tab::Policies => &mut self.policy_list_state,
            Tab::Status => return,
        };
        let len = match self.tab {
            Tab::Directories => self.protected_files.len(),
            Tab::Logs => self.access_logs.len(),
            Tab::Policies => self.policies.len(),
            Tab::Status => 0,
        };
        if len > 0 {
            state.select(Some((state.selected().unwrap_or(0) + 1).min(len - 1)));
        }
    }

    fn select_previous(&mut self) {
        let state = match self.tab {
            Tab::Directories => &mut self.dir_list_state,
            Tab::Logs => &mut self.log_list_state,
            Tab::Policies => &mut self.policy_list_state,
            Tab::Status => return,
        };
        let len = match self.tab {
            Tab::Directories => self.protected_files.len(),
            Tab::Logs => self.access_logs.len(),
            Tab::Policies => self.policies.len(),
            Tab::Status => 0,
        };
        if len > 0 {
            let current = state.selected().unwrap_or(0);
            state.select(Some(current.saturating_sub(1)));
        }
    }

    /// Draw the TUI.
    fn draw(&mut self, f: &mut Frame) {
        let size = f.area();

        // Create main layout: title, content, status bar
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // title + tabs
                Constraint::Min(0),      // content
                Constraint::Length(2),    // status bar
            ])
            .split(size);

        // Draw title + tabs
        let tab_titles: Vec<Line> = Tab::ALL
            .iter()
            .map(|t| {
                let style = if *t == self.tab {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                Line::from(Span::styled(t.title(), style))
            })
            .collect();

        let tabs = Tabs::new(tab_titles)
            .block(Block::default().borders(Borders::BOTTOM).title("secapp"))
            .select(self.tab.index());
        f.render_widget(tabs, chunks[0]);

        // Draw content based on active tab
        match self.tab {
            Tab::Directories => self.draw_directories(f, chunks[1]),
            Tab::Logs => self.draw_logs(f, chunks[1]),
            Tab::Policies => self.draw_policies(f, chunks[1]),
            Tab::Status => self.draw_status(f, chunks[1]),
        }

        // Draw status bar
        let lock_icon = if self.unlocked { "🔓" } else { "🔒" };
        let status = if self.password_mode {
            format!("{} {}", self.status_message, "*".repeat(self.password_input.len()))
        } else {
            format!("{} secapp | [u]nlock [l]ock [r]efresh [q]uit | {}", lock_icon, self.status_message)
        };
        let status_bar = Paragraph::new(status)
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(status_bar, chunks[2]);
    }

    fn draw_directories(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.protected_files
            .iter()
            .map(|f| ListItem::new(f.clone()))
            .collect();
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Protected Files"))
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));
        f.render_stateful_widget(list, area, &mut self.dir_list_state);
    }

    fn draw_logs(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.access_logs
            .iter()
            .map(|log| {
                let icon = if log.allowed { "✓" } else { "✗" };
                ListItem::new(format!(
                    "{} [{}] {} by {} ({}) - {}",
                    icon, log.timestamp, log.action, log.process_name, log.process_pid, log.file_path
                ))
            })
            .collect();
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Access Logs"))
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));
        f.render_stateful_widget(list, area, &mut self.log_list_state);
    }

    fn draw_policies(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.policies
            .iter()
            .map(|p| {
                let style = match p.action.as_str() {
                    "allow" => Style::default().fg(Color::Green),
                    "deny" => Style::default().fg(Color::Red),
                    "log" => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                };
                ListItem::new(Span::styled(
                    format!("[{}] {} on {} from {} (p={})", p.action, p.action, p.path_pattern, p.process_pattern, p.priority),
                    style,
                ))
            })
            .collect();
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Access Policies"))
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));
        f.render_stateful_widget(list, area, &mut self.policy_list_state);
    }

    fn draw_status(&self, f: &mut Frame, area: Rect) {
        let lines = vec![
            Line::from(format!(
                "Keystore status: {}",
                if self.unlocked { "UNLOCKED" } else { "LOCKED" }
            )),
            Line::from(format!("Protected directories: {}", self.config.protected_paths.len())),
            Line::from(format!("Tracked files: {}", self.protected_files.len())),
            Line::from(format!("Access policies: {}", self.policies.len())),
            Line::from(format!("Algorithm: {:?}", self.config.encryption.algorithm)),
            Line::from(""),
            Line::from("Keybindings:"),
            Line::from("  [u] Unlock keystore"),
            Line::from("  [l] Lock keystore"),
            Line::from("  [r] Refresh data"),
            Line::from("  [Tab] Switch tabs"),
            Line::from("  [q/Esc] Quit"),
        ];
        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Status"))
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
    }
}

/// Run the TUI application.
pub async fn run_tui(config: Config, daemon: Daemon) -> Result<()> {
    // Set up terminal
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen, crossterm::cursor::Hide)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = App::new(config, daemon);
    app.refresh_data().await;

    // Main event loop
    let result = loop {
        terminal.draw(|f| app.draw(f))?;

        // Poll for events with timeout
        if event::poll(std::time::Duration::from_millis(100))? {
            if let CrosstermEvent::Key(key) = event::read()? {
                if app.handle_key(key).await? {
                    break Ok(());
                }
            }
        }
    };

    // Restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        io::stdout(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::cursor::Show
    )?;

    result
}