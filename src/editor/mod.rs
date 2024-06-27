mod terminal;
mod view;
mod buffer;
mod line;
mod editorcommand;

use view::View;
use crossterm::event::{read, KeyEvent, KeyEventKind, Event};
use terminal::{Position, Terminal};
use std::{
    env,
    io::Error,
    panic::{set_hook, take_hook},
};
use editorcommand::EditorCommand;

pub struct Editor {
    should_quit: bool,
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
            // let _ = self.render_bottom_info();
            let _ = Terminal::move_cursor_to(self.view.caret_position());
        }
        let _ = Terminal::show_cursor();
        let _ = Terminal::execute();

    }

    fn evaluate_event(&mut self, event: crossterm::event::Event)  {
        let should_process = match &event {
            Event::Key(KeyEvent { kind, .. }) => kind == &KeyEventKind::Press,
            Event::Resize(_, _) => true,
            _ => false,
        };
        if should_process {
            match EditorCommand::try_from(event) {
                Ok(command) => {
                    if matches!(command, EditorCommand::Quit) {
                        self.should_quit = true;
                    } else {
                        self.view.handle_command(command);
                    }
                }
                Err(err) => {
                    #[cfg(debug_assertions)]
                    {
                        panic!("Could not handle command: {err}");
                    }
                }
            }
        }else {
            // #[cfg(debug_assertions)]
            // {
            //     panic!("Received and discarded unsupported or non-press event.");
            // }
        }


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