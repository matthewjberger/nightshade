---
name: reference_camera_on_a_body_inherits_the_physics_step
description: A camera parented to a character body moves in whole physics steps, so a fast first person game sets the step from the frame and leaves interpolation off
type: reference
---

The physics adds the frame time to an accumulator and runs whole steps of
`PhysicsWorld.fixed_timestep`. `character_controller_system` writes the body's
position once per step, so a camera parented to that body holds still between
steps and jumps at each one. At sixteen units a second a sixty hertz step moves
the body a quarter of a unit, and against a wall three units away that is about
five degrees of view. At a hundred and forty frames a second a frame takes one
step or none, so the view holds for two frames and then jumps. It reads as a
constant shake rather than as a stutter, and it does not stop on a pause screen,
because the physics runs in `Stage::FrameUpdate` in every state.

`PhysicsInterpolation` does not save it. The interpolation writes the body
transform twice in one frame, once restored to the simulated position at the
start of each substep and once as a lerp from the previous step toward it, and
the lerp is behind the simulation by the part of a step the frame has not
simulated yet. That lag is a different size every frame, so the view swings by
up to one step of movement at the beat between the frame rate and the step.

For a game whose only physics is a character controller over static geometry,
one step per frame removes both. `physics_world_set_timestep` takes the frame's
own duration, clamped to a floor and a ceiling, and the body then advances
exactly the distance the frame covers. That command sets the accumulator step,
rapier's own `dt` and the substep cap together, because
`integration_parameters.dt` is private and does not follow `fixed_timestep` on
its own. A body driven this way wants `PhysicsInterpolation` disabled, because
interpolation over a step that equals the frame only adds the lag back.

Related: [[project_boomstick_app]]
