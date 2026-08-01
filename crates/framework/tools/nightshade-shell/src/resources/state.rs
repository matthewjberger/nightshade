use nightshade::ecs::Entity;
use nightshade::prelude::Commands;
use nightshade_platform::KeyCode;

use crate::data::{Command, OutputLine, format_entity};

pub struct ShellState<C> {
    pub context: C,
    pub visible: bool,
    pub input_buffer: String,
    pub history: Vec<String>,
    pub output: Vec<OutputLine>,
    pub history_index: Option<usize>,
    pub registry: CommandRegistry<C>,
    pub animation_progress: f32,
    pub scroll_to_bottom: bool,
    pub height: f32,
    pub scroll_offset: f32,
    pub dragging_resize: bool,
    pub drag_start_y: f32,
    pub drag_start_height: f32,
    pub pending_enter: bool,
    pub pending_up: bool,
    pub pending_down: bool,
    pub pending_escape: bool,
}

impl<C: Default> Default for ShellState<C> {
    fn default() -> Self {
        Self::new(C::default())
    }
}

impl<C> ShellState<C> {
    pub fn new(context: C) -> Self {
        Self {
            context,
            visible: false,
            input_buffer: String::new(),
            history: Vec::new(),
            output: vec![OutputLine {
                text: "Type 'help' for available commands.".to_string(),
                is_command: false,
            }],
            history_index: None,
            registry: CommandRegistry::new(),
            animation_progress: 0.0,
            scroll_to_bottom: true,
            height: 100.0,
            scroll_offset: 0.0,
            dragging_resize: false,
            drag_start_y: 0.0,
            drag_start_height: 0.0,
            pending_enter: false,
            pending_up: false,
            pending_down: false,
            pending_escape: false,
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn insert_text(&mut self, text: &str) {
        if !self.input_buffer.is_empty() && !self.input_buffer.ends_with(' ') {
            self.input_buffer.push(' ');
        }
        self.input_buffer.push_str(text);
    }

    pub fn insert_entity(&mut self, entity: Entity) {
        self.insert_text(&format_entity(entity));
    }

    pub fn handle_key(&mut self, key: KeyCode, pressed: bool) {
        if pressed {
            match key {
                KeyCode::Enter => self.pending_enter = true,
                KeyCode::ArrowUp => self.pending_up = true,
                KeyCode::ArrowDown => self.pending_down = true,
                KeyCode::Escape => self.pending_escape = true,
                KeyCode::Backspace => {
                    self.input_buffer.pop();
                }
                _ => {}
            }
        }
    }

    pub fn execute_command(&mut self, commands: &mut Commands) {
        let input = self.input_buffer.trim().to_string();
        if input.is_empty() {
            return;
        }

        self.output.push(OutputLine {
            text: format!("> {}", input),
            is_command: true,
        });

        self.history.push(input.clone());
        self.history_index = None;

        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            self.input_buffer.clear();
            return;
        }

        let command_name = parts[0];
        let args = &parts[1..];

        if command_name == "clear" {
            self.output.clear();
        } else if command_name == "help" {
            if args.is_empty() {
                let mut help_text = String::from("Commands:\n");
                help_text.push_str("  clear - Clear the console output\n");
                help_text.push_str("  help - List all available commands\n");

                let builtin_names = self.registry.builtin_command_names();
                if !builtin_names.is_empty() {
                    help_text.push_str("\nBuilt-in:\n");
                    for name in builtin_names {
                        if let Some(command) = self.registry.get(name) {
                            help_text.push_str(&format!("  {} - {}\n", name, command.description));
                        }
                    }
                }

                let app_names = self.registry.app_command_names();
                if !app_names.is_empty() {
                    help_text.push_str("\nApp:\n");
                    for name in app_names {
                        if let Some(command) = self.registry.get(name) {
                            help_text.push_str(&format!("  {} - {}\n", name, command.description));
                        }
                    }
                }

                self.output.push(OutputLine {
                    text: help_text,
                    is_command: false,
                });
            } else if let Some(command) = self.registry.get(args[0]) {
                self.output.push(OutputLine {
                    text: format!(
                        "{}\nUsage: {}\n{}",
                        command.name, command.usage, command.description
                    ),
                    is_command: false,
                });
            } else {
                self.output.push(OutputLine {
                    text: format!("Unknown command: {}", args[0]),
                    is_command: false,
                });
            }
        } else if let Some(command) = self.registry.get(command_name) {
            let execute = command.execute;
            let result = execute(args, commands, &mut self.context);
            if !result.is_empty() {
                self.output.push(OutputLine {
                    text: result,
                    is_command: false,
                });
            }
        } else {
            self.output.push(OutputLine {
                text: format!("Unknown command: {}", command_name),
                is_command: false,
            });
        }

        self.input_buffer.clear();
        self.scroll_to_bottom = true;
    }

    pub fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }

        match self.history_index {
            None => {
                self.history_index = Some(self.history.len() - 1);
            }
            Some(index) if index > 0 => {
                self.history_index = Some(index - 1);
            }
            _ => {}
        }

        if let Some(index) = self.history_index {
            self.input_buffer = self.history[index].clone();
        }
    }

    pub fn history_down(&mut self) {
        match self.history_index {
            Some(index) if index + 1 < self.history.len() => {
                self.history_index = Some(index + 1);
                self.input_buffer = self.history[index + 1].clone();
            }
            Some(_) => {
                self.history_index = None;
                self.input_buffer.clear();
            }
            None => {}
        }
    }

    pub fn update_animation(&mut self, delta_time: f32) {
        let target = if self.visible { 1.0 } else { 0.0 };
        let speed = 8.0;
        self.animation_progress += (target - self.animation_progress) * speed * delta_time;
        self.animation_progress = self.animation_progress.clamp(0.0, 1.0);
    }

    pub fn should_render(&self) -> bool {
        self.animation_progress > 0.001
    }
}

pub struct CommandRegistry<C> {
    pub builtin: Vec<Command<C>>,
    pub app: Vec<Command<C>>,
}

impl<C> Default for CommandRegistry<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C> CommandRegistry<C> {
    pub fn new() -> Self {
        Self {
            builtin: Vec::new(),
            app: Vec::new(),
        }
    }

    pub fn register(&mut self, command: Command<C>) {
        debug_assert!(
            !self.contains(command.name),
            "shell command \"{}\" already registered",
            command.name
        );
        self.app.push(command);
    }

    pub fn register_builtin(&mut self, command: Command<C>) {
        debug_assert!(
            !self.contains(command.name),
            "shell command \"{}\" already registered",
            command.name
        );
        self.builtin.push(command);
    }

    pub fn get(&self, name: &str) -> Option<&Command<C>> {
        self.builtin
            .iter()
            .chain(self.app.iter())
            .find(|command| command.name == name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    pub fn builtin_command_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.builtin.iter().map(|command| command.name).collect();
        names.sort();
        names
    }

    pub fn app_command_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.app.iter().map(|command| command.name).collect();
        names.sort();
        names
    }
}
