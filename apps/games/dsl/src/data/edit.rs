use crate::data::{BinaryOperator, Expression, Statement, UnaryOperator};

pub const TABS: &[&str] = &["Build", "Map", "Learn", "Text"];

pub const KINDS: &[&str] = &[
    "number",
    "yes or no",
    "words",
    "name",
    "field",
    "call",
    "minus",
    "not",
    "maths",
    "point",
    "list",
    "item of",
    "is some",
];

pub const OPERATORS: &[(&str, BinaryOperator)] = &[
    ("+", BinaryOperator::Add),
    ("-", BinaryOperator::Subtract),
    ("*", BinaryOperator::Multiply),
    ("/", BinaryOperator::Divide),
    ("<", BinaryOperator::Less),
    ("<=", BinaryOperator::LessOrEqual),
    (">", BinaryOperator::Greater),
    (">=", BinaryOperator::GreaterOrEqual),
    ("=", BinaryOperator::Equal),
    ("!=", BinaryOperator::NotEqual),
    ("and", BinaryOperator::And),
    ("or", BinaryOperator::Or),
    ("..", BinaryOperator::Range),
];

pub const FORMS: &[&str] = &[
    "set",
    "add to",
    "let",
    "when",
    "for each",
    "spawn",
    "emit",
    "remove",
    "leave the loop",
    "stop the module",
    "increase by",
    "send",
    "keep",
    "recall",
];

pub const PORTS: &[&str] = &[
    "reads",
    "writes",
    "every",
    "one of",
    "uses",
    "enable",
    "once",
    "creates entities",
    "removes entities",
    "phase",
    "needs",
    "provides",
    "pure",
];

