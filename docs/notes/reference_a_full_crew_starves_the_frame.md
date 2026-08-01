---
name: reference_a_full_crew_starves_the_frame
description: The worker crews together take one in two logical processors, because a crew on every spare processor slows the frame thread and the driver
type: reference
---

`nightshade-worker` sizes every crew from one budget, `available_parallelism`
divided by `CREW_DIVISOR`, and every crew of the process draws from it. The
processors outside the budget belong to the frame thread, to the threads the
driver runs, and to the sibling threads of the cores the crews run on. A crew
sized from the cores minus a constant leaves those three with nothing on a
machine with many processors, and the crew's memory traffic slows the frame's
own work while the crew is busy, so the frame rate drops while a world streams
in and comes back when the crew goes idle.

A plugin that hires more than one crew reads `workers_free` and splits what is
left on purpose, the way `nightshade-voxels` gives three of every five to the
diggers, so the split does not depend on which crew is hired first.
