use bevy::prelude::*;

pub(crate) mod animation;
pub(crate) mod hover;
pub(crate) mod on_screen_log;
pub(crate) mod timer;

#[derive(SystemSet, Hash, Debug, Clone, Eq, PartialEq)]
pub enum ChangeDetectionSystemSet {
    MouseDetection,
    MouseDetectionFlush,
    Animation,
}

pub(crate) struct ComponentsPlugin;

impl Plugin for ComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            (
                ChangeDetectionSystemSet::MouseDetection,
                ChangeDetectionSystemSet::MouseDetectionFlush,
                ChangeDetectionSystemSet::Animation,
            )
                .chain(),
        )
        .add_system(apply_system_buffers.in_set(ChangeDetectionSystemSet::MouseDetectionFlush))
        .add_plugin(animation::AnimationPlugin)
        .add_plugin(hover::HoverPlugin)
        .add_plugin(on_screen_log::OnScreenLogPlugin)
        .add_plugin(timer::TimerPlugin);
    }
}
