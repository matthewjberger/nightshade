use crate::data::cell::Cell;

#[derive(Clone, Copy, Debug)]
pub enum Ask {
    SetColumn {
        key: u16,
        row: u32,
        value: Cell,
    },
    AddColumn {
        key: u16,
        row: u32,
        value: Cell,
    },
    SetSpan {
        key: u16,
        row: u32,
        at: u32,
        count: u32,
    },
    AddSpan {
        key: u16,
        row: u32,
        at: u32,
        count: u32,
    },
    SetField {
        slot: u16,
        value: Cell,
    },
    AddField {
        slot: u16,
        value: Cell,
    },
    Emit {
        key: u16,
        value: Cell,
    },
    ClearBank {
        key: u16,
    },
    Keep {
        site: u16,
    },
    Recall {
        site: u16,
    },
}
