use std::fs::read_to_string;
use std::io::Error;
use crate::editor::line::Line;

pub struct Buffer {
    pub lines: Vec<Line>,
}

impl Buffer {
    pub fn default() -> Buffer {
        Buffer {
            lines: vec![]
        }
    }

    pub fn load(file_name: &str) -> Result<Self, Error> {
        let contents = read_to_string(file_name)?;
        let mut lines = Vec::new();
        for value in contents.lines() {
            lines.push(Line::from(value));
        }
        Ok(Self { lines })
    }
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn height(&self) -> usize {
        self.lines.len()
    }

    pub fn height_u16(&self) -> u16 {
        self.height() as u16
    }

}
