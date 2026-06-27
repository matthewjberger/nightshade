//! Plain-data time accumulators a game owns and ticks itself.
//!
//! A [`Timer`] counts toward a fixed duration and reports when it elapses, once
//! or on a repeating cycle. A [`Stopwatch`] counts up without end. Neither holds
//! engine state: store one in a component or a resource, advance it with the
//! frame's delta, and read its progress. A repeating timer carries the overshoot
//! into the next cycle and reports how many cycles a single tick completed, so a
//! long frame neither drifts the schedule nor silently drops events. Both derive
//! serde, so they round-trip through a save file mid-countdown.

use serde::{Deserialize, Serialize};

/// Whether a [`Timer`] stops after it elapses or restarts for another cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum TimerMode {
    /// Elapses once and then stays finished until [`Timer::reset`].
    #[default]
    Once,
    /// Restarts each time it elapses, carrying the remainder into the next cycle.
    Repeating,
}

/// A countdown toward a duration. Tick it with the frame delta and read
/// [`Timer::just_finished`] for the cycle it completes, [`Timer::fraction`] for
/// progress to drive a bar or a fade, or [`Timer::remaining`] for a clock.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Timer {
    duration: f32,
    elapsed: f32,
    mode: TimerMode,
    paused: bool,
    finished: bool,
    times_finished_this_tick: u32,
}

impl Timer {
    /// A timer of `duration` seconds with the given mode.
    pub fn new(duration: f32, mode: TimerMode) -> Self {
        Self {
            duration: duration.max(0.0),
            mode,
            ..Default::default()
        }
    }

    /// A one-shot timer of `duration` seconds.
    pub fn once(duration: f32) -> Self {
        Self::new(duration, TimerMode::Once)
    }

    /// A repeating timer that elapses every `duration` seconds.
    pub fn repeating(duration: f32) -> Self {
        Self::new(duration, TimerMode::Repeating)
    }

    /// Advances the timer by `delta` seconds, returning it for chaining. A
    /// repeating timer wraps and counts every cycle the delta crossed; a
    /// finished one-shot timer ignores further ticks until reset; a paused timer
    /// ignores the delta entirely.
    pub fn tick(&mut self, delta: f32) -> &mut Self {
        if self.paused {
            self.times_finished_this_tick = 0;
            return self;
        }

        if self.mode == TimerMode::Once && self.finished {
            self.times_finished_this_tick = 0;
            return self;
        }

        self.elapsed += delta;

        if self.duration <= 0.0 {
            self.finished = true;
            self.times_finished_this_tick = 1;
            self.elapsed = 0.0;
            return self;
        }

        if self.elapsed >= self.duration {
            self.finished = true;
            match self.mode {
                TimerMode::Repeating => {
                    self.times_finished_this_tick = (self.elapsed / self.duration) as u32;
                    self.elapsed %= self.duration;
                }
                TimerMode::Once => {
                    self.times_finished_this_tick = 1;
                    self.elapsed = self.duration;
                }
            }
        } else {
            self.finished = false;
            self.times_finished_this_tick = 0;
        }

        self
    }

    /// Whether the timer reached its duration. For a repeating timer this is true
    /// only on a tick that completed a cycle, matching [`Timer::just_finished`].
    pub fn finished(&self) -> bool {
        match self.mode {
            TimerMode::Once => self.finished,
            TimerMode::Repeating => self.times_finished_this_tick > 0,
        }
    }

    /// Whether the most recent tick completed at least one cycle.
    pub fn just_finished(&self) -> bool {
        self.times_finished_this_tick > 0
    }

    /// How many full cycles the most recent tick completed. Always 0 or 1 for a
    /// one-shot timer; for a repeating timer a long frame can complete several.
    pub fn times_finished_this_tick(&self) -> u32 {
        self.times_finished_this_tick
    }

    /// Seconds elapsed in the current cycle.
    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    /// Seconds left in the current cycle.
    pub fn remaining(&self) -> f32 {
        (self.duration - self.elapsed).max(0.0)
    }

    /// The configured duration in seconds.
    pub fn duration(&self) -> f32 {
        self.duration
    }

    /// Progress through the current cycle from 0 at the start to 1 at the end.
    pub fn fraction(&self) -> f32 {
        if self.duration <= 0.0 {
            1.0
        } else {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        }
    }

    /// Progress remaining in the current cycle, from 1 at the start to 0 at the
    /// end.
    pub fn fraction_remaining(&self) -> f32 {
        1.0 - self.fraction()
    }

    /// This timer's mode.
    pub fn mode(&self) -> TimerMode {
        self.mode
    }

