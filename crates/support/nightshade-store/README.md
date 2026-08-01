# nightshade-store

The store is one pair of calls over whatever the platform keeps files in. A program names a folder and a file and reads or writes bytes, and the crate picks the place: a file below the system's data folder on a desktop, a file below the folder the activity was handed on android, and local storage in a browser. It has no ECS in it and takes no engine dependency.

## Composing it

The crate carries no plugin. A program calls the functions directly and composes nothing.

## Using it

```rust
use nightshade_store::prelude::{store_read_text, store_remove, store_write, store_write_text};

let text = config_text(&config);
if store_write_text(file.folder, file.name, &text) {
    file.saved_revision = config.revision();
}

match store_read_text(folder, name) {
    Some(text) => settle(&text),
    None => settle_defaults(),
}
```

Every call takes the folder and the file name as plain strings. `store_write` and `store_write_text` report whether the bytes landed, and `store_remove` reports whether the file went away. `store_read` hands back the bytes and `store_read_text` the string, and both report nothing when the file is absent or unreadable, so a program that finds nothing opens on its own defaults.

## Taking it from another crate

The crate does not split at a `contract` feature, because the whole crate is already the contract. `nightshade-save` and `nightshade-config` take it whole and name it by path.
