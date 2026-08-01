#[derive(Clone, Debug, PartialEq)]
pub enum TemplateCommand {
    Ping { message: String },
}

#[derive(Clone, Debug, PartialEq)]
pub enum TemplateEvent {
    Pong { message: String },
}

#[derive(Default)]
pub struct TemplateCommands {
    pub pending: Vec<TemplateCommand>,
}
