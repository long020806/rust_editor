use crate::editor::line::Line;
use std::fs::{read_to_string, File};
use std::io::Error;
use std::io::Write;
use super::terminal::Position;
use super::fileinfo::FileInfo;
#[derive(Default,Clone)]
pub struct Buffer {
    pub lines: Vec<Line>,
    pub file_info: FileInfo,
    pub dirty:bool
}

impl Buffer {

    pub fn load(file_name: &str) -> Result<Self, Error> {
        let contents = read_to_string(file_name)?;
        let mut lines = Vec::new();
        for value in contents.lines() {
            lines.push(Line::from(value));
        }
        Ok(Self { lines,file_info:FileInfo::from(file_name),dirty:false })
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
            self.dirty = true;
        } else if let Some(line) = self.lines.get_mut(at.y as usize) {
            // 正常行插入
            line.insert_char(character, at.x as usize);
            self.dirty = true;
        }
    }

    pub fn delete(&mut self, at: Position) {
        if let Some(line) = self.lines.get(at.y as usize) {
            // 如果delete 执行在行尾且不失最后一行 则将下一行内容合并到当前行
            if at.x >= line.grapheme_count_u16()
                &&  self.lines.len() > at.y.saturating_add(1) as usize
            {
                let next_line = self.lines.remove(at.y.saturating_add(1) as usize);
                // clippy::indexing_slicing: We checked for existence of this line in the surrounding if statment
                #[allow(clippy::indexing_slicing)]
                self.lines[at.y as usize].append(&next_line);
                self.dirty = true;
            } else if at.x < line.grapheme_count_u16() {
                // clippy::indexing_slicing: We checked for existence of this line in the surrounding if statment
                #[allow(clippy::indexing_slicing)]
                self.lines[at.y as usize].delete(at.x as usize);
                self.dirty = true;
            }
        }
    }

    pub fn insert_newline(&mut self, at: Position){
        if at.y == self.height_u16() {
            self.lines.push(Line::default());
            self.dirty = true;
        } else if let Some(line) = self.lines.get_mut(at.y as usize) {
            let new = line.split(at.x as usize);
            self.lines.insert(at.y.saturating_add(1) as usize, new);
            self.dirty = true;
        }
    }

    pub fn save(&mut self) -> Result<(),Error>{
        if let Some(file_name) = &self.file_info.path {
            let mut file = File::create(file_name)?;
            for line in &self.lines {
                writeln!(file, "{line}")?;
            }
            self.dirty = false;
        }
        Ok(())
    }
}
