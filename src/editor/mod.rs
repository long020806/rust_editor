mod terminal;
mod view;
mod buffer;
use std::{cmp::min, env, io::Error};
use view::View;
use crossterm::event::Event::{Key,Resize};
use crossterm::event::{read, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use terminal::{Position, Size, Terminal};
pub struct Editor {
    should_quit: bool,
    location: Position,
    view:View
}
impl Editor {
    pub fn default() -> Self {
        Self {
            should_quit: false,
            location: Position { x: 0, y: 0 },
            view:View::default()
        }
    }


     
    fn handle_args(&mut self) {
        let args: Vec<String> = env::args().collect();
        if let Some(file_name) = args.get(1) {
            self.view.load(file_name);
        }
    }

    pub fn run(&mut self) {
        Terminal::initialize().unwrap();
        self.handle_args();
        let result = self.repl();

        Terminal::terminate().unwrap();
        result.unwrap();
    }

    fn repl(&mut self) -> Result<(), std::io::Error> {
        loop {
            self.refresh_screen()?;
            if self.should_quit {
                break;
            }
            let event = read()?;
            self.evaluate_event(event)?;
        }
        Ok(())
    }

    fn refresh_screen(&mut self) -> Result<(), std::io::Error> {
        Terminal::hide_cursor()?;
        Terminal::move_cursor_to(Position{x:0,y:0})?;
        if self.should_quit {
            Terminal::clear_screen()?;
            Terminal::print("Goodbye.\r\n")?;
        } else {
            self.view.render()?;
            self.render_bottom_info()?;
            Terminal::move_cursor_to(Position { x: self.location.x, y:  self.location.y })?;
        }
        Terminal::show_cursor()?;
        Terminal::execute()?;
        Ok(())
    }

    fn render_bottom_info(&self) -> Result<(), std::io::Error>{
        // 打印x,y,width,height信息
        let Size{ height,width } = Terminal::size()?;
        Terminal::move_cursor_to(Position{x:0,y:height + 1})?;
        Terminal::clear_line()?;
        Terminal::print(format!("x:{} y:{} width:{} height:{}",self.location.x,self.location.y,width,height).as_str())?;
        Ok(())
    }

    fn evaluate_event(&mut self, event: crossterm::event::Event) -> Result<(), std::io::Error> {
        match event
        { 
            Key(KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press,
                ..
            }) => {
                match code {
                    KeyCode::Char('q') if modifiers == KeyModifiers::CONTROL => {
                        self.should_quit = true;
                    }
                    KeyCode::Up
                    | KeyCode::Down
                    | KeyCode::Left
                    | KeyCode::Right
                    | KeyCode::PageDown
                    | KeyCode::PageUp
                    | KeyCode::End
                    | KeyCode::Home => {
                        self.view.set_render(true);
                        self.move_point(code)?;
                    }
    
                    _ => {
                    },
                }     
            }
            Resize( width , height) => {
                self.view.resize(Size{width:width,height:height})
            } 
            _=>{

            },
      
        };


        Ok(())
    }

    fn move_point(&mut self, key_code: KeyCode) -> Result<(), Error> {
        let Position { mut x, mut y } = self.location;
        let Size { height, width } = Terminal::size()?;
        match key_code {
            KeyCode::Up => {
                y = y.saturating_sub(1);
            }
            KeyCode::Down => {
                y = min(height.saturating_sub(1), y.saturating_add(1));
            }
            KeyCode::Left => {
                x = x.saturating_sub(1);
            }
            KeyCode::Right => {
                x = min(width.saturating_sub(1), x.saturating_add(1));
            }
            KeyCode::PageUp => {
                y = 0;
            }
            KeyCode::PageDown => {
                y = height.saturating_sub(1);
            }
            KeyCode::Home => {
                x = 0;
            }
            KeyCode::End => {
                x = width.saturating_sub(1);
            }
            _ => (),
        }
        self.location = Position { x, y };
        Ok(())
    }
}
