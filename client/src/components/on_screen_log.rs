use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2},
    EguiContexts,
};
use chrono::{DateTime, Local};

pub(crate) struct OnScreenLogPlugin;

impl Plugin for OnScreenLogPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LogEntry>()
            .insert_resource(LogEntries(Vec::new()))
            .add_systems((add_log, ui_log));
    }
}

#[derive(Resource)]
struct LogEntries(pub Vec<LogEntry>);

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub text: String,
    pub created: DateTime<Local>,
    pub lvl: LogLevel,
}

impl Default for LogEntry {
    fn default() -> Self {
        Self {
            text: String::new(),
            created: Local::now(),
            lvl: LogLevel::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warning,
    Error,
}

fn ui_log(mut contexts: EguiContexts, mut log_entries: ResMut<LogEntries>) {
    let ctx = contexts.ctx_mut();

    log_entries.0.retain(|log_entry| {
        let now = Local::now();
        let diff = now.signed_duration_since(log_entry.created);
        diff.num_seconds() < 10
    });

    if log_entries.0.is_empty() {
        return;
    }

    egui::Window::new("log")
        .anchor(Align2::RIGHT_BOTTOM, egui::vec2(0.0, 0.0))
        .auto_sized()
        .show(ctx, |ui| {
            egui::ScrollArea::new([false, true])
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        log_entries.0.iter().for_each(|log_entry| {
                            ui.label(format!(
                                "[{:?}] {}: {}",
                                log_entry.lvl,
                                log_entry.created.format("%d/%m/%Y %H:%M"),
                                log_entry.text
                            ));
                        });
                    });
                });
        });
}

fn add_log(mut log_entries: ResMut<LogEntries>, mut ev_log: EventReader<LogEntry>) {
    for log_event in ev_log.iter() {
        log_entries.0.push(log_event.clone());
    }
}
