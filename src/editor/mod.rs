mod terminal;
mod view;
mod buffer;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::{cmp::min, io::Error};
use view::View;
use crossterm::event::Event::Key;
use crossterm::event::{read, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use terminal::{Position, Size, Terminal};
pub struct Editor {
    should_quit: bool,
    location: Position,
    view:View
}
impl Editor {
    pub fn default(text:Option<Vec<String>>) -> Self {
        Self {
            should_quit: false,
            location: Position { x: 0, y: 0 },
            view:View::default(text)
        }
    }

    pub fn read() -> Option<Vec<String>> {
        match std::env::args().skip(1).next() {
            Some(file_path) => {
                match File::open(file_path) {
                    file => {
                        match file {
                            Ok(file) => {
                                let lines = BufReader::new(file).lines();
                                println!("file_path:{:?}",lines);
                                let results:Vec<String> = lines.map(|item|{
                                        let result = match item {
                                            Ok(item) => {item},
                                            Err(_) => {"错误信息".to_string()},
                                        };
                                        result
                                    }).collect();
                                Some(results)
                            },
                            Err(_) => {
                                None
                            },
                        }
                    } 
                }
            },
            None => {
                None
            },
        }


    }

    pub fn run(&mut self) {
        Terminal::initialize().unwrap();
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
            self.evaluate_event(&event)?;
        }
        Ok(())
    }

    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        Terminal::hide_cursor()?;
        if self.should_quit {
            Terminal::clear_screen()?;
            Terminal::print("Goodbye.\r\n")?;
        } else {
            self.view.render()?;
            Terminal::move_cursor_to(Position { x: self.location.x, y:  self.location.y })?;
        }
        Terminal::show_cursor()?;
        Terminal::execute()?;
        Ok(())
    }
    
    fn evaluate_event(&mut self, event: &crossterm::event::Event) -> Result<(), std::io::Error> {
        if let Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            ..
        }) = event
        {
            match code {
                KeyCode::Char('q') if *modifiers == KeyModifiers::CONTROL => {
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
                    self.move_point(*code)?;
                }
                _ => (),
            }
        }
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
