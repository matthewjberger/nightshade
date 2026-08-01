use crate::contract::HearthsideCommands;
use crate::data;
use crate::data::Ask;
use crate::resources::{HearthsideShell, Lot};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_shell::data::Command;
use nightshade_shell::resources::state::ShellState;

const COMMANDS: &[Command<HearthsideShell>] = &[MAXMONEY, MONEY, NEEDS, CATALOG];

const MAXMONEY: Command<HearthsideShell> = Command {
    name: "maxmoney",
    description: "Drop 50000 into the household account",
    usage: "maxmoney",
    execute: maxmoney,
};

const MONEY: Command<HearthsideShell> = Command {
    name: "money",
    description: "Read the balance, or add to it",
    usage: "money [amount]",
    execute: money,
};

const NEEDS: Command<HearthsideShell> = Command {
    name: "needs",
    description: "Fill every need to the top",
    usage: "needs",
    execute: needs,
};

const CATALOG: Command<HearthsideShell> = Command {
    name: "catalog",
    description: "List the pieces you can buy",
    usage: "catalog",
    execute: catalog,
};

pub fn new_shell() -> ShellState<HearthsideShell> {
    let mut shell = ShellState::<HearthsideShell>::default();
    for command in COMMANDS {
        shell.registry.register(*command);
    }
    shell
}

pub fn route_keys(
    input: Res<Input>,
    actions: Res<Actions<Ask>>,
    mut window: ResMut<Window>,
    mut shell: ResMut<ShellState<HearthsideShell>>,
    reader: EventReader<AppEvent>,
) {
    for event in reader.iter() {
        if let AppEvent::Keyboard { key, state } = event {
            nightshade_shell::shell_key(&mut shell, &input, *key, *state);
        }
    }
    let leaving = !shell.visible && actions.just_pressed(Ask::Quit);
    if leaving {
        window.should_exit = true;
    }
}

pub fn route_the_asks(
    lot: Res<Lot>,
    mut shell: ResMut<ShellState<HearthsideShell>>,
    mut asks: ResMut<HearthsideCommands>,
) {
    shell.context.money = lot.money;
    asks.pending.append(&mut shell.context.asked);
}

fn maxmoney(_args: &[&str], _commands: &mut Commands, context: &mut HearthsideShell) -> String {
    context
        .asked
        .push(crate::contract::HearthsideCommand::Grant {
            amount: data::MAXMONEY_GRANT,
        });
    format!("{} added to the account", data::MAXMONEY_GRANT)
}

fn money(args: &[&str], _commands: &mut Commands, context: &mut HearthsideShell) -> String {
    match args {
        [] => format!("balance {:.0}", context.money),
        [amount] => match amount.parse::<u32>() {
            Ok(value) => {
                context
                    .asked
                    .push(crate::contract::HearthsideCommand::Grant { amount: value });
                format!("{value} added to the account")
            }
            Err(_) => "amount must be a whole number".to_string(),
        },
        _ => MONEY.usage.to_string(),
    }
}

fn needs(_args: &[&str], _commands: &mut Commands, context: &mut HearthsideShell) -> String {
    context
        .asked
        .push(crate::contract::HearthsideCommand::FillNeeds);
    "every need filled".to_string()
}

fn catalog(_args: &[&str], _commands: &mut Commands, _context: &mut HearthsideShell) -> String {
    let mut listing = String::new();
    for category in data::CATEGORIES {
        listing.push_str(&format!("{}:\n", data::category_name(category)));
        for piece in data::CATALOG
            .iter()
            .filter(|entry| entry.category == category)
        {
            let serves = match piece.need {
                Some(need) => data::need_name(need),
                None => "the house",
            };
            listing.push_str(&format!("  {} {} {}\n", piece.label, piece.price, serves));
        }
    }
    listing
}
