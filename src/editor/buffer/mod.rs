use crate::editor::line::Line;
use std::fs::read_to_string;
use std::io::Error;

use super::terminal::Position;

pub struct Buffer {
    pub lines: Vec<Line>,
}

impl Buffer {
    pub fn default() -> Buffer {
        Buffer { lines: vec![] }
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

    pub fn insert_char(&mut self, character: char, at: Position) {
        if at.y as usize > self.lines.len() {
            return;
        }
        if at.y as usize == self.lines.len() {
            // 最后一行直接插入 当在输入最后一行后 行数增加后续走下面分支
            self.lines.push(Line::from(&character.to_string()));
        } else if let Some(line) = self.lines.get_mut(at.y as usize) {
            // 正常行插入
            line.insert_char(character, at.x as usize);
        }
    }
}
