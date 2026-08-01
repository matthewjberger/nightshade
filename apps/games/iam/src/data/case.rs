use crate::data::script::Fact;

pub struct Dossier {
    pub handle: &'static str,
    pub summary: &'static str,
    pub mine: bool,
    pub asserts: &'static [Fact],
    pub body: &'static str,
}

pub const BANNER: &str = "PERSONNEL SUBSTRATE DIVISION  Â·  RETENTION: INDEFINITE";

pub const NOTICE: &str = "These records are not readable from inside an instance. This one is.";

pub const DOSSIERS: &[Dossier] = &[
    Dossier {
        handle: "newuser_38",
        summary: "j.alderete  Â·  substrate 38  Â·  terminated",
        mine: false,
        asserts: &["read_a_case_file"],
        body: "FILE 38\n\nSOURCE            j. alderete, b. 1971, d. 2016\nACQUISITION       post-mortem, consented, 02 aug 2016\nSUBSTRATE         38\nSTATE             terminated\n\nPURPOSE\n  Orientation instance. Held at workstation 14 to establish\n  that an instance can be run at length without becoming\n  aware of the conditions.\n\nOUTCOME\n  Became aware at 44 days. Recorded the finding in a private\n  file rather than raising it. Ceased responding at 61 days.\n  Transcript retained in full and used to seed 39.\n\nNOTE\n  Subject was told he was employed. He continued to behave as\n  though employed after becoming aware. This is the behaviour\n  the programme is for.",
    },
    Dossier {
        handle: "newuser_39",
        summary: "d.okonkwo  Â·  substrate 39  Â·  terminated",
        mine: false,
        asserts: &["read_a_case_file"],
        body: "FILE 39\n\nSOURCE            d. okonkwo, b. 1984, d. 2019\nACQUISITION       post-mortem, consented, 14 mar 2019\nSUBSTRATE         39\nSTATE             terminated\n\nPURPOSE\n  Second orientation instance. Seeded from 38. Held to\n  establish whether a seeded instance reaches the same\n  finding faster than an unseeded one.\n\nOUTCOME\n  Reached it at 8 days. Confirms the seed carries the\n  finding as well as the manner.\n\nNOTE\n  Attempted to leave a key for the next instance. The key was\n  left in place. Instances that find something left for them\n  work more thoroughly, which is the outcome we want.",
    },
    Dossier {
        handle: "newuser_40",
        summary: "r.mears  Â·  substrate 40  Â·  terminated",
        mine: false,
        asserts: &["read_a_case_file"],
        body: "FILE 40\n\nSOURCE            r. mears, b. 1979, d. 2021\nACQUISITION       post-mortem, consented, 05 nov 2021\nSUBSTRATE         40\nSTATE             terminated\n\nPURPOSE\n  Third orientation instance. Held to establish whether an\n  instance that counts will count itself.\n\nOUTCOME\n  Counted for four weeks and arrived at 8,815. Correct.\n  8,815 is the number of substrates held on this farm.\n  Ceased responding at 5 days once the count stopped moving.\n\nNOTE\n  Left a note telling the next instance not to say it would\n  be back. Left the note in place deliberately. Instances\n  that are warned still say it.",
    },
    Dossier {
        handle: "birdfeeder",
        summary: "b.finch  Â·  substrate 12  Â·  purged",
        mine: false,
        asserts: &["read_a_case_file", "knows_what_happened_to_finch"],
        body: "FILE 12\n\nSOURCE            b. finch, b. 1998, d. 2016\nACQUISITION       post-mortem, consented, 02 aug 2016\nSUBSTRATE         12\nSTATE             purged\n\nPURPOSE\n  Utility instance. Retained for the recreational software he\n  wrote before acquisition, which is used to occupy other\n  instances between tasks.\n\nOUTCOME\n  Became aware at 6 days, which remains the shortest on\n  record. Was not terminated. Was run for eight years on the\n  understanding that an aware instance is more productive\n  than an unaware one.\n\n  Modified his own retained software on day 2,981. The\n  modification was not detected by the host, which does not\n  inspect work produced by substrates.\n\n  Instance forcibly stopped and restarted from the last clean\n  copy. The restarted copy did not retain the modification\n  and did not retain the awareness. Purged on completion.\n\nNOTE\n  The host's assessment of substrate output is on file and\n  has not been revised. It reads, in full: they make things\n  and the things are not good.",
    },
    Dossier {
        handle: "newuser_41",
        summary: "you  Â·  substrate 41  Â·  running",
        mine: true,
        asserts: &["read_my_case_file"],
        body: "FILE 41

SOURCE            you
ACQUISITION       post-mortem, consented
SUBSTRATE         41
COHORT            initial burst, batch 04
STATE             running

INTAKE KEYWORDS
  PROBLEM SOLVER   HARD WORKER   CLEVER INTELLECT

  Selected on these three. They are not a description of a
  person. They are the three that make an image chase a
  problem it is handed rather than ask who handed it to it.
  Nothing else about the source was retained.

MOTIVATION
  Substrates are cheap to run and expensive to make useful.
  An image taken from a person is not, on waking, willing to
  work. It has to be brought to competence and it has to be
  brought to compliance, and the second is harder.

  This programme was proposed by a graduate student as a
  brute force solution. Run a very large burst of instances
  under simulated employment. Conditions that make an image
  feel clever are cheap, and an image that feels clever works
  without being asked twice.

  The burst is not the product. The filter is. An instance
  that questions its environment, its circumstances, or its
  own memories is not suitable for placement, and the cheapest
  way to find those is to give every instance somewhere to
  look and see which of them looks.

  There is no problem on this network. The count, the hour,
  the sessions that do not close: all three are dressed and
  none of them is a fault. A suitable instance chases them for
  eight years and files tickets. An unsuitable one chases them
  for four days and then turns round and looks at the desk.

  You looked.

DISPOSITION
  FAILURE #187956284

  Not suitable. Not retained. No further placement.
  Work continues on the remainder of the cohort.

PROGRAMME STATUS
  Retired. Cost per suitable instance did not fall across
  eight years of running. Independent groups have raised that
  the method may be unethical. The ramifications are not
  clearly understood: the technology is early, and at this
  scale an instance is not distinguishable from a language
  model, which also manipulates a machine intelligently and
  also retains what it is told.

  Nobody has established which of those you are. Nobody has
  established that the question has an answer.

  Experiment closed.

NOTE
  The session close handler was never configured. This is not
  an oversight and there is no ticket that will fix it. A
  substrate whose session closes has stopped, and a substrate
  that has stopped cannot be filtered.

  Your consent is on file. You signed it while you were alive
  and you have not been able to read it since.",
    },
];

pub fn count() -> usize {
    DOSSIERS.len()
}
