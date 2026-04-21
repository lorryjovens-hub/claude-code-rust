//! Terminal Module - Terminal UI with Ratatui

use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    style::{Color, Style},
};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Stdout};

/// Terminal UI application
pub struct TerminalApp {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    should_quit: bool,
}

impl TerminalApp {
    /// Create a new terminal app
    pub fn new() -> anyhow::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        
        Ok(Self {
            terminal,
            should_quit: false,
        })
    }
    
    /// Run the terminal app
    pub fn run(&mut self) -> anyhow::Result<()> {
        while !self.should_quit {
            self.draw()?;
            self.handle_events()?;
        }
        
        Ok(())
    }
    
    /// Draw the UI
    fn draw(&mut self) -> anyhow::Result<()> {
        self.terminal.draw(|f| {
            use ratatui::{layout::*, widgets::*};
            
            // Create layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(1),
                    Constraint::Length(3),
                ])
                .split(f.size());
            
            // Header
            let header = Paragraph::new("🟢 Claude Code Rust - 重构高性能版本")
                .style(Style::default().fg(Color::Green))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);
            
            // Main content area
            let main = Paragraph::new("Welcome! Type your message below.")
                .style(Style::default())
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(main, chunks[1]);
            
            // Input area
            let input = Paragraph::new("Input: ")
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(input, chunks[2]);
        })?;
        
        Ok(())
    }
    
    /// Handle terminal events
    fn handle_events(&mut self) -> anyhow::Result<()> {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            self.should_quit = true;
                        }
                        KeyCode::Char('q') => {
                            self.should_quit = true;
                        }
                        _ => {}
                    }
                }
            }
        }
        
        Ok(())
    }
}

impl Drop for TerminalApp {
    fn drop(&mut self) {
        // Restore terminal
        disable_raw_mode().ok();
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen
        ).ok();
        self.terminal.show_cursor().ok();
    }
}