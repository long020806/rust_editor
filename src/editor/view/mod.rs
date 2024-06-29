use super::buffer::Buffer;
use super::command::{Edit, Move};
use super::documentstatus::DocumentStatus;
use super::line::Line;
use super::terminal::{Position, Size, Terminal};
use super::uicomponent::UIComponent;
use std::char;
use std::cmp::min;
const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
#[derive(Default)]
pub struct View {
    view_buffer: Buffer,
    needs_redraw: bool,
    size: Size,
    location: Position,
    offset: Position,
}

impl View {
    pub fn get_status(&self) -> DocumentStatus {
        DocumentStatus {
            total_lines: self.view_buffer.height(),
            current_line_index: self.location.y as usize,
            file_name: format!("{}", self.view_buffer.file_info),
            is_modified: self.view_buffer.dirty,
        }
    }

    fn scroll_text_location_into_view(&mut self) {
        let Position { y, x } = self.text_location_to_position();
        self.scroll_vertically(y);
        self.scroll_horizontally(x);
    }
    fn scroll_vertically(&mut self, row: u16) {
        let Size { height, .. } = self.size;
        let mut offset_changed = false;

        // Scroll vertically
        if row < self.offset.y {
            self.offset.y = row;
            offset_changed = true;
        } else if row >= self.offset.y.saturating_add(height) {
            // 如果 y >= offset.y + height offset.y = y -height + 1
            self.offset.y = row.saturating_sub(height).saturating_add(1);
            offset_changed = true;
        }

        self.needs_redraw = self.needs_redraw || offset_changed;
    }

    fn scroll_horizontally(&mut self, col: u16) {
        let Size { width, .. } = self.size;
        let mut offset_changed = false;

        //Scroll horizontally
        if col < self.offset.x {
            self.offset.x = col;
            offset_changed = true;
        } else if col >= self.offset.x.saturating_add(width) {
            self.offset.x = col.saturating_sub(width).saturating_add(1);
            offset_changed = true;
        }

        self.needs_redraw = self.needs_redraw || offset_changed;
    }

    // fn scroll_location_into_view(&mut self) {
    //     let Position { x, y } = self.location;
    //     self.scroll_vertically(x);
    //     self.scroll_horizontally(y);
    // }
    pub fn caret_position(&self) -> Position {
        self.text_location_to_position().subtract(&self.offset)
    }

    fn text_location_to_position(&self) -> Position {
        let row = self.location.y;
        let col = self
            .view_buffer
            .lines
            .get(row as usize)
            .map_or(0, |line| line.width_until_u16(self.location.x as usize));
        Position { x: col, y: row }
    }

    fn move_up(&mut self, step: usize) {
        self.location.y = self.location.y.saturating_sub(step as u16);
        self.snap_to_valid_grapheme();
    }
    fn move_down(&mut self, step: usize) {
        self.location.y = self.location.y.saturating_add(step as u16);
        self.snap_to_valid_grapheme();
        self.snap_to_valid_line();
    }
    // clippy::arithmetic_side_effects: This function performs arithmetic calculations
    // after explicitly checking that the target value will be within bounds.
    #[allow(clippy::arithmetic_side_effects)]
    fn move_right(&mut self) {
        let line_width = self
            .view_buffer
            .lines
            .get(self.location.y as usize)
            .map_or(0, Line::grapheme_count_u16);
        if self.location.x < line_width {
            self.location.x += 1;
        } else {
            self.move_to_start_of_line();
            self.move_down(1);
        }
    }
    // clippy::arithmetic_side_effects: This function performs arithmetic calculations
    // after explicitly checking that the target value will be within bounds.
    #[allow(clippy::arithmetic_side_effects)]
    fn move_left(&mut self) {
        if self.location.x > 0 {
            self.location.x -= 1;
        } else if self.location.y > 0 {
            self.move_up(1);
            self.move_to_end_of_line();
        }
    }

    // Ensures self.location.grapheme_index points to a valid grapheme index by snapping it to the left most grapheme if appropriate.
    // Doesn't trigger scrolling.
    fn snap_to_valid_grapheme(&mut self) {
        self.location.x = self
            .view_buffer
            .lines
            .get(self.location.y as usize)
            .map_or(0, |line| min(line.grapheme_count_u16(), self.location.x));
    }
    // Ensures self.location.line_index points to a valid line index by snapping it to the bottom most line if appropriate.
    // Doesn't trigger scrolling.
    fn snap_to_valid_line(&mut self) {
        self.location.y = min(self.location.y, self.view_buffer.height_u16());
    }

