# Candlewake

A lighthouse with the light out, told in words. The whole game is one story file at `src/data/story.txt` and the two crates that read and draw it, `nightshade-story` and `nightshade-ui-story`. The Rust in here composes the app, reads the file at startup, and builds the card. Nothing else.

```
just run candlewake
```

Click a link to take it, or type a command on the line at the foot of the card and press enter. `take the key`, `go north`, `light the lantern`, `look at the beacon` and `inventory` all read. The links under the text offer the same deeds the typed words reach, because both go through the one world.

Take the tinderbox from the shingle, the lantern from the hall, and light it. The stair and the hatch stay dark until you do. The oil is in the cellar and the beacon is at the top.

To change the game, change the story file. Every room, thing, verb, condition and ending in it is data.
