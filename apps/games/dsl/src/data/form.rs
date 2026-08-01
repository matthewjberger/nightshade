#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    Number,
    Truth,
    Text,
    Vector,
    List,
    Thing,
}

pub const FORMS: &[(&str, Form)] = &[
    ("number", Form::Number),
    ("truth", Form::Truth),
    ("boolean", Form::Truth),
    ("text", Form::Text),
    ("vector", Form::Vector),
    ("list", Form::List),
    ("thing", Form::Thing),
];

pub fn form_of(name: &str) -> Option<Form> {
    FORMS
        .iter()
        .find(|(held, _)| *held == name)
        .map(|(_, form)| *form)
}

pub fn name_of(form: Form) -> &'static str {
    match form {
        Form::Number => "a number",
        Form::Truth => "a truth",
        Form::Text => "a line of text",
        Form::Vector => "a vector",
        Form::List => "a list",
        Form::Thing => "a thing",
    }
}

pub fn agrees(wanted: Form, held: Form) -> bool {
    if wanted == held {
        return true;
    }
    matches!(
        (wanted, held),
        (Form::Vector, Form::Number) | (Form::Number, Form::Truth) | (Form::List, Form::Vector)
    )
}
