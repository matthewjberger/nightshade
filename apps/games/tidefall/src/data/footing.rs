stateless::statemachine! {
    transitions: {
        *Flying + Touch = Landed,
        Landed + Burn = Flying,
    }
}
