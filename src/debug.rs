use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, EntityCountDiagnosticsPlugin};
use bevy::input::{keyboard::KeyboardInput, ButtonState};

use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiSet};

#[derive(Default, Resource)]
struct DebugUiState {
    perf_stats: bool,
}

fn display_perf_stats(mut egui: EguiContexts, diagnostics: Res<DiagnosticsStore>) {
    egui::Window::new("Perf Info").show(egui.ctx_mut(), |ui| {
        ui.label(format!(
            "Avg. FPS: {:.02}",
            diagnostics
                .get(&FrameTimeDiagnosticsPlugin::FPS)
                .unwrap()
                .average()
                .unwrap_or_default()
        ));
        ui.label(format!(
            "Total Entity count: {}",
            diagnostics
                .get(&EntityCountDiagnosticsPlugin::ENTITY_COUNT)
                .unwrap()
                .average()
                .unwrap_or_default()
        ));
    });
}

fn should_display_perf_stats(state: Res<DebugUiState>) -> bool {
    state.perf_stats
}

fn toggle_debug_ui_displays(
    mut inputs: EventReader<KeyboardInput>,
    mut ui_state: ResMut<DebugUiState>,
) {
    for input in inputs.read() {
        match (input.key_code, input.state) {
            (KeyCode::F3, ButtonState::Pressed) => {
                ui_state.perf_stats = !ui_state.perf_stats;
            }
            _ => {}
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, SystemSet)]
/// Systems related to the debug UIs.
enum DebugUiSet {
    Toggle,
    Display,
}

pub struct DebugUiPlugins;

impl Plugin for DebugUiPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_plugins(FrameTimeDiagnosticsPlugin)
            .add_plugins(EntityCountDiagnosticsPlugin)
            .add_systems(Update, toggle_debug_ui_displays.in_set(DebugUiSet::Toggle))
            .add_systems(
                Update,
                display_perf_stats
                    .in_set(DebugUiSet::Display)
                    .run_if(should_display_perf_stats),
            )
            .configure_sets(
                Update,
                (DebugUiSet::Toggle, DebugUiSet::Display)
                    .chain()
                    .after(EguiSet::ProcessInput),
            )
            .init_resource::<DebugUiState>();
    }
}