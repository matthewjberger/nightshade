use crate::data::{
    Asset, BinaryOperator, Cardinality, Column, Declaration, Expression, Field, Place, Port,
    Program, Selection, Statement, System, Token, UnaryOperator,
};

pub fn read(source: &str) -> Result<Program, String> {
    let tokens = super::lex::scan(source)?;
    let mut reader = Reader { tokens, at: 0 };
    let mut program = Program::default();

    loop {
        reader.skip_newlines();
        if reader.done() {
            break;
        }
        match reader.declaration()? {
            Declaration::Component { name, kind } => {
                if let Some(kind) = kind {
                    program.shapes.push((name.clone(), kind));
                }
                program.components.push(name);
            }
            Declaration::Resource { name, fields } => program.resources.push((name, fields)),
            Declaration::Config { name, fields } => program.configs.push((name, fields)),
            Declaration::Event { name, fields } => {
                if !fields.is_empty() {
                    let named = fields.iter().map(|field| field.name.clone()).collect();
                    program.payloads.push((name.clone(), named));
                }
                program.events.push(name);
            }
            Declaration::Action { name, key, held } => program.actions.push((name, key, held)),
            Declaration::Command { name, fields } => {
                if !fields.is_empty() {
                    let named = fields.iter().map(|field| field.name.clone()).collect();
                    program.payloads.push((name.clone(), named));
                }
                program.commands.push(name.clone());
                program.events.push(name);
            }
            Declaration::Query {
                name,
                parameters,
                body,
            } => program.queries.push((name, parameters, body)),
            Declaration::System(system) => program.systems.push(system),
            Declaration::Order { earlier, later } => program.orders.push((earlier, later)),
            Declaration::Kind { variants } => program.variants.extend(variants),
            Declaration::Use { file, sets, runs } => {
                for (target, value) in sets {
                    program.sets.push((file.clone(), target, value));
                }
                if runs {
                    program.runs.push(file.clone());
                }
                program.uses.push(file);
            }
            Declaration::Module { name } => program.module = name,
            Declaration::Pulse { name } => {
                program.pulses.push(name.clone());
                program.components.push(name);
            }
            Declaration::Assets {
                name,
                source,
                root,
                rows,
            } => program.assets.push((name, source, root, rows)),
            Declaration::Shows { names } => program.shows.extend(names),
        }
    }
    Ok(program)
}

struct Reader {
    tokens: Vec<Token>,
    at: usize,
}

