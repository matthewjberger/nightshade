stateless::statemachine! {
    transitions: {
        *Off + Engage = Align,
        Align + Aligned = Burn,
        Burn + Turn = Brake,
        Brake + Arrive = Descend,
        Align + Cancel = Off,
        Burn + Cancel = Off,
        Brake + Cancel = Off,
        Descend + Cancel = Off,
    }
}
