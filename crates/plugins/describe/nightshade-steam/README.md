# nightshade-steam

The Steam crate wraps the Steamworks client in one resource a system reads. It holds the signed-in user, the achievements and stats, the friends list, and the peer-to-peer messaging sessions, and it refreshes them on the calls a program makes. When Steam is unavailable the resource stays uninitialized and every call reports nothing rather than failing the program.

## Composing it

```rust
App::new()
    .add_plugins(DefaultPlugins)
    .add_plugin(nightshade_steam::SteamPlugin)
    .add_plugin(MyGamePlugin)
    .run()
```

`SteamPlugin` inserts `SteamResources`, runs `steam_initialize` once on `Stage::Startup`, and pumps `steam_run_callbacks` every frame on `Stage::First`.

## Using it

```rust
use nightshade_steam::{
    SteamResources, steam_is_initialized, steam_refresh_achievements, steam_set_stat_int,
    steam_store_stats, steam_unlock_achievement,
};

fn record_the_run(steam: &mut SteamResources, wins: i32) -> Result<(), String> {
    if !steam_is_initialized(steam) {
        return Ok(());
    }
    steam_refresh_achievements(steam);
    steam_unlock_achievement(steam, "FIRST_WIN")?;
    steam_set_stat_int(steam, "wins", wins)?;
    steam_store_stats(steam)
}
```

A system takes the resource as `ResMut<SteamResources>` and hands it to a helper like this one. `steam_is_initialized` is the guard every other call wants, and `initialization_error` on the resource carries what went wrong. The achievement and stat calls report a `Result<(), String>`, and `steam_store_stats` is what commits them to Steam. `steam_refresh_achievements`, `steam_refresh_stats` and `steam_refresh_friends` fill the `achievements`, `stats` and `friends` lists on the resource. The networking half is `steam_setup_networking_callbacks`, `steam_send_message`, `steam_receive_messages`, `steam_process_pending_requests` and `steam_close_session`, which fill `received_messages`, `pending_session_requests` and `active_sessions`. `steam_open_invite_dialog`, `steam_open_overlay_to_user` and `steam_set_rich_presence` reach the overlay.

## Taking it from another crate

The crate does not split at a `contract` feature. Nothing asks it for types, so a program takes it whole, and `contract.rs` holds the plain values it reports: `PersonaState`, `AchievementInfo`, `StatInfo`, `StatValue`, `FriendInfo`, `NetworkMessage`, `PendingSessionRequest`, `SessionState` and `ActiveSession`.
