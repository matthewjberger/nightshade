pub struct NoteSite {
    pub col: i32,
    pub row: i32,
    pub upper: bool,
    pub index: usize,
}

const fn site(col: i32, row: i32, upper: bool, index: usize) -> NoteSite {
    NoteSite {
        col,
        row,
        upper,
        index,
    }
}

pub const NOTE_SITES: &[NoteSite] = &[
    site(24, 34, false, 0),
    site(5, 26, true, 1),
    site(26, 30, false, 2),
    site(36, 16, false, 3),
    site(44, 20, false, 4),
    site(36, 26, false, 5),
    site(44, 26, false, 6),
    site(51, 29, false, 7),
    site(14, 44, false, 8),
    site(20, 3, false, 9),
    site(26, 6, false, 10),
    site(5, 4, false, 11),
    site(24, 11, false, 12),
    site(22, 10, true, 13),
    site(38, 3, false, 14),
    site(59, 11, false, 15),
];

pub const NOTES: [(&str, &str); 16] = [
    (
        "Your own handwriting",
        "You wrote this in the car before the fog came up to the windows. It says: the road out is closed, walk to the church, do not stop at the school. You do not remember writing it. The ink is still wet.",
    ),
    (
        "Duty log, torn at the spine",
        "Third shift. Fog came in off the reservoir at four and did not lift. Radios give static in a pattern. Not weather. A pattern. Hendry says it gets loud when they get close and I told him to go home. He walked out the front door and I have not seen him since.",
    ),
    (
        "Evidence tag 114",
        "One brass key, taken from the coat of a man found on Mill Road with no shoes. It is in the wheel case and the wheels are set to the number at the top of this tag. He would not say his name. He said the same eleven words for six hours and then he stopped breathing. I will not write them here.",
    ),
    (
        "Municipal notice, nailed to a post",
        "By order of the town board the reservoir works are closed and the water is not to be drunk. Ash in the water is not harmful. Ash in the water is not harmful. Ash in the water is not harmful. Someone has printed the same line down the whole page.",
    ),
    (
        "Clinic intake sheet",
        "Nineteen came in on the first night with burns on the hands and no fire to account for them. All nineteen asked for the same doctor. We do not have a doctor by that name. We have never had a doctor by that name.",
    ),
    (
        "Nurse's note, folded small",
        "Ward eleven is not on the plan of this building. I have walked the corridor and counted the doors and there are eleven and there should be ten. I am not going to open it. Whoever reads this, the ward key is in the pharmacy and I am sorry that I am leaving it there.",
    ),
    (
        "Prescription pad, one sheet used",
        "For the patient in eleven. Do not treat the burns. Do not turn on the lamp. Do not answer if it uses your voice. Signed with a name that has been scratched through so hard the paper has torn.",
    ),
    (
        "Letter that was never sent",
        "Mum, the town has gone quiet in a way that is not quiet. The neighbours are still here. They stand in their rooms with the lights off. When I knocked they turned to face the wall. I am going to walk to the church tomorrow with the Merrick boy. If you get this, do not come and find me.",
    ),
    (
        "Child's drawing, in crayon",
        "A house, a garden, and a tall grey person in every window. On the back, in a careful hand: mummy says the tall ones are the neighbours and they are only sleeping. i counted them. there are more of them than there are houses.",
    ),
    (
        "Attendance register",
        "Every name in the class has been ticked present for eleven years. The last teacher to sign it added a line at the bottom: the bell rings itself at four and the children come in from the fog. I let them in the first year. I will not do it again.",
    ),
    (
        "Report card, never sent home",
        "The child draws the same figure in every lesson. Tall, grey, arms too long, no face. When asked, the child says it is the school. Not that it is in the school. That it is the school. Recommend the family leave the town.",
    ),
    (
        "Library card, one book listed",
        "The Ashfall Rite, borrowed eleven years ago and never returned. A pencil note in the margin of the card reads: the bone key opens the church and the church opens the rest. They broke it in two so that nobody could open it on their own. Put it back together and the town turns over. Be at the gate before it does.",
    ),
    (
        "Staff room notice",
        "The fire in the east wing was not an accident and it was not eleven years ago. It is happening. Go and stand in the corridor at four and you will hear it happening. Do not go and stand in the corridor at four.",
    ),
    (
        "Caretaker's list, in pencil",
        "Floor gave way over the hall in the winter. I have roped it off. Below the hole I found the shoes, all of them in pairs and all of them facing the same way, and I put them back exactly as I found them because I am not a fool.",
    ),
    (
        "Parish record, last entry",
        "Father Aldous took the bone key into the school on the fourth night to bring the children out. He came back at four in the morning with the key snapped through and without the children, and he would not come in past the door. He left the haft up in the reading gallery and the teeth here in the vestry, and he said the town is inside out now and we are the lining.",
    ),
    (
        "Headstone rubbing, folded twice",
        "The name on the stone is your name. The date cut under it is today. Someone has scratched a line beneath the date and written: this is not a warning, it is a receipt. The stone is old. The scratch is new.",
    ),
];
