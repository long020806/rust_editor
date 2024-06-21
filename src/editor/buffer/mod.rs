pub struct Buffer{
    pub lines:Vec<String>
}

impl Buffer{
    pub fn default(text:Option<Vec<String>>) -> Buffer{
        let buffer = match text {
            Some(lines) => {
                Buffer{lines}
            },
            None => {
                Buffer{
                    lines:vec!["Hello, World!".to_string()]
                }
            },
        };
        buffer

    }

    fn save_buffer(self:&mut Self,str:&str)->Result<(),std::io::Error>{
        self.lines.push(str.to_string());
        Ok(())
    }
}
