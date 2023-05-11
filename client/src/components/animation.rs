use std::{collections::HashMap, time::Duration};

use bevy::{
    prelude::*,
    time::{Time, Timer},
};

use super::ChangeDetectionSystemSet;

pub(crate) struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (animate_sprite, animate_transform).in_set(ChangeDetectionSystemSet::Animation),
        );
    }
}

#[derive(Bundle)]
pub struct AnimationBundle {
    pub animation: Animation,
    pub animation_timer: AnimationTimer,

    #[bundle]
    pub sprite_sheet: SpriteSheetBundle,
}

#[derive(Debug, Clone)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

impl AnimationIndices {
    pub fn new(first: usize, last: usize) -> Self {
        Self { first, last }
    }
}

#[derive(Debug, Component, Clone)]
pub struct Animation {
    pub states: HashMap<String, AnimationState>,
    pub global_transitions: HashMap<String, AnimationTransition>,
    pub playing: bool,
    pub current_state: Option<AnimationState>,
}

impl Animation {
    pub fn add_state(&mut self, state: AnimationState) -> &mut Self {
        self.states.insert(state.name.clone(), state);
        self
    }

    pub fn with_state(mut self, state: AnimationState) -> Self {
        self.add_state(state);
        self
    }

    pub fn add_global_transition(&mut self, transition: AnimationTransition) -> &mut Self {
        self.global_transitions
            .insert(transition.name.clone(), transition);
        self
    }

    pub fn with_global_transition(mut self, transition: AnimationTransition) -> Self {
        self.add_global_transition(transition);
        self
    }

    pub fn with_current_state(mut self, state: &str) -> Self {
        self.current_state = self.states.get(state).cloned();
        self
    }

    pub fn with_playing(mut self, playing: bool) -> Self {
        self.playing = playing;
        self
    }

    pub fn get_transition(&self, name: &str) -> Option<AnimationTransition> {
        self.current_state
            .as_ref()
            .and_then(|state| state.transitions.get(name))
            .or(self.global_transitions.get(name))
            .cloned()
    }
}

