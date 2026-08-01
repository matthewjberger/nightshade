# actions

A cube on a floor, driven through `nightshade-input` and nothing else.

The app declares five actions in `src/data.rs`: walk, jump, dash, next profile
and rebind. Each carries the keys, buttons, sticks and triggers it answers to
when nobody has said otherwise. The plugin opens a second profile, `southpaw`,
where walking answers the arrow keys and jumping answers a different key, and
the app starts on `default`.

The panel on the left is read off the `Bindings` resource: the profile that is
worn, and each action with the bindings it holds in that profile. It refreshes
when the bindings change. The last line names the actions held this frame, read
off the `Actions` resource, so a stick and a key show the same word.

Next profile wears the other profile in one write. Rebind listens for the next
key or pad button a person presses and makes it the only binding for jump in
the worn profile. Both changes go to `actions.toml` below the `nightshade`
folder of the platform's data directory through `persist`, so the next run
starts where this one left off. The file names actions by their `name` and a
binding it cannot read is dropped, so a stale file never binds the wrong key.

```
just run actions
```

What to look for: walk with the keys or the left stick and the cube moves the
same, hold dash and it moves faster, jump and it hops. Swap the profile and the
walk keys change under you while the stick keeps working. Rebind jump to any
key, quit, run again, and the panel still shows the key you picked.
