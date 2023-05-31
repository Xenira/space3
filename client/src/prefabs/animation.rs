use crate::components::animation::{
    Animation, AnimationDirection, AnimationIndices, AnimationRepeatType, AnimationState,
    AnimationTransition, AnimationTransitionType,
};

pub fn simple(first: usize, last: usize) -> Animation {
    Animation::default()
        .with_state(AnimationState::new(
            "idle",
            AnimationIndices { first, last },
        ))
        .with_current_state("idle")
}

pub fn add_hover_state(animation: &mut Animation, first: usize, last: usize) {
    animation
        .add_state(
            AnimationState::new("hover", AnimationIndices { first, last })
                .with_repeat_type(AnimationRepeatType::Once)
                .with_fps(32.0),
        )
        .add_state(
            AnimationState::new("leave", AnimationIndices { first, last })
                .with_repeat_type(AnimationRepeatType::Once)
                .with_direction(AnimationDirection::Backward)
                .with_fps(32.0),
        )
        .add_global_transition(AnimationTransition {
            name: "hover".to_string(),
            transition_type: AnimationTransitionType::Imediate,
            to_state: "hover".to_string(),
        })
        .add_global_transition(AnimationTransition {
            name: "leave".to_string(),
            transition_type: AnimationTransitionType::Imediate,
            to_state: "leave".to_string(),
        });
}
