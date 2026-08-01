use crate::data::Ask;
use nightshade_input::{Action, Actions, Binding, Press};

pub fn describe(binding: &Binding) -> String {
    match binding {
        Binding::Key(key) => format!("{key:?}"),
        Binding::Keys {
            up,
            down,
            left,
            right,
        } => format!("{up:?} {left:?} {down:?} {right:?}"),
        Binding::Pointer(button) => format!("{button:?} button"),
        Binding::Pad(button) => format!("pad {button:?}"),
        Binding::Stick(stick) => format!("{stick:?} stick"),
        Binding::Trigger(pulled) => format!("{pulled:?}"),
        Binding::Pair { positive, negative } => {
            format!(
                "{} / {}",
                describe_press(positive),
                describe_press(negative)
            )
        }
    }
}

pub fn describe_all(bindings: &[Binding]) -> String {
    if bindings.is_empty() {
        return "nothing".to_string();
    }
    bindings.iter().map(describe).collect::<Vec<_>>().join(", ")
}

pub fn held_names(actions: &Actions<Ask>) -> String {
    let names: Vec<&str> = Ask::all()
        .iter()
        .filter(|ask| actions.pressed(**ask))
        .map(Action::name)
        .collect();
    if names.is_empty() {
        return "held: nothing".to_string();
    }
    format!("held: {}", names.join(", "))
}

fn describe_press(press: &Press) -> String {
    match press {
        Press::Key(key) => format!("{key:?}"),
        Press::Pointer(button) => format!("{button:?} button"),
        Press::Pad(button) => format!("pad {button:?}"),
        Press::Trigger(pulled) => format!("{pulled:?}"),
    }
}
