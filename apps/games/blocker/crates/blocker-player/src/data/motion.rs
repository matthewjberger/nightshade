stateless::statemachine! {
    transitions: {
        *Walk + Fly = Fly,
        Fly + Fly = Walk,
        Walk + Watch = Watch,
        Watch + Watch = Walk,
        Fly + Watch = Watch,
        Walk + Soar = Glide,
        Glide + Land = Walk,
    }
}
