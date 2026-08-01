---
name: reference_day_night_system_gating
description: day_night_cycle_system is a no-op unless auto_cycle is true AND atmosphere is Atmosphere::DayNight, and it needs sun_entity set
type: reference
---

`day_night_cycle_system` is registered by `GraphicsPlugin` on
`Stage::FrameUpdate`, and it returns early unless BOTH
`RendererState.day_night.auto_cycle` is true AND
`RenderSettings.atmosphere` is `Atmosphere::DayNight`. It also needs
`day_night.sun_entity` pointing at a directional light to move. Setting
`day_night.hour` alone does nothing you can see.

To drive the hour yourself (the sun arcs, the light warms and cools), set
`atmosphere` to `Atmosphere::DayNight`, `auto_cycle` to true, `speed` to `0.0`
so the system does not advance the hour itself, and `sun_entity`, then write
`hour` each frame.
