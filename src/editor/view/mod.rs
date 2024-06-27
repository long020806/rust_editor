
use super::editorcommand::{Direction,EditorCommand};
use super::buffer::Buffer;
use super::terminal::{ Position, Size, Terminal};
const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
pub struct View {
    view_buffer: Buffer,
    needs_redraw: bool,
    size: Size,
    location:Position,
    offset: Position
}

impl View {
    pub fn default() -> View {
        View {
            view_buffer: Buffer::default(),
            needs_redraw: true,
            size: Size {
                width: Terminal::size().unwrap_or_default().width,
                height: Terminal::size().unwrap_or_default().height,
            },
            location:Position::default(),
            offset:Position::default()
        }
    }

    pub fn render(&mut self) {
        if !self.needs_redraw {
            return ;
        }
        let Size { height, width } = self.size;
        if height == 0 || width == 0 {
            return ;
        }
        let vertical_center = height / 3;
        let top = self.offset.y;
        for current_row in 0..height {
            if let Some(line) = self.view_buffer.lines.get(current_row.saturating_add(top) as usize) {
                let left = self.offset.x;
                let right = self.offset.x.saturating_add(width);
                Self::render_line(current_row as usize, &line.get(left as usize..right as usize));
            } else if current_row == vertical_center && self.view_buffer.is_empty() {
                Self::render_line(current_row as usize, &Self::build_welcome_message(width  as usize));
            } else {
                Self::render_line(current_row as usize, "~");
            }
            if current_row.saturating_add(1) < height {
                let _ = Terminal::print("\r\n");
            }
        }
        self.needs_redraw = false;

    }
    pub fn get_position(&self) -> Position {
        self.location.subtract(&self.offset).into()
    }

    fn move_text_location(&mut self, direction: &Direction) {
        let Position { mut x, mut y } = self.location;
        let Size { height, width } = self.size;
        match direction {
            Direction::Up => {
                y = y.saturating_sub(1);
            }
            Direction::Down => {
                y = y.saturating_add(1);
            }
            Direction::Left => {
                x = x.saturating_sub(1);
            }
            Direction::Right => {
                x = x.saturating_add(1);
            }
            Direction::PageUp => {
                y = 0;
            }
            Direction::PageDown => {
                y = height.saturating_sub(1);
            }
            Direction::Home => {
                x = 0;
            }
            Direction::End => {
                x = width.saturating_sub(1);
            }
        }
        self.location = Position { x, y };
        self.scroll_location_into_view();
    }


    fn scroll_location_into_view(&mut self) {
        let Position { x, y } = self.location;
        let Size { width, height } = self.size;
        let mut offset_changed = false;

        // Scroll vertically
        if y < self.offset.y {
            self.offset.y = y;
            offset_changed = true;
        } else if y >= self.offset.y.saturating_add(height) {
            // 如果 y >= offset.y + height offset.y = y -height + 1
            self.offset.y = y.saturating_sub(height).saturating_add(1);
            offset_changed = true;
        }

        //Scroll horizontally
        if x < self.offset.x {
            self.offset.x = x;
            offset_changed = true;
        } else if x >= self.offset.x.saturating_add(width) {
            self.offset.x = x.saturating_sub(width).saturating_add(1);
            offset_changed = true;
        }
        self.needs_redraw = offset_changed;
    }
    pub fn handle_command(&mut self, command: EditorCommand) {
        match command {
            EditorCommand::Resize(size) => self.resize(size),
            EditorCommand::Move(direction) => self.move_text_location(&direction),
            EditorCommand::Quit => {}
        }
    }
    fn render_line(at: usize, line_text: &str) {
        let result = Terminal::print_row(at, line_text);
        debug_assert!(result.is_ok(), "Failed to render line");
    }
    // fn render_line(at: usize, line_text: &str) -> Result<(), Error> {
    //     Terminal::move_cursor_to(Position { x: 0, y: at as u16 })?;
    //     Terminal::clear_line()?;
    //     Terminal::print(line_text)?;
    //     Ok(())
    // }

    fn build_welcome_message(width: usize) -> String {
        if width == 0 {
            return " ".to_string();
        }
        let welcome_message = format!("{NAME} editor -- version {VERSION}");
        let len = welcome_message.len();
        if width <= len {
            return "~".to_string();
        }
        // we allow this since we don't care if our welcome message is put _exactly_ in the middle.
        // it's allowed to be a bit to the left or right.
        #[allow(clippy::integer_division)]
        let padding = (width.saturating_sub(len).saturating_sub(1)) / 2;

        let mut full_message = format!("~{}{}", " ".repeat(padding), welcome_message);
        full_message.truncate(width);
        full_message
    }


    pub fn load(&mut self, file_name: &str) {
        if let Ok(buffer) = Buffer::load(file_name) {
            self.view_buffer = buffer;
            self.needs_redraw = true;
        }
    }

    pub fn resize(&mut self, to: Size) {
        self.size = to;
        self.scroll_location_into_view();
        self.needs_redraw = true;
    }
}