impl Default for Animation {
    fn default() -> Self {
        Self {
            states: HashMap::new(),
            global_transitions: HashMap::new(),
            playing: true,
            current_state: None,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct TransformAnimation {
    pub target: Transform,
    pub speed: f32,
}

#[derive(Debug, Clone)]
pub struct AnimationState {
    pub name: String,
    pub indices: AnimationIndices,
    pub transitions: HashMap<String, AnimationTransition>,
    pub repeat_type: AnimationRepeatType,
    pub direction: AnimationDirection,
    pub on_finish: Option<String>,
    pub fps: f32,
}

impl AnimationState {
    pub fn new(name: &str, indices: AnimationIndices) -> Self {
        Self {
            name: name.to_string(),
            indices,
            transitions: HashMap::new(),
            repeat_type: AnimationRepeatType::Loop,
            direction: AnimationDirection::Forward,
            on_finish: None,
            fps: 12.0,
        }
    }

    pub fn with_fps(mut self, fps: f32) -> Self {
        self.fps = fps;
        self
    }

    pub fn with_repeat_type(mut self, repeat_type: AnimationRepeatType) -> Self {
        self.repeat_type = repeat_type;
        self
    }

    pub fn with_direction(mut self, direction: AnimationDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn with_transition(mut self, transition: AnimationTransition) -> Self {
        self.transitions.insert(transition.name.clone(), transition);
        self
    }

    pub fn with_on_finish(mut self, on_finish: &str) -> Self {
        self.on_finish = Some(on_finish.to_string());
        self
    }
}

#[derive(Component, Debug, Clone)]
pub struct AnimationTransition {
    pub name: String,
    pub transition_type: AnimationTransitionType,
    pub to_state: String,
}

#[derive(Debug, Default, Clone)]
pub enum AnimationTransitionType {
    Imediate,
    #[default]
    Finish,
    Blend(u32),
    Intermediary(String),
}

#[derive(Debug, Default, Clone)]
pub enum AnimationRepeatType {
    #[default]
    Loop,
    Once,
    PingPong,
}

#[derive(Debug, Default, Clone)]
pub enum AnimationDirection {
    #[default]
    Forward,
    Backward,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

fn animate_sprite(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut Animation,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        Option<&AnimationTransition>,
    )>,
) {
    for (entity, mut animation, mut timer, mut sprite, transition) in &mut query {
        if !animation.playing {
            continue;
        }

        if animation.current_state.is_none() {
            if let Some(next_state) = animation.states.values().next() {
                timer.set_duration(Duration::from_secs_f32(1.0 / next_state.fps));
                animation.current_state = Some(next_state.clone());
            } else {
                continue;
            }
        }

        timer.tick(time.delta());
        if timer.just_finished() {
            let current_state = animation.current_state.clone().unwrap();
            let new_state = if let Some(transition) = transition {
                debug!("{:?} Animation transition {:?}", entity, transition);
                animation
                    .states
                    .get(&transition.to_state)
                    .or(animation.states.values().next())
                    .map(|next_state| match transition.transition_type {
                        AnimationTransitionType::Finish => {
                            if sprite.index == current_state.indices.last {
                                Some(next_state.clone())
                            } else {
                                None
                            }
                        }
                        AnimationTransitionType::Imediate | _ => Some(next_state.clone()),
                    })
                    .flatten()
            } else {
                None
            };

            if let Some(new_state) = new_state {
                debug!(
                    "{:?} Animation {} -> {}",
                    entity, current_state.name, new_state.name
                );
                animation.current_state = Some(new_state.clone());
                timer.set_duration(Duration::from_secs_f32(1.0 / new_state.fps));
                commands.entity(entity).remove::<AnimationTransition>();
            }

            let indices = &current_state.indices;
            let indices = match current_state.direction {
                AnimationDirection::Forward => indices.first..=indices.last,
                AnimationDirection::Backward => indices.last..=indices.first,
            };
            sprite.index = if sprite.index == *indices.end() {
                match current_state.repeat_type {
                    AnimationRepeatType::Loop => *indices.start(),
                    AnimationRepeatType::Once => {
                        trace!("Animation {} finished", current_state.name);
                        sprite.index
                    }
                    AnimationRepeatType::PingPong => {
                        animation.current_state.as_mut().unwrap().direction =
                            match current_state.direction {
                                AnimationDirection::Forward => AnimationDirection::Backward,
                                AnimationDirection::Backward => AnimationDirection::Forward,
                            };

                        sprite.index
                    }
                }
            } else if sprite.index < current_state.indices.first
                || sprite.index > current_state.indices.last
            {
                match current_state.direction {
                    AnimationDirection::Forward => *indices.end(),
                    AnimationDirection::Backward => *indices.start(),
                }
            } else {
                match current_state.direction {
                    AnimationDirection::Forward => sprite.index + 1,
                    AnimationDirection::Backward => sprite.index - 1,
                }
            }
            .clamp(current_state.indices.first, current_state.indices.last);
        }
    }
}

fn animate_transform(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &TransformAnimation)>,
) {
    for (entity, mut transform, animation) in &mut query {
        let delta = time.delta_seconds() * animation.speed;
        let transform = transform.as_mut();
        let translation = animation.target.translation - transform.translation;
        let rotation = animation.target.rotation - transform.rotation;
        let scale = animation.target.scale - transform.scale;

        let mut finished = true;
        if translation.length() < delta * 128.0 {
            transform.translation = animation.target.translation;
        } else {
            finished = false;
            transform.translation += translation.normalize() * delta * 128.0;
        }

        if rotation.length() < delta {
            transform.rotation = animation.target.rotation;
        } else {
            finished = false;
            transform.rotation = transform.rotation + rotation.normalize() * delta;
        }

        if scale.length() < delta {
            transform.scale = animation.target.scale;
        } else {
            finished = false;
            transform.scale += scale.normalize() * delta;
        }

        if finished {
            commands.entity(entity).remove::<TransformAnimation>();
        }
    }
}
