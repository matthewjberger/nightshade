use crate::data::script::Fact;

pub struct Sticky {
    pub author: &'static str,
    pub text: &'static str,
    pub requires: &'static [Fact],
}

pub const NOTES: &[Sticky] = &[
    Sticky {
        author: "r.mears",
        text: "guest / guest
leave it, they reuse the desk",
        requires: &[],
    },
    Sticky {
        author: "r.mears",
        text: "IT will not reimage this. asked twice.
do not put anything on here you want gone",
        requires: &[],
    },
    Sticky {
        author: "r.mears",
        text: "iam signs you up on your start date
before you get here
ask d how it knows",
        requires: &["heard_the_hour"],
    },
    Sticky {
        author: "d.okonkwo",
        text: "it does not know. it is told.
same form as payroll.
nobody reads the other end",
        requires: &["knows_the_form_is_constant"],
    },
    Sticky {
        author: "r.mears",
        text: "counted the room again. 8815.
it says 8814 when you ask it.
it takes one off every time",
        requires: &["knows_the_count_was_wrong"],
    },
    Sticky {
        author: "d.okonkwo",
        text: "rot13 my handle if you ever need in
not clever, but it is not written down
anywhere either",
        requires: &["knows_the_name_is_rotated"],
    },
    Sticky {
        author: "j.alderete",
        text: "sundial posted eleven lines about this
the server kept four
i wrote the number on my own login",
        requires: &["saw_a_missing_line"],
    },
    Sticky {
        author: "d.okonkwo",
        text: "the close column is not broken
there is a ticket for it from 2016
nobody has ever been assigned it",
        requires: &["knows_sessions_never_close"],
    },
    Sticky {
        author: "j.alderete",
        text: "it does not keep the questions
i tested it for a month
it keeps everything else",
        requires: &["knows_why_questions_go"],
    },
    Sticky {
        author: "r.mears",
        text: "there is always one account open
with nobody in it
i think that is the point of it",
        requires: &["knows_the_empty_account"],
    },
    Sticky {
        author: "d.okonkwo",
        text: "do not sign off saying you will be back
say nothing at all
it writes down what you say",
        requires: &["knows_it_copies_you"],
    },
    Sticky {
        author: "r.mears",
        text: "if you are reading this i did not come back
that is not the same as leaving
check the offline list",
        requires: &["saw_own_session"],
    },
];

pub fn readable(note: &Sticky, facts: &[Fact]) -> bool {
    note.requires.iter().all(|fact| facts.contains(fact))
}
