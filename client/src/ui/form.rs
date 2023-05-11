use std::collections::HashMap;

use bevy::{ecs::system::Resource, prelude::*};

use super::{focus::FocusPlugin, text_input::TextInputPlugin};

pub type FormMapping = HashMap<String, FormValue>;
pub type FormElements = HashMap<String, FormElement>;

pub(crate) struct FormPlugin;

impl Plugin for FormPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FormSubmitEvent>()
            .add_event::<FormElementChangedEvent>();
    }
}

pub struct FormPluginGroup;

impl PluginGroup for FormPluginGroup {
    fn build(&mut self, group: &mut bevy::app::PluginGroupBuilder) {
        group.add(FormPlugin).add(FocusPlugin).add(TextInputPlugin);
    }
}

#[derive(Debug)]
pub enum FormError {
    FieldsMissing(String),
    TypeMismatch { expected: FormValue, got: FormValue },
    Invalid(String),
    Unknown(String),
}

pub struct FormSubmitEvent;

pub struct FormElementChangedEvent(pub FormElements);

#[derive(Component)]
pub struct FormId(pub String);

#[derive(Default)]
pub struct Form<T: Default> {
    pub name: String,
    pub data: T,
    pub controls: FormElements,
}

pub trait FromFormMapping {
    fn from_mapping(_: &FormMapping) -> Result<Self, FormError>
    where
        Self: Sized;
}
pub trait IntoFormMapping {
    fn into_mapping(self) -> FormMapping;
}

impl<T: FromFormMapping + Default> Form<T> {
    pub fn get(&self) -> Result<T, FormError> {
        T::from_mapping(&self.get_mapping())
    }
}

impl<T: IntoFormMapping + Default> Form<T> {
    pub fn set(&mut self, data: T) -> Result<(), FormError> {
        let elements: FormMapping = data.into_mapping();
        for (name, value) in elements {
            self.set_val(&name, Some(value));
        }
        Ok(())
    }
}

impl<T: Default> Form<T> {
    pub(crate) fn set_val(
        &mut self,
        name: &str,
        value: Option<FormValue>,
    ) -> Result<(), FormError> {
        if let Some(mut element) = self.controls.get_mut(name) {
            element.value = value;
            return Ok(());
        }

        Err(FormError::FieldsMissing(name.to_string()))
    }

    pub(crate) fn get_val(&self, name: &str) -> Result<Option<&FormElement>, FormError> {
        Ok(self.controls.get(name))
    }

    pub fn get_mapping(&self) -> FormMapping {
        self.controls
            .iter()
            .filter(|(name, _)| !name.starts_with("_"))
            .filter(|(_, value)| value.value.is_some())
            .map(|e| (e.0.clone(), e.1.value.clone().unwrap()))
            .collect::<FormMapping>()
    }
}

pub fn on_change<T: Default + Resource>(
    mut form: ResMut<Form<T>>,
    mut ev_change: EventReader<FormElementChangedEvent>,
) {
    for change in ev_change.iter() {
        form.controls.extend(change.0.clone());
        debug!("{:?}", form.controls);
    }
}

#[derive(Clone, Default, Debug)]
pub struct FormElement {
    pub entity: Option<Entity>,
    pub value: Option<FormValue>,
}

#[derive(Debug)]
struct FormValidation {
    pub pristine: bool,
    pub touched: bool,
    pub dirty: bool,
    pub valid: bool,
}

impl Default for FormValidation {
    fn default() -> Self {
        Self {
            pristine: true,
            touched: false,
            dirty: false,
            valid: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum FormValue {
    String(String),
    Bool(bool),
    Int(i32),
    Float(f32),
}