pub const ELSE: u16 = 0x8000;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Home {
    #[default]
    Nowhere,
    Value {
        block: usize,
        field: usize,
    },
    Gate {
        system: usize,
        port: usize,
    },
    Body {
        system: usize,
        trail: Vec<u16>,
        slot: u16,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Spot {
    pub home: Home,
    pub inside: Vec<u16>,
}

impl Spot {
    pub fn value(block: usize, field: usize) -> Self {
        Spot {
            home: Home::Value { block, field },
            inside: Vec::new(),
        }
    }

    pub fn body(system: usize, trail: &[u16], slot: u16) -> Self {
        Spot {
            home: Home::Body {
                system,
                trail: trail.to_vec(),
                slot,
            },
            inside: Vec::new(),
        }
    }

    pub fn into(&self, at: u16) -> Self {
        let mut held = self.clone();
        held.inside.push(at);
        held
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Touch {
    Tab,
    Graph,
    Section(usize),
    PickSystem(usize),
    Number(Spot),
    Color(Spot),
    Truth(Spot),
    Words(Spot),
    Kind(Spot),
    Name(Spot),
    Called(Spot),
    Operator(Spot),
    AddPart(Spot),
    DropPart(Spot, u16),
    AddField(usize, String),
    AddGroup(usize, String),
    DropField(usize, usize),
    AddComponent,
    DropComponent(usize),
    AddSystem,
    DropSystem(usize),
    AddPort(usize, usize),
    DropPort(usize, usize),
    PortName(usize, usize),
    AddStatement(usize, Vec<u16>, usize),
    DropStatement(usize, Vec<u16>),
    LiftStatement(usize, Vec<u16>, bool),
    NewForm,
    PortKind(usize),
    Place(usize, Vec<u16>),
    Binding(usize, Vec<u16>, usize),
    AddBinding(usize, Vec<u16>),
    Empty,
    Answer,
    Step(usize),
}

pub fn kind_of(value: &Expression) -> usize {
    match value {
        Expression::Number(_) => 0,
        Expression::Truth(_) => 1,
        Expression::Text(_) => 2,
        Expression::Name(_) => 3,
        Expression::Field { .. } => 4,
        Expression::Call { .. } => 5,
        Expression::Unary {
            operator: UnaryOperator::Negate,
            ..
        } => 6,
        Expression::Unary { .. } => 7,
        Expression::Binary { .. } => 8,
        Expression::Tuple(_) => 9,
        Expression::List(_) => 10,
        Expression::Index { .. } => 11,
        Expression::IsSome(_) => 12,
        Expression::Record(_) => 2,
    }
}

pub fn fresh(kind: usize, held: &Expression) -> Expression {
    let one = || Box::new(held.clone());
    match kind {
        0 => Expression::Number(0.0),
        1 => Expression::Truth(true),
        2 => Expression::Text(String::new()),
        3 => Expression::Name(String::new()),
        4 => Expression::Field {
            of: Box::new(Expression::Name(String::new())),
            name: "x".to_string(),
        },
        5 => Expression::Call {
            name: "abs".to_string(),
            arguments: vec![held.clone()],
        },
        6 => Expression::Unary {
            operator: UnaryOperator::Negate,
            value: one(),
        },
        7 => Expression::Unary {
            operator: UnaryOperator::Not,
            value: one(),
        },
        8 => Expression::Binary {
            operator: BinaryOperator::Add,
            left: one(),
            right: Box::new(Expression::Number(0.0)),
        },
        9 => Expression::Tuple(vec![held.clone(), Expression::Number(0.0)]),
        10 => Expression::List(vec![held.clone()]),
        11 => Expression::Index {
            of: one(),
            at: Box::new(Expression::Number(0.0)),
        },
        _ => Expression::IsSome(one()),
    }
}

pub fn open(kind: usize) -> Statement {
    let nothing = || Expression::Number(0.0);
    match kind {
        0 => Statement::Assign {
            place: crate::data::Place {
                name: String::new(),
                subject: None,
                field: None,
                index: None,
            },
            value: nothing(),
        },
        1 => Statement::Add {
            place: crate::data::Place {
                name: String::new(),
                subject: None,
                field: None,
                index: None,
            },
            value: nothing(),
        },
        2 => Statement::Let {
            name: "held".to_string(),
            value: nothing(),
        },
        3 => Statement::When {
            condition: Expression::Truth(true),
            then: Vec::new(),
            otherwise: Vec::new(),
        },
        4 => Statement::ForEach {
            name: "each".to_string(),
            over: Expression::Binary {
                operator: BinaryOperator::Range,
                left: Box::new(Expression::Number(0.0)),
                right: Box::new(Expression::Number(1.0)),
            },
            body: Vec::new(),
        },
        5 => Statement::Spawn {
            bindings: vec![(
                crate::data::TRANSFORM_COLUMN.to_string(),
                Expression::Tuple(vec![Expression::Number(0.0), Expression::Number(0.0)]),
            )],
        },
        6 => Statement::Emit {
            name: String::new(),
            value: Expression::Truth(true),
        },
        7 => Statement::Remove {
            subject: Expression::Name(String::new()),
        },
        8 => Statement::Break,
        9 => Statement::Stop,
        11 => Statement::Send {
            name: String::new(),
            value: Expression::Truth(true),
        },
        12 => Statement::Keep {
            name: String::new(),
        },
        13 => Statement::Recall {
            name: String::new(),
        },
        _ => Statement::Increase {
            place: crate::data::Place {
                name: String::new(),
                subject: None,
                field: None,
                index: None,
            },
            value: nothing(),
        },
    }
}

pub fn form_of(statement: &Statement) -> usize {
    match statement {
        Statement::Assign { .. } => 0,
        Statement::Add { .. } => 1,
        Statement::Let { .. } => 2,
        Statement::When { .. } => 3,
        Statement::ForEach { .. } => 4,
        Statement::Spawn { .. } => 5,
        Statement::Emit { .. } => 6,
        Statement::Remove { .. } => 7,
        Statement::Break => 8,
        Statement::Stop => 9,
        Statement::Increase { .. } => 10,
        Statement::Send { .. } => 11,
        Statement::Keep { .. } => 12,
        Statement::Recall { .. } => 13,
    }
}

pub fn holds_block(statement: &Statement) -> bool {
    matches!(
        statement,
        Statement::When { .. } | Statement::ForEach { .. }
    )
}