impl Reader {
    fn done(&self) -> bool {
        self.at >= self.tokens.len()
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.at)
    }

    fn take(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.at).cloned();
        if token.is_some() {
            self.at += 1;
        }
        token
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek(), Some(Token::Newline)) {
            self.at += 1;
        }
    }

    fn at_word(&self, wanted: &str) -> bool {
        matches!(self.peek(), Some(Token::Word(word)) if word == wanted)
    }

    fn at_symbol(&self, wanted: &str) -> bool {
        matches!(self.peek(), Some(Token::Symbol(symbol)) if symbol == wanted)
    }

    fn accept_word(&mut self, wanted: &str) -> bool {
        if self.at_word(wanted) {
            self.at += 1;
            return true;
        }
        false
    }

    fn accept_symbol(&mut self, wanted: &str) -> bool {
        if self.at_symbol(wanted) {
            self.at += 1;
            return true;
        }
        false
    }

    fn expect_symbol(&mut self, wanted: &str) -> Result<(), String> {
        if self.accept_symbol(wanted) {
            return Ok(());
        }
        Err(format!("expected {wanted} after {}", self.recent()))
    }

    fn word(&mut self) -> Result<String, String> {
        match self.take() {
            Some(Token::Word(word)) => Ok(word),
            _ => Err(format!("expected a name after {}", self.recent())),
        }
    }

    fn recent(&self) -> String {
        let from = self.at.saturating_sub(7);
        self.tokens[from..self.at.min(self.tokens.len())]
            .iter()
            .map(|token| match token {
                Token::Word(word) => word.clone(),
                Token::Number(value) => value.to_string(),
                Token::Text(value) => format!("\"{value}\""),
                Token::Symbol(symbol) => symbol.clone(),
                Token::Begin => "<in>".to_string(),
                Token::End => "<out>".to_string(),
                Token::Newline => "/".to_string(),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn dotted(&mut self) -> Result<String, String> {
        let mut name = self.word()?;
        while self.at_symbol(".") {
            self.at += 1;
            name.push('.');
            name.push_str(&self.word()?);
        }
        Ok(name)
    }

    fn end_of_line(&mut self) -> Result<(), String> {
        while !matches!(self.peek(), Some(Token::Newline) | Some(Token::End) | None) {
            self.at += 1;
        }
        self.skip_newlines();
        Ok(())
    }

    fn open_block(&mut self) -> Result<(), String> {
        self.skip_newlines();
        if matches!(self.peek(), Some(Token::Begin)) {
            self.at += 1;
            return Ok(());
        }
        Err(format!(
            "expected an indented block, found {:?}",
            self.peek()
        ))
    }

    fn at_block_end(&mut self) -> bool {
        self.skip_newlines();
        matches!(self.peek(), Some(Token::End) | None)
    }

    fn close_block(&mut self) {
        if matches!(self.peek(), Some(Token::End)) {
            self.at += 1;
        }
    }

    fn declaration(&mut self) -> Result<Declaration, String> {
        if self.accept_word("shows") {
            let names = self.name_list()?;
            self.end_of_line()?;
            return Ok(Declaration::Shows { names });
        }
        if self.accept_word("module") {
            let name = self.dotted()?;
            self.end_of_line()?;
            return Ok(Declaration::Module { name });
        }
        if self.at_word("use") || self.at_word("runs") {
            let runs = self.accept_word("runs");
            if !runs {
                let _ = self.accept_word("use");
            }
            let file = match self.take() {
                Some(Token::Text(file)) => file,
                other => return Err(format!("expected a file name, found {other:?}")),
            };
            self.end_of_line()?;
            let mut sets = Vec::new();
            if matches!(self.peek(), Some(Token::Begin)) {
                self.open_block()?;
                while !self.at_block_end() {
                    let target = self.dotted()?;
                    self.expect_symbol("=")?;
                    let value = self.expression()?;
                    sets.push((target, value));
                    self.end_of_line()?;
                }
                self.close_block();
            }
            return Ok(Declaration::Use { file, sets, runs });
        }
        if self.accept_word("type") {
            let _ = self.word()?;
            self.expect_symbol("=")?;
            let mut variants = vec![self.word()?];
            while self.accept_symbol("|") {
                variants.push(self.word()?);
            }
            self.end_of_line()?;
            return Ok(Declaration::Kind { variants });
        }
        if self.accept_word("pulse") {
            let name = self.word()?;
            self.end_of_line()?;
            return Ok(Declaration::Pulse { name });
        }
        if self.accept_word("assets") {
            let name = self.word()?;
            let _ = self.accept_word("from");
            let source = self.word()?;
            let root = match self.peek() {
                Some(Token::Text(_)) => match self.take() {
                    Some(Token::Text(held)) => held,
                    _ => String::new(),
                },
                _ => String::new(),
            };
            self.end_of_line()?;
            let mut rows = Vec::new();
            self.open_block()?;
            while !self.at_block_end() {
                let label = self.word()?;
                let kind = self.word()?;
                let file = match self.take() {
                    Some(Token::Text(file)) => file,
                    _ => return Err(format!("expected a file after {}", self.recent())),
                };
                rows.push(Asset { label, kind, file });
                self.end_of_line()?;
            }
            self.close_block();
            return Ok(Declaration::Assets {
                name,
                source,
                root,
                rows,
            });
        }
        if self.accept_word("component") {
            let name = self.word()?;
            let kind = self.shape();
            self.end_of_line()?;
            return Ok(Declaration::Component { name, kind });
        }
        if self.accept_word("config") {
            let name = self.word()?;
            self.end_of_line()?;
            let fields = self.record()?;
            return Ok(Declaration::Config { name, fields });
        }
        if self.accept_word("resource") {
            let name = self.word()?;
            self.end_of_line()?;
            let fields = self.record()?;
            return Ok(Declaration::Resource { name, fields });
        }
        if self.accept_word("event") {
            let name = self.word()?;
            self.end_of_line()?;
            let fields = self.payload()?;
            return Ok(Declaration::Event { name, fields });
        }
        if self.accept_word("action") {
            let name = self.word()?;
            self.expect_symbol("=")?;
            let key = match self.take() {
                Some(Token::Text(key)) => key,
                other => return Err(format!("expected a key name, found {other:?}")),
            };
            let held = self.accept_word("held");
            self.end_of_line()?;
            return Ok(Declaration::Action { name, key, held });
        }
        if self.accept_word("command") {
            let name = self.word()?;
            self.end_of_line()?;
            let fields = self.payload()?;
            return Ok(Declaration::Command { name, fields });
        }
        if self.accept_word("query") {
            let name = self.word()?;
            let mut parameters = Vec::new();
            self.expect_symbol("(")?;
            while !self.accept_symbol(")") {
                parameters.push(self.word()?);
                let _ = self.accept_symbol(",");
            }
            if self.accept_symbol("->") {
                while !self.at_symbol("=") {
                    self.at += 1;
                }
            }
            self.expect_symbol("=")?;
            let body = self.expression()?;
            self.end_of_line()?;
            return Ok(Declaration::Query {
                name,
                parameters,
                body,
            });
        }
        if self.accept_word("order") {
            let earlier = self.dotted()?;
            let _ = self.accept_word("before");
            let later = self.dotted()?;
            self.end_of_line()?;
            return Ok(Declaration::Order { earlier, later });
        }
        if self.accept_word("system") {
            let name = self.dotted()?;
            self.end_of_line()?;
            let mut ports = Vec::new();
            let mut body = Vec::new();
            self.open_block()?;
            while !self.at_block_end() {
                if self.at_port() {
                    ports.push(self.port()?);
                } else {
                    body.push(self.statement()?);
                }
                self.skip_newlines();
            }
            self.close_block();
            return Ok(Declaration::System(System {
                name,
                module: String::new(),
                ports,
                body,
            }));
        }

        Err(format!("expected a declaration, found {:?}", self.peek()))
    }

    fn shape(&mut self) -> Option<String> {
        let held = match self.peek() {
            Some(Token::Word(word)) if crate::data::form::form_of(word).is_some() => word.clone(),
            _ => return None,
        };
        self.at += 1;
        Some(held)
    }

    fn payload(&mut self) -> Result<Vec<Field>, String> {
        if !matches!(self.peek(), Some(Token::Begin)) {
            return Ok(Vec::new());
        }
        self.record()
    }

    fn record(&mut self) -> Result<Vec<Field>, String> {
        let mut fields = Vec::new();
        self.open_block()?;
        while !self.at_block_end() {
            let name = self.word()?;
            let mut kind = self.shape();
            let mut default = None;
            while !matches!(self.peek(), Some(Token::Newline) | Some(Token::End) | None) {
                if self.accept_symbol("=") {
                    default = Some(self.expression()?);
                } else {
                    if kind.is_none()
                        && let Some(held) = self.shape()
                    {
                        kind = Some(held);
                        continue;
                    }
                    self.at += 1;
                }
            }
            if default.is_none() && self.at_group() {
                for held in self.record()? {
                    fields.push(Field {
                        name: format!("{name}.{}", held.name),
                        kind: held.kind,
                        default: held.default,
                    });
                }
                self.skip_newlines();
                continue;
            }
            fields.push(Field {
                name,
                kind,
                default,
            });
            self.skip_newlines();
        }
        self.close_block();
        Ok(fields)
    }

    fn at_group(&self) -> bool {
        let mut at = self.at;
        while matches!(self.tokens.get(at), Some(Token::Newline)) {
            at += 1;
        }
        matches!(self.tokens.get(at), Some(Token::Begin))
    }

    fn at_port(&self) -> bool {
        matches!(self.peek(), Some(Token::Word(word))
            if matches!(word.as_str(), "in" | "out" | "over" | "one" | "uses" | "enable" | "once" | "creates" | "removes" | "phase" | "needs" | "provides" | "pure" | "any"))
    }

    fn port(&mut self) -> Result<Port, String> {
        if self.accept_word("once") {
            self.end_of_line()?;
            return Ok(Port::Once);
        }
        if self.accept_word("pure") {
            self.end_of_line()?;
            return Ok(Port::Pure);
        }
        if self.accept_word("needs") {
            let names = self.name_list()?;
            self.end_of_line()?;
            return Ok(Port::Needs(names));
        }
        if self.accept_word("provides") {
            let names = self.name_list()?;
            self.end_of_line()?;
            return Ok(Port::Provides(names));
        }
        if self.accept_word("phase") {
            let name = self.word()?;
            self.end_of_line()?;
            return Ok(Port::Phase(name));
        }
        if self.accept_word("creates") {
            self.end_of_line()?;
            return Ok(Port::Creates);
        }
        if self.accept_word("removes") {
            self.end_of_line()?;
            return Ok(Port::Removes);
        }
        if self.accept_word("enable") {
            let condition = self.expression()?;
            self.end_of_line()?;
            return Ok(Port::Enable(condition));
        }
        if self.accept_word("in") {
            let names = self.name_list()?;
            self.end_of_line()?;
            return Ok(Port::Reads(names));
        }
        if self.accept_word("out") {
            let names = self.name_list()?;
            self.end_of_line()?;
            return Ok(Port::Writes(names));
        }

        let mut loose = false;
        if self.accept_word("any") {
            if !self.accept_word("order") {
                return Err(format!("expected order after any, found {}", self.recent()));
            }
            loose = true;
        }
        let cardinality = if self.accept_word("over") {
            Cardinality::Every
        } else if self.accept_word("one") {
            Cardinality::One
        } else {
            let _ = self.accept_word("uses");
            Cardinality::Set
        };

        let label = self.word()?;
        self.expect_symbol(":")?;

        let mut columns = Vec::new();
        loop {
            let written = if self.accept_word("out") {
                true
            } else {
                let _ = self.accept_word("in");
                false
            };
            let name = self.word()?;
            let start = if self.accept_symbol("=") {
                Some(self.expression()?)
            } else {
                None
            };
            columns.push(Column {
                name,
                written,
                start,
            });
            if !self.accept_symbol(",") {
                break;
            }
        }

        let mut requires = Vec::new();
        let mut excludes = Vec::new();
        if self.accept_word("with") {
            requires = self.name_list()?;
        }
        if self.accept_word("without") {
            excludes = self.name_list()?;
        }
        self.end_of_line()?;

        Ok(Port::Select(Selection {
            label,
            cardinality,
            loose,
            columns,
            requires,
            excludes,
        }))
    }

    fn name_list(&mut self) -> Result<Vec<String>, String> {
        let mut names = vec![self.dotted()?];
        while self.accept_symbol(",") {
            names.push(self.dotted()?);
        }
        Ok(names)
    }

    fn statement(&mut self) -> Result<Statement, String> {
        if self.accept_word("let") {
            let name = self.word()?;
            self.expect_symbol(":=")?;
            let value = self.expression()?;
            self.end_of_line()?;
            return Ok(Statement::Let { name, value });
        }
        if self.accept_word("break") {
            self.end_of_line()?;
            return Ok(Statement::Break);
        }
        if self.accept_word("stop") {
            self.end_of_line()?;
            return Ok(Statement::Stop);
        }
        if self.accept_word("add") {
            let value = self.expression()?;
            let _ = self.accept_word("to");
            let place = self.place()?;
            self.end_of_line()?;
            return Ok(Statement::Add { place, value });
        }
        if self.accept_word("remove") {
            let subject = self.expression()?;
            self.end_of_line()?;
            return Ok(Statement::Remove { subject });
        }
        if self.accept_word("keep") {
            let name = self.word()?;
            self.end_of_line()?;
            return Ok(Statement::Keep { name });
        }
        if self.accept_word("recall") {
            let name = self.word()?;
            self.end_of_line()?;
            return Ok(Statement::Recall { name });
        }
        if self.accept_word("send") {
            let name = self.word()?;
            let value = if matches!(self.peek(), Some(Token::Newline) | Some(Token::End) | None) {
                Expression::Truth(true)
            } else {
                self.expression()?
            };
            self.end_of_line()?;
            return Ok(Statement::Send { name, value });
        }
        if self.accept_word("emit") {
            let name = self.word()?;
            let value = if matches!(self.peek(), Some(Token::Newline) | Some(Token::End) | None) {
                Expression::Truth(true)
            } else {
                self.expression()?
            };
            self.end_of_line()?;
            return Ok(Statement::Emit { name, value });
        }
        if self.accept_word("spawn") {
            let mut bindings = Vec::new();
            loop {
                let name = self.word()?;
                let value = self.expression()?;
                bindings.push((name, value));
                if !self.accept_symbol(",") {
                    break;
                }
            }
            self.end_of_line()?;
            return Ok(Statement::Spawn { bindings });
        }
        if self.accept_word("when") {
            let condition = self.expression()?;
            self.end_of_line()?;
            let then = self.block()?;
            let mut otherwise = Vec::new();
            self.skip_newlines();
            if self.accept_word("else") {
                self.end_of_line()?;
                otherwise = self.block()?;
            }
            return Ok(Statement::When {
                condition,
                then,
                otherwise,
            });
        }
        if self.accept_word("for") {
            let _ = self.accept_word("each");
            let name = self.word()?;
            let _ = self.accept_word("in");
            let over = self.expression()?;
            self.end_of_line()?;
            let body = self.block()?;
            return Ok(Statement::ForEach { name, over, body });
        }

        let place = self.place()?;
        if self.accept_symbol("+=") {
            let value = self.expression()?;
            self.end_of_line()?;
            return Ok(Statement::Increase { place, value });
        }
        self.expect_symbol(":=")?;
        let value = self.expression()?;
        self.end_of_line()?;
        Ok(Statement::Assign { place, value })
    }

    fn place(&mut self) -> Result<Place, String> {
        let name = self.word()?;
        let subject = if self.accept_symbol("(") {
            let inner = self.expression()?;
            self.expect_symbol(")")?;
            Some(inner)
        } else {
            None
        };
        let mut field = if self.accept_symbol(".") {
            Some(self.word()?)
        } else {
            None
        };
        while let Some(held) = field.as_mut()
            && self.at_symbol(".")
        {
            self.at += 1;
            held.push('.');
            held.push_str(&self.word()?);
        }
        let index = if self.accept_symbol("[") {
            let inner = self.expression()?;
            self.expect_symbol("]")?;
            Some(inner)
        } else {
            None
        };
        Ok(Place {
            name,
            subject,
            field,
            index,
        })
    }

    fn block(&mut self) -> Result<Vec<Statement>, String> {
        let mut body = Vec::new();
        self.open_block()?;
        while !self.at_block_end() {
            body.push(self.statement()?);
            self.skip_newlines();
        }
        self.close_block();
        Ok(body)
    }

    fn expression(&mut self) -> Result<Expression, String> {
        self.any_of()
    }

    fn any_of(&mut self) -> Result<Expression, String> {
        let mut left = self.all_of()?;
        while self.accept_word("or") {
            let right = self.all_of()?;
            left = Expression::Binary {
                operator: BinaryOperator::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn all_of(&mut self) -> Result<Expression, String> {
        let mut left = self.comparison()?;
        while self.accept_word("and") {
            let right = self.comparison()?;
            left = Expression::Binary {
                operator: BinaryOperator::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn comparison(&mut self) -> Result<Expression, String> {
        let left = self.span()?;
        if self.accept_word("is") {
            let _ = self.accept_word("some");
            return Ok(Expression::IsSome(Box::new(left)));
        }
        let operator = if self.accept_symbol("<=") {
            BinaryOperator::LessOrEqual
        } else if self.accept_symbol(">=") {
            BinaryOperator::GreaterOrEqual
        } else if self.accept_symbol("!=") {
            BinaryOperator::NotEqual
        } else if self.accept_symbol("<") {
            BinaryOperator::Less
        } else if self.accept_symbol(">") {
            BinaryOperator::Greater
        } else if self.accept_symbol("=") {
            BinaryOperator::Equal
        } else {
            return Ok(left);
        };
        let right = self.span()?;
        Ok(Expression::Binary {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn span(&mut self) -> Result<Expression, String> {
        let left = self.sum()?;
        if self.accept_symbol("..") {
            let right = self.sum()?;
            return Ok(Expression::Binary {
                operator: BinaryOperator::Range,
                left: Box::new(left),
                right: Box::new(right),
            });
        }
        Ok(left)
    }

    fn sum(&mut self) -> Result<Expression, String> {
        let mut left = self.product()?;
        loop {
            let operator = if self.accept_symbol("+") {
                BinaryOperator::Add
            } else if self.accept_symbol("-") {
                BinaryOperator::Subtract
            } else {
                break;
            };
            let right = self.product()?;
            left = Expression::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn product(&mut self) -> Result<Expression, String> {
        let mut left = self.prefix()?;
        loop {
            let operator = if self.accept_symbol("*") {
                BinaryOperator::Multiply
            } else if self.accept_symbol("/") {
                BinaryOperator::Divide
            } else {
                break;
            };
            let right = self.prefix()?;
            left = Expression::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn prefix(&mut self) -> Result<Expression, String> {
        if self.accept_symbol("-") {
            let value = self.prefix()?;
            return Ok(Expression::Unary {
                operator: UnaryOperator::Negate,
                value: Box::new(value),
            });
        }
        if self.accept_word("not") {
            let value = self.prefix()?;
            return Ok(Expression::Unary {
                operator: UnaryOperator::Not,
                value: Box::new(value),
            });
        }
        self.suffix()
    }

    fn suffix(&mut self) -> Result<Expression, String> {
        let mut value = self.primary()?;
        loop {
            if self.accept_symbol(".") {
                let name = self.word()?;
                value = Expression::Field {
                    of: Box::new(value),
                    name,
                };
                continue;
            }
            if self.accept_symbol("[") {
                let at = self.expression()?;
                self.expect_symbol("]")?;
                value = Expression::Index {
                    of: Box::new(value),
                    at: Box::new(at),
                };
                continue;
            }
            break;
        }
        Ok(value)
    }

    fn primary(&mut self) -> Result<Expression, String> {
        match self.take() {
            Some(Token::Number(value)) => Ok(Expression::Number(value)),
            Some(Token::Text(value)) => Ok(Expression::Text(value)),
            Some(Token::Word(word)) => {
                if word == "true" {
                    return Ok(Expression::Truth(true));
                }
                if word == "false" {
                    return Ok(Expression::Truth(false));
                }
                if self.accept_symbol("(") {
                    let mut arguments = Vec::new();
                    while !self.accept_symbol(")") {
                        arguments.push(self.expression()?);
                        let _ = self.accept_symbol(",");
                    }
                    return Ok(Expression::Call {
                        name: word,
                        arguments,
                    });
                }
                Ok(Expression::Name(word))
            }
            Some(Token::Symbol(symbol)) if symbol == "(" => {
                let mut parts = vec![self.expression()?];
                while self.accept_symbol(",") {
                    parts.push(self.expression()?);
                }
                self.expect_symbol(")")?;
                if parts.len() == 1 {
                    return Ok(parts.remove(0));
                }
                Ok(Expression::Tuple(parts))
            }
            Some(Token::Symbol(symbol)) if symbol == "[" => {
                let mut items = Vec::new();
                while !self.accept_symbol("]") {
                    items.push(self.expression()?);
                    let _ = self.accept_symbol(",");
                }
                Ok(Expression::List(items))
            }
            Some(Token::Symbol(symbol)) if symbol == "{" => {
                let mut fields = Vec::new();
                while !self.accept_symbol("}") {
                    let name = self.word()?;
                    let value = self.expression()?;
                    fields.push((name, value));
                    let _ = self.accept_symbol(",");
                }
                Ok(Expression::Record(fields))
            }
            other => Err(format!("expected a value, found {other:?}")),
        }
    }
}
