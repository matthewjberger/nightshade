#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum UiLayer {
    #[default]
    Background = 0,
    DockedPanels = 10,
    FloatingPanels = 20,
    Popups = 30,
    DockIndicator = 40,
    Tooltips = 50,
}
