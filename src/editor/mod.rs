mod buffer;
mod documentstatus;
mod editorcommand;
mod fileinfo;
mod line;
mod statusbar;
mod terminal;
mod view;
use view::View;
mod messagebar;
mod uicomponent;
use crossterm::event::{read, Event, KeyEvent, KeyEventKind};
use editorcommand::EditorCommand;
use statusbar::StatusBar;
use std::{
    env,
    io::Error,
    panic::{set_hook, take_hook},
};
use terminal::{Position, Terminal};

use self::{messagebar::MessageBar, terminal::Size, uicomponent::UIComponent};

pub const NAME: &str = env!("CARGO_PKG_NAME");
// pub const VERSION: &str = env!("CARGO_PKG_VERSION");
#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    view: View,
    statusbar: StatusBar,
    message_bar: MessageBar,
    terminal_size: Size,
    title: String,
}
impl Editor {
    pub fn new() -> Result<Self, Error> {
        let current_hook = take_hook();
        set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));
        Terminal::initialize()?;
        let mut editor = Self::default();
        let size = Terminal::size().unwrap_or_default();
        editor.resize(size);
        let args: Vec<String> = env::args().collect();
        if let Some(file_name) = args.get(1) {
            editor.view.load(file_name);
        }
        editor
            .message_bar
            .update_message("HELP: Ctrl-S = save | Ctrl-Q = quit".to_string());
        editor.refresh_status();
        Ok(editor)
    }

    fn resize(&mut self, size: Size) {
        self.terminal_size = size;
        self.view.resize(Size {
            height: size.height.saturating_sub(2),
            width: size.width,
        });
        self.message_bar.resize(Size {
            height: 1,
            width: size.width,
        });
        self.statusbar.resize(Size {
            height: 1,
            width: size.width,
        });
    }

    pub fn refresh_status(&mut self) {
        let status = self.view.get_status();
        let title = format!("{} - {NAME}", status.file_name);
        self.statusbar.update_status(status);
        if title != self.title && matches!(Terminal::set_title(&title), Ok(())) {
            self.title = title;
        }
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
            let status = self.view.get_status();
            self.statusbar.update_status(status);
        }
    }

    fn refresh_screen(&mut self) {
        if self.terminal_size.height == 0 || self.terminal_size.width == 0 {
            return;
        }
        let _ = Terminal::hide_cursor();
        let _ = Terminal::move_cursor_to(Position { x: 0, y: 0 });
        self.message_bar
            .render(self.terminal_size.height.saturating_sub(1) as usize);
        if self.terminal_size.height > 1 {
            self.statusbar
                .render(self.terminal_size.height.saturating_sub(2) as usize);
        }
        if self.terminal_size.height > 2 {
            self.view.render(0);
        }
        let _ = Terminal::move_cursor_to(self.view.caret_position());
        let _ = Terminal::show_cursor();
        let _ = Terminal::execute();
    }

    fn evaluate_event(&mut self, event: crossterm::event::Event) {
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
                    } else if let EditorCommand::Resize(size) = command {
                        self.resize(size);
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
        } else {
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
