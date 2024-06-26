use std::cmp::min;

use super::buffer::Buffer;
use super::terminal::{ Position, Size, Terminal};
const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
pub struct View {
    view_buffer: Buffer,
    needs_redraw: bool,
    size: Size,
    pub offset: Position
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
        for current_row in 0..height {
            if let Some(line) = self.view_buffer.lines.get(current_row.saturating_add(self.offset.y) as usize) {
                
                let truncated_line = &line[min(self.offset.x as usize,line.len())..min(line.len(),self.offset.x.saturating_add(width) as usize)];
                Self::render_line(current_row as usize, truncated_line);
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

    pub fn set_render(&mut self, should_render: bool) {
        self.needs_redraw = should_render;
    }

    pub fn resize(&mut self, to: Size) {
        self.size = to;
        self.needs_redraw = true;
    }
}
