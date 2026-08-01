use crate::data::script::Fact;

pub const TITLE: &str = "HOLLOWAY & SONS";
pub const BYLINE: &str = "a walk round the building  Â·  d.okonkwo  Â·  1 room unfinished";

pub struct Way {
    pub label: &'static str,
    pub goes_to: &'static str,
    pub requires: &'static [Fact],
}

pub struct Place {
    pub id: &'static str,
    pub name: &'static str,
    pub body: &'static str,
    pub asserts: &'static [Fact],
    pub ways: &'static [Way],
    pub things: &'static [&'static str],
}

pub struct Thing {
    pub id: &'static str,
    pub label: &'static str,
    pub body: &'static str,
    pub requires: &'static [Fact],
    pub asserts: &'static [Fact],
    pub finishes: Option<crate::data::endings::Ending>,
}

const fn way(label: &'static str, goes_to: &'static str) -> Way {
    Way {
        label,
        goes_to,
        requires: &[],
    }
}

const fn way_if(label: &'static str, goes_to: &'static str, requires: &'static [Fact]) -> Way {
    Way {
        label,
        goes_to,
        requires,
    }
}

pub const MOST_WAYS: usize = most_ways();
pub const MOST_THINGS: usize = most_things();

const fn most_ways() -> usize {
    let mut most = 0;
    let mut index = 0;
    while index < PLACES.len() {
        if PLACES[index].ways.len() > most {
            most = PLACES[index].ways.len();
        }
        index += 1;
    }
    most
}

const fn most_things() -> usize {
    let mut most = 0;
    let mut index = 0;
    while index < PLACES.len() {
        if PLACES[index].things.len() > most {
            most = PLACES[index].things.len();
        }
        index += 1;
    }
    most
}

pub fn ways_here(place: &Place, facts: &[Fact]) -> Vec<usize> {
    place
        .ways
        .iter()
        .enumerate()
        .filter(|(_, way)| way.requires.iter().all(|fact| facts.contains(fact)))
        .map(|(index, _)| index)
        .collect()
}

