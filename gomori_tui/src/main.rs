use std::io::{self, stdout};

use ratatui::{
    crossterm::{
        event::{self, Event, KeyCode},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    prelude::*,
    widgets::*,
};

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let player = HumanPlayer {};

    let mut should_quit = false;
    while !should_quit {
        terminal.draw(|frame| player.ui(frame))?;
        should_quit = handle_events()?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

struct HumanPlayer {}

const CARD_WIDTH: u16 = 6;

const HAND_CARDS_WIDGET_WIDTH: u16 = CARD_WIDTH * 5;

struct HandCardsWidget {}

impl Widget for HandCardsWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(HAND_CARDS_WIDGET_WIDTH),
                Constraint::Min(0),
            ])
            .split(area)[1];
        let coords = [
            (area.x, area.y),
            (area.x + CARD_WIDTH, area.y + 1),
            (area.x + 2 * CARD_WIDTH, area.y + 1),
            (area.x + 3 * CARD_WIDTH, area.y + 1),
            (area.x + 4 * CARD_WIDTH, area.y),
        ];
        for (x, y) in coords {
            let block = Block::new()
                .border_type(BorderType::Rounded)
                .borders(Borders::all());
            block.render(
                Rect {
                    x,
                    y,
                    width: CARD_WIDTH,
                    height: area.height - 1,
                },
                buf,
            );
            buf.set_string(x + 2, y + 1, "J♥", Style::new());
        }
    }
}

fn handle_events() -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(16))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

impl HumanPlayer {
    fn ui(&self, frame: &mut Frame) {
        let main_layout = Layout::new(
            Direction::Vertical,
            [Constraint::Min(0), Constraint::Length(6)],
        )
        .split(frame.size());
        frame.render_widget(HandCardsWidget {}, main_layout[1]);
    }
}
