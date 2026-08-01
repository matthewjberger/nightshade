# blocker

A voxel world to dig, build, farm and live in. A seed raises the terrain, the biomes and the villages, the villagers walk their day, the beasts roam the hills, and a player mines, crafts, plants, sails, fights and keeps a journal of it.

The game is seventeen crates under `crates/` and an app that composes them. Each crate holds one capability, installs its own resources and registers its own systems, and names only the crates below it. The app keeps the composition, the screen flow, the keys, the start of a run and the four startup edges that cross a crate, which no plugin can state on its own.

The order runs `blocker-screen`, `blocker-blocks`, `blocker-pocket`, `blocker-weather`, `blocker-world`, `blocker-farm`, `blocker-save`, `blocker-player`, `blocker-quests`, `blocker-crew`, which composes `blocker-figure`, `blocker-chat`, `blocker-beasts`, `blocker-barrow`, `blocker-talk`, `blocker-hud`, `blocker-screens`.

The shapes the framework tier already carries come from there. The health of every body is `nightshade-health`, the toast is `nightshade-toast`, a villager's speech is `nightshade-dialogue`, the bag and every chest are `nightshade-inventory`, the settings rows are `nightshade-ui-config`, and the cards, the rows, the bars and the item cells every screen is built from are `nightshade-ui-menu`. The weather, the day and the villagers' walk stay in the game, because the framework's weather knows four skies and no season, the day rides the renderer's own clock, and a villager keeps a schedule and walks the voxel ground.

## Running it

```
just run blocker
```

Every app flag works here: `--capture <path>`, `--capture-frame <n>` and `--size WxH`.

The player, the villagers, the beasts and the weather each declare their settings with `nightshade-config`, and each keeps its file in the `blocker` folder under the system's data folder, so a number that is changed stays changed.
