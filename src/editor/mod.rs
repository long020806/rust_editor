mod buffer;
mod commandbar;
mod documentstatus;
mod editorcommand;
mod fileinfo;
mod line;
mod statusbar;
mod terminal;
mod view;
use command::{
    Command::{self, Edit, Move, System},
    Edit::InsertNewline,
    System::{Dismiss, Quit, Resize, Save,Search},
};

use commandbar::CommandBar;
use view::View;
mod messagebar;
mod uicomponent;
use crossterm::event::{read, Event, KeyEvent, KeyEventKind};
mod command;
use statusbar::StatusBar;
use std::{
    env,
    io::Error,
    panic::{set_hook, take_hook},
};
use std::fmt::format;
use terminal::{Position, Terminal};

use self::{messagebar::MessageBar, terminal::Size, uicomponent::UIComponent};

pub const NAME: &str = env!("CARGO_PKG_NAME");
// pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const QUIT_TIMES: u8 = 3;

#[derive(Eq, PartialEq, Default)]
enum PromptType {
    Search,
    Save,
    #[default]
    None,
}

impl PromptType {
    fn is_none(&self) -> bool {
          *self == Self::None
    }
}

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    view: View,
    statusbar: StatusBar,
    message_bar: MessageBar,
    command_bar: CommandBar,
    prompt_type: PromptType,
    terminal_size: Size,
    title: String,
    quit_times: u8,
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
        editor
            .message_bar
            .update_message("HELP: Ctrl-F = search | Ctrl-S = save | Ctrl-Q = quit");
        let args: Vec<String> = env::args().collect();
        if let Some(file_name) = args.get(1) {
            if editor.view.load(file_name).is_err() {
                // editor.view.load(file_name);
                editor
                    .message_bar
                    .update_message(&format!("ERR: Could not open file: {file_name}"));
            }
        }
        editor.refresh_status();
        Ok(editor)
    }

    fn process_command(&mut self, command: Command) {
        match command {
            System(Quit) => {
                if self.prompt_type.is_none() {
                    self.handle_quit();
                }
            }
            System(Resize(size)) => self.resize(size),
            _ => self.reset_quit_times(), // Reset quit times for all other commands
        }
        self.message_bar.update_message(format!("{:?}",command).as_str());
        self.message_bar.set_needs_redraw(true);
        match command {
            System(Quit | Resize(_)) => {} // already handled above
            System(Search) =>{
                if self.prompt_type.is_none() {
                    self.prompt_type = PromptType::Search;
                    self.handle_search();
                }
            }
            System(Save) => {
                if self.prompt_type.is_none() {
                    self.prompt_type = PromptType::Save;
                    self.handle_save();
                }
            }
            System(Dismiss) => {
                match self.prompt_type{
                    PromptType::Save => {
                        self.dismiss_prompt();
                        self.message_bar.update_message("Save aborted.");
                        self.prompt_type = PromptType::None;
                    }
                    PromptType::Search =>{
                        self.dismiss_prompt();
                        self.message_bar.update_message("Search aborted.");
                        self.prompt_type = PromptType::None;
                    },
                    PromptType::None =>{

                    }
                } 
            }
            Edit(edit_command) => {
                match self.prompt_type {
                    PromptType::Search => {
                        if matches!(edit_command, InsertNewline) {
                            let search_value = self.command_bar.value();
                            self.dismiss_prompt();
                            self.search(Some(&search_value));
                            self.prompt_type = PromptType::None;
                        }else{
                            self.command_bar.handle_edit_command(edit_command);
                        }
                    }
                    PromptType::Save => {
                        if matches!(edit_command, InsertNewline) {
                            let file_name = self.command_bar.value();
                            self.dismiss_prompt();
                            self.save(Some(&file_name));
                            self.prompt_type = PromptType::None;
                        }else{
                            self.command_bar.handle_edit_command(edit_command);
                        }
                    }
                    PromptType::None => {self.view.handle_edit_command(edit_command);}
                } 
            }

            Move(move_command) => {
                if self.prompt_type.is_none() {
                    self.view.handle_move_command(move_command);
                }
            }
        }
    }
    fn dismiss_prompt(&mut self) {
        self.message_bar.set_needs_redraw(true);
    }

    fn show_prompt(&mut self,prompt:&str) {
        let mut command_bar = CommandBar::default();
        command_bar.set_prompt(prompt);
        command_bar.resize(Size {
            height: 1,
            width: self.terminal_size.width,
        });
        command_bar.set_needs_redraw(true);
        self.command_bar = command_bar;
    }
    fn handle_save(&mut self) {
        if self.view.is_file_loaded() {
            self.save(None);
        } else {
            self.show_prompt("Save as: ");
        }
    }

    fn handle_search(&mut self) {
        self.show_prompt("Search: ");
    }

    fn search(&mut self, search_value: Option<&str>){
        if let Some(search_value) = search_value {
            self.message_bar.update_message(format!("Search {} successfully.",search_value).as_str());

        }
    }

    fn save(&mut self, file_name: Option<&str>) {
        let result = if let Some(name) = file_name {
            self.view.save_as(name)
        } else {
            self.view.save()
        };
        if result.is_ok() {
            self.message_bar.update_message("File saved successfully.");
        } else {
            self.message_bar.update_message("Error writing file!");
            if self.terminal_size.height == 0 || self.terminal_size.width == 0 {
                return;
            }
            let bottom_bar_row = self.terminal_size.height.saturating_sub(1);
            let _ = Terminal::hide_cursor();
            match self.prompt_type {
                PromptType::Search | PromptType::Save =>{
                    self.command_bar.render(bottom_bar_row as usize);
                }
                PromptType::None => {
                    self.message_bar.render(bottom_bar_row as usize);

                }
            } 
            if self.terminal_size.height > 1 {
                self.statusbar
                    .render(self.terminal_size.height.saturating_sub(2) as usize);
            }
            if self.terminal_size.height > 2 {
                self.view.render(0);
            }
            let new_caret_pos = match self.prompt_type {
                PromptType::Search | PromptType::Save =>{
                    Position {
                        x: bottom_bar_row,
                        y: self.command_bar.caret_position_col() as u16,
                    }
                }
                PromptType::None => {
                    self.view.caret_position()
                }
            };
            let _ = Terminal::move_cursor_to(new_caret_pos);
            let _ = Terminal::show_cursor();
            let _ = Terminal::execute();
        }
    }
    // clippy::arithmetic_side_effects: quit_times is guaranteed to be between 0 and QUIT_TIMES
    #[allow(clippy::arithmetic_side_effects)]
    fn handle_quit(&mut self) {
        if !self.view.get_status().is_modified || self.quit_times + 1 == QUIT_TIMES {
            self.should_quit = true;
        } else if self.view.get_status().is_modified {
            self.message_bar.update_message(&format!(
                "WARNING! File has unsaved changes. Press Ctrl-Q {} more times to quit.",
                QUIT_TIMES - self.quit_times - 1
            ));

            self.quit_times += 1;
        }
    }
    fn reset_quit_times(&mut self) {
        if self.quit_times > 0 {
            self.quit_times = 0;
            self.message_bar.update_message("");
        }
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
        match self.prompt_type {
            PromptType::Search | PromptType::Save =>{
                self.command_bar.resize(Size {
                    height: 1,
                    width: size.width,
                });
            }
            _ => {}
        } 
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
        match self.prompt_type {
            PromptType::Search | PromptType::Save =>{
                self.command_bar
                .render(self.terminal_size.height.saturating_sub(1) as usize);
            }
            PromptType::None => {
                self.message_bar
                .render(self.terminal_size.height.saturating_sub(1) as usize);
            }
        } 
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
            if let Ok(command) = Command::try_from(event) {
                self.process_command(command);
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
