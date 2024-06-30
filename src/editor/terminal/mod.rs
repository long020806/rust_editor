use crossterm::style::Attribute;
use crossterm::terminal::DisableLineWrap;
use crossterm::terminal::EnableLineWrap;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::SetTitle;
use crossterm::Command;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use std::io::stdout;
use std::io::Write;
#[derive(Copy, Clone, Default,Debug)]
pub struct Size {
    pub height: u16,
    pub width: u16,
}

#[derive(Copy, Clone,Default)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Position {
    pub const fn subtract(&self, other: &Self) -> Self {
        Self {
            x: self.x.saturating_sub(other.x),
            y: self.y.saturating_sub(other.y),
        }
    }
}

pub struct Terminal {}

impl Terminal {
    pub fn terminate() -> Result<(), std::io::Error> {
        Self::execute()?;
        Self::enable_line_wrap()?;
        Self::leave_alternate_screen()?;
        disable_raw_mode()?;
        Self::show_cursor()?;
        Ok(())
    }

    pub fn disable_line_wrap() -> Result<(), std::io::Error> {
        Self::excute_command(DisableLineWrap)?;
        Ok(())
    }
    pub fn enable_line_wrap() -> Result<(), std::io::Error> {
        Self::excute_command(EnableLineWrap)?;
        Ok(())
    }
    pub fn set_title(title: &str) -> Result<(), std::io::Error> {
        Self::excute_command(SetTitle(title))?;
        Ok(())
    }

    pub fn print_inverted_row(row: usize, line_text: &str) -> Result<(), std::io::Error> {
        let width = Self::size()?.width as usize;
        Self::print_row(
            row,
            &format!(
                "{}{:width$.width$}{}",
                Attribute::Reverse,
                line_text,
                Attribute::Reset
            ),
        )
    }

    pub fn leave_alternate_screen() -> Result<(), std::io::Error> {
        Self::excute_command(LeaveAlternateScreen)?;
        Ok(())
    }
    pub fn enter_alternate_screen() -> Result<(), std::io::Error> {
        Self::excute_command(EnterAlternateScreen)?;
        Ok(())
    }

    pub fn initialize() -> Result<(), std::io::Error> {
        enable_raw_mode()?;
        Self::enter_alternate_screen()?;
        Self::disable_line_wrap()?;
        Self::clear_screen()?;
        Self::move_cursor_to(Position { x: 0, y: 0 })?;
        Self::execute()?;
        Ok(())
    }

    pub fn clear_line() -> Result<(), std::io::Error> {
        Self::excute_command(Clear(ClearType::CurrentLine))?;
        Ok(())
    }

    pub fn clear_screen() -> Result<(), std::io::Error> {
        Self::excute_command(Clear(ClearType::All))?;
        Ok(())
    }

    pub fn move_cursor_to(position: Position) -> Result<(), std::io::Error> {
        Self::excute_command(MoveTo(position.x, position.y))?;
        Ok(())
    }

    pub fn size() -> Result<Size, std::io::Error> {
        let (size_x, size_y) = crossterm::terminal::size()?;
        Ok(Size {
            height: size_y,
            width: size_x,
        })
    }

    pub fn hide_cursor() -> Result<(), std::io::Error> {
        Self::excute_command(Hide)?;
        Ok(())
    }

    pub fn show_cursor() -> Result<(), std::io::Error> {
        Self::excute_command(Show)?;
        Ok(())
    }

    pub fn print(string: &str) -> Result<(), std::io::Error> {
        Self::excute_command(Print(string))?;
        Ok(())
    }

    pub fn execute() -> Result<(), std::io::Error> {
        stdout().flush()?;
        Ok(())
    }

    pub fn excute_command<T: Command>(command: T) -> Result<(), std::io::Error> {
        execute!(stdout(), command)?;
        Ok(())
    }

    pub fn print_row(at: usize, line_text: &str) -> Result<(), std::io::Error> {
        Self::move_cursor_to(Position { x: 0, y: at as u16 })?;
        Self::clear_line()?;
        Self::print(line_text)?;
        Ok(())
    }
}