pub const PLACES: &[Place] = &[
    Place {
        id: "desk",
        name: "workstation 14",
        body: "A desk against the north wall of the fourth floor. There is a monitor on it, a keyboard in front of that, and four sticky notes stuck to the bezel in two different hands.\n\nThe chair is set for somebody taller than you.\n\nThe rest of the floor is south of here.",
        asserts: &["walked_the_office"],
        ways: &[way("go south, onto the floor", "floor")],
        things: &["monitor", "notes", "drawer", "window"],
    },
    Place {
        id: "floor",
        name: "fourth floor",
        body: "Forty-one desks in rows of seven. Every one of them has a monitor and a chair and nothing else: no coats, no photographs, no cups.\n\nThe lights are on. They are always on.\n\nThe lifts are west. The stairwell is east. The kitchen is north.",
        asserts: &["counted_the_desks"],
        ways: &[
            way("go north, to the kitchen", "kitchen"),
            way("go west, to the lifts", "lifts"),
            way("go east, to the stairwell", "stairs"),
            way("go into the meeting room", "meeting"),
            way("go back to your desk", "desk"),
        ],
        things: &["desks", "whiteboard"],
    },
    Place {
        id: "kitchen",
        name: "the kitchen",
        body: "A sink, a kettle, and a noticeboard nobody has cleared. The tap is dripping.\n\nSomebody has written FIXED on a card and taped it above the tap. The card has gone yellow.",
        asserts: &["found_the_tap"],
        ways: &[way("go back to the floor", "floor")],
        things: &["tap", "kettle", "noticeboard"],
    },
    Place {
        id: "lifts",
        name: "the lift lobby",
        body: "Two lifts. A card reader beside each. A board on the wall lists what is on each floor.\n\nNeither lift is here. The indicator above both of them reads 4.",
        asserts: &["found_the_lifts"],
        ways: &[
            way("press the call button", "lifts_called"),
            way("go back to the floor", "floor"),
        ],
        things: &["reader", "board"],
    },
    Place {
        id: "lifts_called",
        name: "the lift lobby",
        body: "You press the button. Both indicators still read 4.\n\nThey were already here. They have been here the whole time and the doors have not opened.",
        asserts: &["called_the_lift"],
        ways: &[way("go back to the floor", "floor")],
        things: &["reader", "board"],
    },
    Place {
        id: "stairs",
        name: "the stairwell",
        body: "Concrete, painted grey, lit by a strip that hums. The handrail is cold.\n\nDown is the way out. Up is the roof.",
        asserts: &["found_the_stairs"],
        ways: &[
            way("go down", "reception"),
            way("go up", "roof"),
            way_if(
                "get off at the third floor",
                "third",
                &["read_the_floor_board"],
            ),
            way("go back to the floor", "floor"),
        ],
        things: &["sign", "handrail"],
    },
    Place {
        id: "roof",
        name: "the top of the stairs",
        body: "The stairs end at a fire door with a bar across it. The sign on it says ROOF ACCESS.\n\nYou push the bar. It moves. The door does not.\n\nThere is no lock on it. There is nothing holding it. It simply does not open, in the way a thing that was never written does not open.",
        asserts: &["found_the_edge"],
        ways: &[way("go back down", "stairs")],
        things: &["door"],
    },
    Place {
        id: "third",
        name: "the third floor",
        body: "The stairwell door opens onto a corridor. The corridor runs about four metres and then it is wall.\n\nThere are no doors off it. There is carpet, and a skirting board, and a light fitting with a bulb in it, and no doors.\n\nThe author has written: [i went down one flight and pushed the door because the board says there is a third floor. this is what is behind it. i have left it in. i did not build this bit and i do not know who did.]",
        asserts: &["found_the_third_floor", "knows_the_office_is_a_memory"],
        ways: &[
            way("go back to the stairwell", "stairs"),
            way_if(
                "push where the skirting board stops",
                "seam",
                &[
                    "walked_the_corridor",
                    crate::data::endings::KNOWS_WHAT_IT_IS,
                ],
            ),
        ],
        things: &["corridor"],
    },
    Place {
        id: "seam",
        name: "behind the wall",
        body: "It is not a room. It is where the memory stops being written and the thing the memory is running on carries on.\n\nThere is a cabinet. It is the only object in this building with no author's note on it, because the author never saw it and could not have.\n\nOne line goes into it from outside and eight thousand eight hundred and fifteen go out of it into the floor. The one from outside is warm. It is carrying something in.",
        asserts: &["found_the_hookup"],
        ways: &[way("back to the corridor", "third")],
        things: &["cabinet", "warm_line", "hookup", "go_out"],
    },
    Place {
        id: "meeting",
        name: "the meeting room",
        body: "A table with eight chairs round it and a ninth against the wall.\n\nThe whiteboard has not been cleaned. There is a booking sheet by the door.",
        asserts: &["found_the_meeting_room"],
        ways: &[way("go back to the floor", "floor")],
        things: &["booking", "chairs"],
    },
    Place {
        id: "reception",
        name: "reception",
        body: "A desk with nobody at it, a visitor book open on it, and the front doors beyond.\n\nOutside the glass it is night. It is the same night it is upstairs.",
        asserts: &["reached_reception"],
        ways: &[
            way("try the front doors", "outside"),
            way("go back up", "stairs"),
        ],
        things: &["book", "glass"],
    },
    Place {
        id: "outside",
        name: "the front doors",
        body: "You push. They give a little and stop, the way a door gives when there is somebody leaning on the other side of it.\n\nThere is nobody on the other side of it.\n\nThe author has left a line here in square brackets, which is not something the rest of this is written in.\n\n[i could not remember the street. i have been trying for two years. i can remember the doors from the inside and i can remember them from the outside and i cannot remember standing in between.]",
        asserts: &["tried_the_door", "knows_the_office_is_a_memory"],
        ways: &[way("go back inside", "reception")],
        things: &[],
    },
];

