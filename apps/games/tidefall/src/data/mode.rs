stateless::statemachine! {
    transitions: {
        Aboard + Board = Afoot,
        *Afoot + Board = Aboard,
    }
}