    /// Whether the timer is paused.
    pub fn paused(&self) -> bool {
        self.paused
    }

    /// Sets the duration, keeping elapsed time. Negative values clamp to zero.
    pub fn set_duration(&mut self, duration: f32) {
        self.duration = duration.max(0.0);
    }

    /// Sets the elapsed time into the current cycle.
    pub fn set_elapsed(&mut self, elapsed: f32) {
        self.elapsed = elapsed.max(0.0);
    }

    /// Switches between one-shot and repeating.
    pub fn set_mode(&mut self, mode: TimerMode) {
        self.mode = mode;
    }

    /// Pauses or resumes ticking.
    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    /// Pauses ticking.
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resumes ticking.
    pub fn unpause(&mut self) {
        self.paused = false;
    }

    /// Clears elapsed time and the finished state, leaving duration and mode.
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.finished = false;
        self.times_finished_this_tick = 0;
    }
}

/// An open-ended count-up. Tick it with the frame delta and read
/// [`Stopwatch::elapsed`]. Use it where there is no fixed deadline, such as time
/// since the player last moved or how long an input has been held.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Stopwatch {
    elapsed: f32,
    paused: bool,
}

impl Stopwatch {
    /// A stopwatch at zero.
    pub fn new() -> Self {
        Self::default()
    }

    /// Advances the stopwatch by `delta` seconds, returning it for chaining. A
    /// paused stopwatch ignores the delta.
    pub fn tick(&mut self, delta: f32) -> &mut Self {
        if !self.paused {
            self.elapsed += delta;
        }
        self
    }

    /// Seconds accumulated since the last reset.
    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    /// Whether the stopwatch is paused.
    pub fn paused(&self) -> bool {
        self.paused
    }

    /// Sets the accumulated time.
    pub fn set_elapsed(&mut self, elapsed: f32) {
        self.elapsed = elapsed.max(0.0);
    }

    /// Pauses or resumes ticking.
    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    /// Pauses ticking.
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resumes ticking.
    pub fn unpause(&mut self) {
        self.paused = false;
    }

    /// Clears the accumulated time.
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn once_latches_finished_and_clamps_elapsed() {
        let mut timer = Timer::once(1.0);
        timer.tick(0.6);
        assert!(!timer.finished());
        timer.tick(0.6);
        assert!(timer.finished());
        assert!(timer.just_finished());
        assert_eq!(timer.elapsed(), 1.0);
        timer.tick(5.0);
        assert!(timer.finished());
        assert!(!timer.just_finished());
        assert_eq!(timer.elapsed(), 1.0);
    }

    #[test]
    fn repeating_carries_remainder() {
        let mut timer = Timer::repeating(1.0);
        timer.tick(1.5);
        assert!(timer.just_finished());
        assert_eq!(timer.times_finished_this_tick(), 1);
        assert!((timer.elapsed() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn repeating_counts_multiple_cycles_in_one_tick() {
        let mut timer = Timer::repeating(0.5);
        timer.tick(1.6);
        assert_eq!(timer.times_finished_this_tick(), 3);
        assert!((timer.elapsed() - 0.1).abs() < 1e-6);
    }

    #[test]
    fn paused_timer_ignores_ticks() {
        let mut timer = Timer::once(1.0);
        timer.pause();
        timer.tick(2.0);
        assert!(!timer.finished());
        assert_eq!(timer.elapsed(), 0.0);
        timer.unpause();
        timer.tick(2.0);
        assert!(timer.finished());
    }

    #[test]
    fn fraction_reports_progress() {
        let mut timer = Timer::once(4.0);
        timer.tick(1.0);
        assert!((timer.fraction() - 0.25).abs() < f32::EPSILON);
        assert!((timer.fraction_remaining() - 0.75).abs() < f32::EPSILON);
        assert!((timer.remaining() - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn reset_clears_progress() {
        let mut timer = Timer::once(1.0);
        timer.tick(1.0);
        timer.reset();
        assert!(!timer.finished());
        assert_eq!(timer.elapsed(), 0.0);
    }

    #[test]
    fn stopwatch_accumulates_and_pauses() {
        let mut stopwatch = Stopwatch::new();
        stopwatch.tick(0.5);
        stopwatch.tick(0.5);
        assert!((stopwatch.elapsed() - 1.0).abs() < f32::EPSILON);
        stopwatch.pause();
        stopwatch.tick(1.0);
        assert!((stopwatch.elapsed() - 1.0).abs() < f32::EPSILON);
        stopwatch.reset();
        assert_eq!(stopwatch.elapsed(), 0.0);
    }
}
