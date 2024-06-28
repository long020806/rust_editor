use std::{ops::Range, fmt};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Copy, Clone)]
enum GraphemeWidth {
    Half,
    Full,
}

impl GraphemeWidth {
    const fn saturating_add(self, other: usize) -> usize {
        match self {
            Self::Half => other.saturating_add(1),
            Self::Full => other.saturating_add(2),
        }
    }
}

struct TextFragment {
    grapheme: String,
    rendered_width: GraphemeWidth,
    replacement: Option<char>,
}
#[derive(Default)]
pub struct Line {
    fragments: Vec<TextFragment>,
}

impl Line {
    pub fn from(line_str: &str) -> Self {
        let fragments = Self::str_to_fragments(line_str);

        Self { fragments }
    }
    /**
     * 获取可见的 字符族
     */
    pub fn get_visible_graphemes(&self, range: Range<usize>) -> String {
        if range.start >= range.end {
            return String::new();
        }
        let mut result = String::new();
        let mut current_pos = 0;
        for fragment in &self.fragments {
            let fragment_end = fragment.rendered_width.saturating_add(current_pos);
            if current_pos >= range.end {
                break;
            }
            if fragment_end > range.start {
                if fragment_end > range.end || current_pos < range.start {
                    // Clip on the right or left
                    result.push('⋯');
                } else if let Some(char) = fragment.replacement {
                    result.push(char);
                } else {
                    result.push_str(&fragment.grapheme);
                }
            }
            current_pos = fragment_end;
        }
        result
    }
    /**
     * 获取字符族长度
     */
    pub fn grapheme_count(&self) -> usize {
        self.fragments.len()
    }

    pub fn grapheme_count_u16(&self) -> u16 {
        self.grapheme_count() as u16
    }
    /**
     * 获取index字符串的长度
     */
    pub fn width_until(&self, grapheme_index: usize) -> usize {
        self.fragments
            .iter()
            .take(grapheme_index)
            .map(|fragment| match fragment.rendered_width {
                GraphemeWidth::Half => 1,
                GraphemeWidth::Full => 2,
            })
            .sum()
    }

    pub fn width_until_u16(&self, grapheme_index: usize) -> u16 {
        self.width_until(grapheme_index) as u16
    }

    fn str_to_fragments(line_str: &str) -> Vec<TextFragment> {
        line_str
            .graphemes(true)
            .map(|grapheme| {
                let unicode_width = grapheme.width();
                let rendered_width = match unicode_width {
                    0 | 1 => GraphemeWidth::Half,
                    _ => GraphemeWidth::Full,
                };
                let replacement = match unicode_width {
                    0 => Some('·'),
                    _ => None,
                };
                TextFragment {
                    grapheme: grapheme.to_string(),
                    rendered_width,
                    replacement,
                }
            })
            .collect()
    }

    pub fn insert_char(&mut self, character: char, grapheme_index: usize) {
        let mut result = String::new();
        for (index, fragment) in self.fragments.iter().enumerate() {
            // 对应位置添加时 直接加入 其他使用原字符
            if index == grapheme_index {
                result.push(character);
            }
            result.push_str(&fragment.grapheme);
        }
        // 如果在最后行尾直接增加
        if grapheme_index >= self.fragments.len() {
            result.push(character);
        }
        // 重新转换
        self.fragments = Self::str_to_fragments(&result);
    }

    pub fn delete(&mut self,grapheme_index: usize){
        let mut result = String::new();
        for (index, fragment) in self.fragments.iter().enumerate() {
            // 对应位置添加时 直接加入 其他使用原字符
            if index == grapheme_index {
                continue;
            }
            result.push_str(&fragment.grapheme);
        }
        // 重新转换
        self.fragments = Self::str_to_fragments(&result);
    }

    pub fn append(&mut self, other: &Self) {
        let mut concat = self.to_string();
        concat.push_str(&other.to_string());
        self.fragments = Self::str_to_fragments(&concat);
    }

    pub fn split(&mut self,at:usize) -> Self{
        if at > self.fragments.len() {
            return Self::default();
        }
        let remainder = self.fragments.split_off(at);
        Self {
            fragments: remainder,
        }
    }

}
/**
 * 提供to_string 方法
 */
impl fmt::Display for Line {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let result: String = self
            .fragments
            .iter()
            .map(|fragment| fragment.grapheme.clone())
            .collect();
        write!(formatter, "{result}")
    }
}