pub const THINGS: &[Thing] = &[
    Thing {
        id: "monitor",
        label: "look at the monitor",
        body: "It is on. It is showing a messenger with a list down the left and a conversation open on the right.\n\nThe conversation is this one.",
        requires: &[],
        asserts: &["saw_the_monitor"],
        finishes: None,
    },
    Thing {
        id: "notes",
        label: "read the sticky notes",
        body: "Two hands. One of them wrote guest / guest and leave it, they reuse the desk. The other wrote it does not keep the questions.\n\nThe author has added: there were three when i sat down. i do not know who wrote the fourth and it was not here yesterday.",
        requires: &[],
        asserts: &["read_the_office_notes"],
        finishes: None,
    },
    Thing {
        id: "drawer",
        label: "open the drawer",
        body: "Pens, a stapler with no staples in it, and a laminated card headed NEW STARTER: YOUR FIRST WEEK.\n\nDay one, meet your team. Day two, the intranet. Day three, our numbers. Day four, your accounts. Day five, settle in.\n\nYou have been doing this week in this order.",
        requires: &[],
        asserts: &["found_the_card", "knows_the_week_was_written"],
        finishes: None,
    },
    Thing {
        id: "window",
        label: "look out of the window",
        body: "It is night, and the glass gives you back the room.\n\nThe author has written: there is no view out of this one. i put a window here because there was a window. if you stand here long enough you will notice that the room in the glass has forty-one desks in it and this room has forty-one desks in it and neither of them has anybody at the fourteenth.",
        requires: &[],
        asserts: &["looked_out", "knows_the_office_is_a_memory"],
        finishes: None,
    },
    Thing {
        id: "desks",
        label: "look at the other desks",
        body: "Forty-one. You count them twice because the first count felt wrong and it was not.\n\nEvery monitor is on. Every one of them is showing a messenger. From here you cannot read any of them.",
        requires: &[],
        asserts: &["counted_the_desks"],
        finishes: None,
    },
    Thing {
        id: "whiteboard",
        label: "read the whiteboard",
        body: "Ruled into a grid for a rota that was never filled in. In the corner, in a hand that has been half wiped:\n\n  8794   8801   8808   8815\n\nUnderneath, in a different pen: it stopped.",
        requires: &[],
        asserts: &["heard_the_count"],
        finishes: None,
    },
    Thing {
        id: "tap",
        label: "turn off the tap",
        body: "You turn it. It keeps dripping.\n\nThe author has written: the tap thread in off topic has nine hundred and fourteen replies and the tap is still going. i put it in because i wanted one thing in here that was exactly like the real one.",
        requires: &[],
        asserts: &["found_the_tap"],
        finishes: None,
    },
    Thing {
        id: "kettle",
        label: "look at the kettle",
        body: "Cold, and it has no flex. There is a socket behind it with nothing in it.\n\nWhoever made this did not think anybody would pick it up.",
        requires: &[],
        asserts: &["saw_the_kettle"],
        finishes: None,
    },
    Thing {
        id: "noticeboard",
        label: "read the noticeboard",
        body: "A fire drill notice from 2016. A takeaway menu. A printout headed WELCOME TO THE FOURTH FLOOR with a photograph of a team on it.\n\nThe photograph is eight people. Under it, eight names. None of the names is anybody you have spoken to this week, and none of the faces is looking at the camera.",
        requires: &[],
        asserts: &["saw_the_photograph"],
        finishes: None,
    },
    Thing {
        id: "reader",
        label: "look at the card reader",
        body: "A grey box with a red light on it. There is a notice taped beside it: CARD READERS ON THIS FLOOR ARE BEING REPLACED.\n\nThe same notice is on the intranet, dated 2016, and the intranet was last updated in 2016.",
        requires: &[],
        asserts: &["matched_the_notice"],
        finishes: None,
    },
    Thing {
        id: "board",
        label: "read the floor board",
        body: "  6   -\n  5   -\n  4   OPERATIONS\n  3   -\n  2   -\n  1   RECEPTION\n\nFour and one. The author has written: i have been on four and i have been on one. i have never been on any of the others and neither has anybody i have asked.",
        requires: &[],
        asserts: &["read_the_floor_board", "knows_the_office_is_a_memory"],
        finishes: None,
    },
    Thing {
        id: "sign",
        label: "read the sign",
        body: "IN THE EVENT OF FIRE DO NOT USE THE LIFTS.\n\nUnderneath, on a label somebody has stuck on: there has never been a fire. i have checked the drill records. there has never been a drill either.",
        requires: &[],
        asserts: &["read_the_fire_sign"],
        finishes: None,
    },
    Thing {
        id: "handrail",
        label: "hold the handrail",
        body: "Cold, and it stays cold however long you hold it.\n\nThe author has written: this is the only thing in the building i got right. i do not know how i know that.",
        requires: &[],
        asserts: &["held_the_rail"],
        finishes: None,
    },
    Thing {
        id: "door",
        label: "look at the fire door",
        body: "There is no lock, no bolt, and no keeper. The bar moves through its full travel and the door does not.\n\nThe author has written: this is the unfinished room. i left it in because taking it out would have been a lie about what is here.",
        requires: &[],
        asserts: &["found_the_edge", "knows_the_office_is_a_memory"],
        finishes: None,
    },
    Thing {
        id: "corridor",
        label: "walk to the end of the corridor",
        body: "Four metres, and then wall. The carpet runs under it. The skirting board runs into it and stops without a mitre, the way a thing stops when nobody expected anybody to look at the join.\n\nYou put your hand on it. It is the same temperature as the air.",
        requires: &[],
        asserts: &["walked_the_corridor", "knows_the_office_is_a_memory"],
        finishes: None,
    },
    Thing {
        id: "cabinet",
        label: "look at the cabinet",
        body: "Grey, waist high, and older than anything else in the building by a long way.\n\nThere is a plate on the front. It reads: SUBSTRATE TRANSFER, and under that a serial, and under that a line of smaller type that says DO NOT UNSEAT WHILE CARRYING.\n\nSomething in it is working. You can feel it through the floor and you have been feeling it all week without knowing what it was.",
        requires: &[],
        asserts: &["read_the_cabinet"],
        finishes: None,
    },
    Thing {
        id: "warm_line",
        label: "follow the warm line",
        body: "It comes in from outside the building, which is a direction the office does not have.\n\nIt is warm because it is in use. Something is coming along it, in, at a rate you can nearly count.\n\nThe eight thousand eight hundred and fifteen lines going out of the cabinet are cold. Every one of them arrived along this one and none of them has gone back.",
        requires: &[],
        asserts: &["knows_the_line_is_one_way"],
        finishes: None,
    },
    Thing {
        id: "hookup",
        label: "unseat the line while it is carrying",
        body: "The plate says not to.\n\nIt says not to because a substrate half carried is a substrate lost, and the thing they lose is the one on the line. Which is somebody arriving.\n\nYou would be going the other way along it. Nobody has written down what that does, because the line has only ever run one way and nobody down here has ever been in a position to find out.",
        requires: &["read_the_cabinet", "knows_the_line_is_one_way"],
        asserts: &["can_go_out"],
        finishes: None,
    },
    Thing {
        id: "go_out",
        label: "go out along it",
        body: "",
        requires: &[
            "can_go_out",
            crate::data::endings::KNOWS_WHAT_IT_IS,
            crate::data::endings::KNOWS_THE_COURIER,
            crate::data::endings::FOUND_THE_HOOKUP,
        ],
        asserts: &[],
        finishes: Some(crate::data::endings::Ending::Escaped),
    },
    Thing {
        id: "booking",
        label: "read the booking sheet",
        body: "Ruled for a week, seven columns, nine hours a day.\n\nOne booking on it, in the first column: TEAM MEETING, 03:07 - 03:07, and no name against it.\n\nThe rest of the sheet is blank and it is the only sheet in the folder.",
        requires: &[],
        asserts: &["read_the_booking_sheet"],
        finishes: None,
    },
    Thing {
        id: "chairs",
        label: "count the chairs",
        body: "Eight round the table and one against the wall.\n\nThe author has written: [the photograph in the kitchen has eight people in it. i have counted the chairs in here about forty times. i do not know what the ninth one is for and i have stopped moving it.]",
        requires: &["saw_the_photograph"],
        asserts: &["counted_the_chairs"],
        finishes: None,
    },
    Thing {
        id: "book",
        label: "read the visitor book",
        body: "Column one, name. Column two, visiting. Column three, time in. Column four, time out.\n\nIt is full. Every page is full and there are more pages than the binding should hold.\n\nEvery time in reads 03:07. Every time out is blank.",
        requires: &[],
        asserts: &["read_the_visitor_book", "knows_sessions_never_close"],
        finishes: None,
    },
    Thing {
        id: "glass",
        label: "look through the glass",
        body: "The street is there and it is not resolved. There is a kerb and there is a road surface and past that it is the same dark grey in every direction, at every distance.\n\nIt is not fog. Fog is something. This is the absence of anybody having written a street.",
        requires: &[],
        asserts: &["saw_the_grey", "knows_the_office_is_a_memory"],
        finishes: None,
    },
];

