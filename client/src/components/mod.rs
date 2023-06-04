use bevy::prelude::*;

pub(crate) mod anchors;
pub(crate) mod animation;
pub(crate) mod cursor;
pub(crate) mod dragndrop;
pub(crate) mod hover;
pub(crate) mod on_screen_log;
pub(crate) mod timer;
pub(crate) mod tooltip;

#[derive(SystemSet, Hash, Debug, Clone, Eq, PartialEq)]
pub enum ChangeDetectionSystemSet {
    MouseDetection,
    MouseDetectionFlush,
    Tooltip,
    TooltipRender,
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
                ChangeDetectionSystemSet::Tooltip,
                ChangeDetectionSystemSet::TooltipRender,
            )
                .chain()
                .in_base_set(CoreSet::PreUpdate),
        )
        .add_system(apply_system_buffers.in_set(ChangeDetectionSystemSet::MouseDetectionFlush))
        .add_plugin(animation::AnimationPlugin)
        .add_plugin(hover::HoverPlugin)
        .add_plugin(dragndrop::DragNDropPlugin)
        .add_plugin(on_screen_log::OnScreenLogPlugin)
        .add_plugin(timer::TimerPlugin)
        .add_plugin(anchors::AnchorsPlugin)
        .add_plugin(cursor::CursorPlugin)
        .add_plugin(tooltip::TooltipPlugin);
    }
}
