use crate::contract::{Action, Binding, Bindings, DEFAULT_PROFILE, filled};
use serde::Deserialize;

pub fn open_profile<A: Action>(bindings: &mut Bindings<A>, profile: &str) {
    bindings
        .profiles
        .entry(profile.to_string())
        .or_insert_with(filled::<A>);
    bindings.revision += 1;
}

pub fn bind<A: Action>(bindings: &mut Bindings<A>, profile: &str, action: A, binding: Binding) {
    let table = bindings
        .profiles
        .entry(profile.to_string())
        .or_insert_with(filled::<A>);
    let held = table.entry(action).or_default();
    if !held.contains(&binding) {
        held.push(binding);
    }
    bindings.revision += 1;
}

pub fn bind_only<A: Action>(
    bindings: &mut Bindings<A>,
    profile: &str,
    action: A,
    binding: Binding,
) {
    let table = bindings
        .profiles
        .entry(profile.to_string())
        .or_insert_with(filled::<A>);
    table.insert(action, vec![binding]);
    bindings.revision += 1;
}

pub fn unbind<A: Action>(bindings: &mut Bindings<A>, profile: &str, action: A) {
    let table = bindings
        .profiles
        .entry(profile.to_string())
        .or_insert_with(filled::<A>);
    table.insert(action, Vec::new());
    bindings.revision += 1;
}

pub fn wear<A: Action>(bindings: &mut Bindings<A>, profile: &str) -> bool {
    if !bindings.profiles.contains_key(profile) {
        return false;
    }
    bindings.worn = profile.to_string();
    bindings.revision += 1;
    true
}

pub fn forget<A: Action>(bindings: &mut Bindings<A>, profile: &str) {
    if profile == DEFAULT_PROFILE {
        return;
    }
    bindings.profiles.remove(profile);
    if bindings.worn == profile {
        bindings.worn = DEFAULT_PROFILE.to_string();
    }
    bindings.revision += 1;
}

pub fn bindings_apply_text<A: Action>(bindings: &mut Bindings<A>, text: &str) {
    let Ok(root) = text.parse::<toml::Table>() else {
        return;
    };
    if let Some(profiles) = root.get("profiles").and_then(toml::Value::as_table) {
        for (profile, written) in profiles {
            let Some(written) = written.as_table() else {
                continue;
            };
            let table = bindings
                .profiles
                .entry(profile.clone())
                .or_insert_with(filled::<A>);
            for action in A::all() {
                let Some(list) = written.get(action.name()) else {
                    continue;
                };
                let Ok(bound) = Vec::<Binding>::deserialize(list.clone()) else {
                    continue;
                };
                table.insert(*action, bound);
            }
        }
    }
    if let Some(worn) = root.get("worn").and_then(toml::Value::as_str)
        && bindings.profiles.contains_key(worn)
    {
        bindings.worn = worn.to_string();
    }
    bindings.revision += 1;
}
