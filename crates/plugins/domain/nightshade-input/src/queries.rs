use crate::contract::{Action, Binding, Bindings};

pub fn worn<A: Action>(bindings: &Bindings<A>) -> &str {
    &bindings.worn
}

pub fn profiles<A: Action>(bindings: &Bindings<A>) -> Vec<&str> {
    let mut names: Vec<&str> = bindings.profiles.keys().map(String::as_str).collect();
    names.sort_unstable();
    names
}

pub fn bindings_of<A: Action>(bindings: &Bindings<A>, action: A) -> &[Binding] {
    bindings
        .profiles
        .get(&bindings.worn)
        .and_then(|table| table.get(&action))
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

pub fn bindings_text<A: Action>(bindings: &Bindings<A>) -> String {
    let mut profiles = toml::Table::new();
    for (profile, table) in &bindings.profiles {
        let mut written = toml::Table::new();
        for action in A::all() {
            let Some(bound) = table.get(action) else {
                continue;
            };
            let Ok(value) = toml::Value::try_from(bound) else {
                continue;
            };
            written.insert(action.name().to_string(), value);
        }
        profiles.insert(profile.clone(), toml::Value::Table(written));
    }
    let mut root = toml::Table::new();
    root.insert(
        "worn".to_string(),
        toml::Value::String(bindings.worn.clone()),
    );
    root.insert("profiles".to_string(), toml::Value::Table(profiles));
    toml::to_string(&root).unwrap_or_default()
}
