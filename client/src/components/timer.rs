use bevy::{
    prelude::{App, Component, Plugin, Query, Res},
    time::{Stopwatch, Time, Timer},
};

pub(crate) struct TimerPlugin;

impl Plugin for TimerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(stopwatch_tick).add_system(timer_tick);
    }
}

#[derive(Component)]
pub struct StopwatchComponent {
    pub(crate) time: Stopwatch,
}

#[derive(Component)]
pub struct TimerComponent {
    pub(crate) time: Timer,
}

fn stopwatch_tick(time: Res<Time>, mut stopwatch: Query<&mut StopwatchComponent>) {
    for mut watch in stopwatch.iter_mut() {
        watch.time.tick(time.delta());
    }
}

fn timer_tick(time: Res<Time>, mut timer: Query<&mut TimerComponent>) {
    for mut t in timer.iter_mut() {
        t.time.tick(time.delta());
    }
}
