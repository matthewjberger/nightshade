use nightshade_dialogue::Conversation;

#[derive(Clone, Copy)]
pub enum Deed {
    Accept(usize),
    Complete(usize),
    Close,
}

#[derive(Default)]
pub struct Deeds(pub Vec<Deed>);

#[derive(Default)]
pub struct Objective(pub String);

#[derive(Default)]
pub struct Shopping {
    pub trader: usize,
}

nightshade::ecs::resource_bundle! {
    pub struct Speech<'a> {
        write {
            conversation: Conversation,
            deeds: Deeds,
            objective: Objective,
            shopping: Shopping,
        }
        read {
        }
    }
}
