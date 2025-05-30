use color_eyre::eyre::{Ok, Result};
use ratatui::{crossterm::event::{self, Event}, widgets::{Paragraph, Widget}, DefaultTerminal, Frame};

fn main() -> Result<()> {
    println!("Starting rustyui by parazeeknova!");
    color_eyre::install()?;

    let terminal: ratatui::Terminal<ratatui::prelude::CrosstermBackend<std::io::Stdout>> = ratatui::init();
    let result: std::result::Result<(), color_eyre::eyre::Error> = run(terminal);
    ratatui::restore();

    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    loop {
        // Rendering
        terminal.draw(render)?;

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

fn render(frame:&mut Frame) {
    Paragraph::new("Starting RustyUI 0.1").render(frame.area(), frame.buffer_mut());
}