    fn move_to_start_of_line(&mut self) {
        self.location.x = 0;
    }
    fn move_to_end_of_line(&mut self) {
        self.location.x = self
            .view_buffer
            .lines
            .get(self.location.y as usize)
            .map_or(0, Line::grapheme_count_u16);
    }

    pub fn save(&mut self) -> Result<(), std::io::Error> {
        self.view_buffer.save()?;
        Ok(())
    }

    // region: command handling
    pub fn handle_edit_command(&mut self, command: Edit) {
        match command {
            Edit::Insert(character) => self.insert_char(character),
            Edit::Delete => self.delete(),
            Edit::DeleteBackward => self.delete(),
            Edit::InsertNewline => self.insert_newline(),
        }
    }
    pub fn handle_move_command(&mut self, command: Move) {
        let Size { height, .. } = self.size;
        // This match moves the positon, but does not check for all boundaries.
        // The final boundarline checking happens after the match statement.
        match command {
            Move::Up => self.move_up(1),
            Move::Down => self.move_down(1),
            Move::Left => self.move_left(),
            Move::Right => self.move_right(),
            Move::PageUp => self.move_up(height.saturating_sub(1) as usize),
            Move::PageDown => self.move_down(height.saturating_sub(1) as usize),
            Move::StartOfLine => self.move_to_start_of_line(),
            Move::EndOfLine => self.move_to_end_of_line(),
        }
        self.scroll_text_location_into_view();
    }

    pub fn insert_char(&mut self, character: char) {
        let old_len = self
            .view_buffer
            .lines
            .get(self.location.y as usize)
            .map_or(0, Line::grapheme_count);
        self.view_buffer.insert_char(character, self.location);
        let new_len = self
            .view_buffer
            .lines
            .get(self.location.y as usize)
            .map_or(0, Line::grapheme_count);
        let grapheme_delta = new_len.saturating_sub(old_len);
        if grapheme_delta > 0 {
            //move right for an added grapheme (should be the regular case)
            self.handle_move_command(Move::Right);
        }
        self.mark_redraw(true);
    }

    pub fn delete(&mut self) {
        self.view_buffer.delete(self.location);
        self.mark_redraw(true);
    }

    pub fn insert_newline(&mut self) {
        self.view_buffer.insert_newline(self.location);
        self.handle_move_command(Move::Right);
        self.mark_redraw(true);
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
            return String::new();
        }
        let welcome_message = format!("{NAME} editor -- version {VERSION}");
        let len = welcome_message.len();
        let remaining_width = width.saturating_sub(1);
        if remaining_width <= len {
            return "~".to_string();
        }
        format!("{:<1}{:^remaining_width$}", "~", welcome_message)
    }

    pub fn load(&mut self, file_name: &str) -> Result<(), std::io::Error> {
        let buffer = Buffer::load(file_name)?;
        self.view_buffer = buffer;
        self.mark_redraw(true);
        Ok(())
    }
    
    pub const fn is_file_loaded(&self) -> bool {
        self.view_buffer.is_file_loaded()
    }

    pub fn save_as(&mut self, file_name: &str) -> Result<(), std::io::Error> {
        self.view_buffer.save_as(file_name)
    }
}

impl UIComponent for View {
    fn mark_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }
    fn set_size(&mut self, size: Size) {
        self.size = size;
        self.scroll_text_location_into_view();
    }

    fn draw(&mut self, origin_y: usize) -> Result<(), std::io::Error> {
        let Size { height, width } = self.size;
        let end_y = origin_y.saturating_add(height as usize);
        // we allow this since we don't care if our welcome message is put _exactly_ in the top third.
        // it's allowed to be a bit too far up or down
        #[allow(clippy::integer_division)]
        let top_third = height as usize / 3;
        let scroll_top = self.offset.y as usize;
        for current_row in origin_y..end_y {
            if let Some(line) = self.view_buffer.lines.get(
                current_row
                    .saturating_sub(origin_y)
                    .saturating_add(scroll_top) as usize,
            ) {
                let left = self.offset.x;
                let right = self.offset.x.saturating_add(width);
                Self::render_line(
                    current_row as usize,
                    &line.get_visible_graphemes(left as usize..right as usize),
                );
            } else if current_row == top_third && self.view_buffer.is_empty() {
                Self::render_line(
                    current_row as usize,
                    &Self::build_welcome_message(width as usize),
                );
            } else {
                Self::render_line(current_row as usize, "~");
            }
            if current_row.saturating_add(1) < height as usize {
                let _ = Terminal::print("\r\n");
            }
        }
        Ok(())
    }
    
}
