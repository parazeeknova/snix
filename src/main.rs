use color_eyre::eyre::{Ok, Result};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::{
        event::{self, Event},
        style::Color,
    },
    layout::{Constraint, Layout},
    style::Stylize,
    widgets::{Block, List, ListItem, Paragraph, Widget},
};

#[derive(Debug, Default)]
struct AppState {
    items: Vec<TodoItem>,
}

#[derive(Debug, Default)]
struct TodoItem {
    is_done: bool,
    description: String,
}

fn main() -> Result<()> {
    println!("Starting rustyui by parazeeknova!");

    let mut state = AppState::default();

    // Demo states for debug
    state.items.push(TodoItem {
        is_done: false,
        description: String::from("Debug"),
    });

    color_eyre::install()?;

    let terminal: ratatui::Terminal<ratatui::prelude::CrosstermBackend<std::io::Stdout>> =
        ratatui::init();
    let result: std::result::Result<(), color_eyre::eyre::Error> = run(terminal, &mut state);
    ratatui::restore();

    result
}

fn run(mut terminal: DefaultTerminal, app_state: &mut AppState) -> Result<()> {
    loop {
        // Rendering
        terminal.draw(|f| render(f, app_state))?;

        // Input Handling
        if let Event::Key(key) = event::read()? {
            match key.code {
                event::KeyCode::Esc => {
                    break;
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn render(frame: &mut Frame, app_state: &AppState) {
    // Paragraph::new("Starting RustyUI 0.1").render(frame.area(), frame.buffer_mut());

    let [border_area] = Layout::vertical([Constraint::Fill(1)])
        .margin(1)
        .areas(frame.area());

    let [inner_area] = Layout::vertical([Constraint::Fill(1)])
        .margin(1)
        .areas(border_area);

    Block::bordered()
        .border_type(ratatui::widgets::BorderType::Rounded)
        .fg(Color::Yellow)
        .render(border_area, frame.buffer_mut());

    List::new(
        app_state
            .items
            .iter()
            .map(|x| ListItem::from(x.description.clone())),
    )
    .render(inner_area, frame.buffer_mut());
}
