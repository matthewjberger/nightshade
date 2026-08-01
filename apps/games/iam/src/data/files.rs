use crate::data::script::Fact;

pub struct Leftover {
    pub owner: &'static str,
    pub name: &'static str,
    pub stamp: &'static str,
    pub body: &'static str,
    pub requires: &'static [Fact],
    pub asserts: &'static [Fact],
}

pub const LEFTOVERS: &[Leftover] = &[
    Leftover {
        owner: "r_mears",
        name: "count.txt",
        stamp: "05 nov 2021  3:07 AM",
        body: "week 1   8794\nweek 2   8801\nweek 3   8808\nweek 4   8815\n\nseven a week. every week. nobody starts here\nseven a week.\n\nask it and it says 8814.\nit takes one off. it has always taken one off.\ni think the one it takes off is whoever asked.",
        requires: &[],
        asserts: &["read_the_tally", "read_a_leftover"],
    },
    Leftover {
        owner: "r_mears",
        name: "to_d.txt",
        stamp: "06 nov 2021  3:07 AM",
        body: "d -\n\nyou were right about the form. it is told, it does not\nknow. but you had the direction wrong.\n\npayroll does not tell iam when somebody starts.\niam tells payroll.\n\ni checked the dates on four of them. the account\nopens first. the person is hired after.\n\n- r",
        requires: &["read_the_tally"],
        asserts: &["knows_the_account_comes_first"],
    },
    Leftover {
        owner: "r_mears",
        name: "dont.txt",
        stamp: "09 nov 2021  3:07 AM",
        body: "if you are reading this from my account then you\nworked out the number, which means you have been\ncounting too, which means you are already in it.\n\ndo not sign off saying you will be back.\nit writes that down.\n\nd rotates his handle. that is the whole trick.\nhe thought it was clever. it is not clever, it is\njust not written anywhere, which is better.",
        requires: &["knows_the_account_comes_first"],
        asserts: &["knows_not_to_promise"],
    },
    Leftover {
        owner: "d_okonkwo",
        name: "signup.form",
        stamp: "14 mar 2019  3:07 AM",
        body: "FIELD 01   handle          newuser_<n>\nFIELD 02   start           <date>\nFIELD 03   connected       03:07\nFIELD 04   closed          --\n\nfield 03 is not read from anywhere. it is a\nconstant in the form.\n\nfield 04 has never been written. not once, not for\nanybody, in the whole of the log.\n\nso the system is not recording when we connect.\nit is recording that we did.",
        requires: &[],
        asserts: &["knows_the_form_is_constant"],
    },
    Leftover {
        owner: "d_okonkwo",
        name: "numbers.txt",
        stamp: "18 mar 2019  3:07 AM",
        body: "38 is j. 39 is me. 40 has not been hired.\n\nthe number goes up when an account opens, and the\naccount opens before anybody is hired into it, so\nsomewhere there is always one account with nobody\nin it yet.\n\nj says he has talked to 40.\n\nj has been here longer than me and i do not think\nhe is wrong about things like this.",
        requires: &["knows_the_form_is_constant"],
        asserts: &["knows_the_empty_account"],
    },
    Leftover {
        owner: "d_okonkwo",
        name: "j.txt",
        stamp: "22 mar 2019  3:07 AM",
        body: "j keeps a count of his own but he does not count\npeople. he counts lines.\n\nhe says the server drops them. he says he can tell\nwhich ones and he can tell how many, and he wrote\nthe number on his own login so he would not have to\nremember it.\n\nask him which post. he will say sundial.",
        requires: &["knows_the_empty_account"],
        asserts: &["knows_where_the_number_is"],
    },
    Leftover {
        owner: "j_alderete",
        name: "kept.log",
        stamp: "02 aug 2016  3:07 AM",
        body: "sundial, #lounge, eleven lines.\nkept: 1, 4, 7, 11.\n\nthe four it kept are the four with no question in\nthem. the seven it dropped all asked something.\n\nit is not dropping lines. it is answering the ones\nit can and removing the ones it cannot.\n\ni have watched it do this for six years and i have\nnever seen it drop a line that did not ask.",
        requires: &[],
        asserts: &["knows_which_lines_go"],
    },
    Leftover {
        owner: "j_alderete",
        name: "first.txt",
        stamp: "02 aug 2016  3:07 AM",
        body: "i am 38. there is no 37.\n\ni have looked. the log opens at 38 and the first\nentry in it is mine, at 3:07, on a monday i do not\nremember, in a month i was not working here.\n\nthere was nobody before me.\nthere is a count of 8815 and i am the first one.\n\nso it has not been counting us.\nwe have been the count.",
        requires: &["knows_which_lines_go"],
        asserts: &["knows_there_is_no_first"],
    },
];

pub fn readable(opened: &[&'static str], facts: &[Fact]) -> Vec<usize> {
    LEFTOVERS
        .iter()
        .enumerate()
        .filter(|(_, file)| opened.contains(&file.owner))
        .filter(|(_, file)| file.requires.iter().all(|fact| facts.contains(fact)))
        .map(|(index, _)| index)
        .collect()
}