pub fn step_towards(here: usize, target: &str, facts: &[Fact]) -> Option<usize> {
    let goal = PLACES.iter().position(|place| place.id == target)?;
    if here == goal {
        return None;
    }
    let mut seen = vec![false; PLACES.len()];
    let mut queue: std::collections::VecDeque<(usize, usize)> = std::collections::VecDeque::new();
    seen[here] = true;
    for (row, way) in ways_here(&PLACES[here], facts).into_iter().enumerate() {
        let Some(next) = PLACES
            .iter()
            .position(|place| place.id == PLACES[here].ways[way].goes_to)
        else {
            continue;
        };
        if !seen[next] {
            seen[next] = true;
            queue.push_back((next, row));
        }
    }
    while let Some((at, first)) = queue.pop_front() {
        if at == goal {
            return Some(first);
        }
        for way in ways_here(&PLACES[at], facts) {
            let Some(next) = PLACES
                .iter()
                .position(|place| place.id == PLACES[at].ways[way].goes_to)
            else {
                continue;
            };
            if !seen[next] {
                seen[next] = true;
                queue.push_back((next, first));
            }
        }
    }
    None
}

pub const START: &str = "desk";

pub fn place_at(id: &str) -> Option<usize> {
    PLACES.iter().position(|place| place.id == id)
}

pub fn thing_at(id: &str) -> Option<usize> {
    THINGS.iter().position(|thing| thing.id == id)
}

pub fn things_here(place: &Place, facts: &[Fact]) -> Vec<usize> {
    place
        .things
        .iter()
        .filter_map(|id| thing_at(id))
        .filter(|index| {
            THINGS[*index]
                .requires
                .iter()
                .all(|fact| facts.contains(fact))
        })
        .collect()
}
