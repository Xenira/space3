use bevy::{
    prelude::*,
    text::Text,
    time::{Stopwatch, Time, Timer},
};

pub(crate) struct TimerPlugin;

impl Plugin for TimerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TimerUi>()
            .add_system(stopwatch_tick)
            .add_systems((timer_tick, timer_text_update).chain());
    }
}

#[derive(Component)]
pub struct StopwatchComponent {
    pub(crate) time: Stopwatch,
}

#[derive(Component)]
pub struct TimerTextComponent;

#[derive(Resource, Default)]
pub struct TimerUi(pub Option<Timer>);

fn stopwatch_tick(time: Res<Time>, mut stopwatch: Query<&mut StopwatchComponent>) {
    for mut watch in stopwatch.iter_mut() {
        watch.time.tick(time.delta());
    }
}

fn timer_tick(time: Res<Time>, mut timer: ResMut<TimerUi>) {
    if let Some(timer) = timer.0.as_mut() {
        timer.tick(time.delta());
    }
}

fn timer_text_update(
    timer: Res<TimerUi>,
    mut q_timer_text: Query<&mut Text, With<TimerTextComponent>>,
) {
    if let Some(timer) = timer.0.as_ref() {
        for mut text in q_timer_text.iter_mut() {
            text.sections[0].value = timer.remaining_secs().ceil().to_string();
        }
    }
}
