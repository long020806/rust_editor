pub struct Buffer{
    pub lines:Vec<String>
}

impl Buffer{
    pub fn default() -> Buffer{
        Buffer{
            lines:vec!["Hello, World!".to_string()]
        }
    }

    fn save_buffer(self:&mut Self,str:&str)->Result<(),std::io::Error>{
        self.lines.push(str.to_string());
        Ok(())
    }
}
