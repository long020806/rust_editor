mod terminal;
mod view;
mod buffer;
use view::View;
use crossterm::event::Event::{Key,Resize};
use crossterm::event::{read, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use terminal::{Position, Size, Terminal};
use std::{
    env,
    io::Error,
    panic::{set_hook, take_hook},
    cmp::min
};
pub struct Editor {
    should_quit: bool,
    location: Position,
    view:View
}
impl Editor {


    pub fn new() -> Result<Self, Error> {
        let current_hook = take_hook();
        set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));
        Terminal::initialize()?;
        let mut view = View::default();
        let args: Vec<String> = env::args().collect();
        if let Some(file_name) = args.get(1) {
            view.load(file_name);
        }
        Ok(Self {
            should_quit: false,
            location: Position::default(),
            view,
        })
    }
    pub fn run(&mut self) {
        loop {
            self.refresh_screen();
            if self.should_quit {
                let _ = Terminal::terminate();
                break;
            }
            match read() {
                Ok(event) => self.evaluate_event(event),
                Err(err) => {
                    #[cfg(debug_assertions)]
                    {
                        panic!("Could not read event: {err:?}");
                    }
                }
            }
        }
    }


    fn refresh_screen(&mut self)  {
        let _ = Terminal::hide_cursor();
        let _ =  Terminal::move_cursor_to(Position{x:0,y:0});
        if self.should_quit {
            let _ = Terminal::clear_screen();
            let _ = Terminal::print("Goodbye.\r\n");
        } else {
            let _ = self.view.render();
            let _ = self.render_bottom_info();
            let _ = Terminal::move_cursor_to(Position { x: self.location.x, y:  self.location.y });
        }
        let _ = Terminal::show_cursor();
        let _ = Terminal::execute();

    }

    fn render_bottom_info(&self) {
        // 打印x,y,width,height信息
        let Size{ height,width } = Terminal::size().unwrap_or_default();
        let _ = Terminal::move_cursor_to(Position{x:0,y:height + 1});
        let _ = Terminal::clear_line();
        let _ = Terminal::print(format!("x:{} y:{} width:{} height:{}",self.location.x,self.location.y,width,height).as_str());

    }

    fn evaluate_event(&mut self, event: crossterm::event::Event)  {
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
                        self.move_point(code);
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

    }

    fn move_point(&mut self, key_code: KeyCode)  {
        let Position { mut x, mut y } = self.location;
        let Size { height, width } = Terminal::size().unwrap_or_default();
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
    }
}


impl Drop for Editor {
    fn drop(&mut self) {
        let _ = Terminal::terminate();
        if self.should_quit {
            let _ = Terminal::print("Goodbye.\r\n");
        }
    }
}