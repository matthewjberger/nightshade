pub const STATE_COUNT: usize = 8;

pub trait UiStateTrait: 'static {
    const INDEX: usize;
}

macro_rules! define_ui_states {
    ($($name:ident = $index:expr),* $(,)?) => {
        $(
            pub struct $name;
            impl UiStateTrait for $name {
                const INDEX: usize = $index;
            }
        )*
    };
}

define_ui_states!(
    UiBase = 0,
    UiHover = 1,
    UiPressed = 2,
    UiSelected = 3,
    UiFocused = 4,
    UiDisabled = 5,
    UiIntro = 6,
    UiOutro = 7,
);
