mod components;
mod layout;
mod math;
mod state;

use crate::app::components::graph::node;
use crate::app::components::graph::node::*;

use eframe::egui::Visuals;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    state: state::AppState,
}

#[allow(clippy::derivable_impls)]
impl Default for App {
    fn default() -> Self {
        Self {
            state: state::AppState::default(),
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        cc.egui_ctx.set_visuals(Visuals::dark());

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self { state } = self;

        egui::TopBottomPanel::top("top_panel")
            .show(ctx, |ui| layout::top_bar::show(state, ui, ctx));

        egui::SidePanel::left("side_panel").show(ctx, |ui| layout::side_pannel::show(state, ui));

        egui::CentralPanel::default().show(ctx, |ui| layout::central_pannel::show(state, ui, ctx));

        if state.first_loop {
            init_nodes(state);
        }

        state.first_loop = false;
    }
}

pub fn init_nodes(state: &mut state::AppState) {
    node::create_node(state, NodeTemplate::ImageFetcher, egui::pos2(30.0, 30.0));
}
