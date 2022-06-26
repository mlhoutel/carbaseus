use egui::TextStyle;

use crate::app::components::graph::node::*;
use crate::app::state;

use egui_node_graph::NodeResponse;

pub fn show(state: &mut state::AppState, ui: &mut egui::Ui, ctx: &egui::Context) {
    show_background(ui);

    if state.d_state {
        show_state(state, ui, ctx);
    }

    let responses = state.graph.draw_graph_editor(ui, AllNodeTemplates);

    if state.auto_compute {
        let must_refresh = responses.node_responses.iter().find(|&&event| match event {
            NodeResponse::ConnectEventEnded(_) | NodeResponse::DisconnectEvent(_) => true,

            NodeResponse::ConnectEventStarted(_, _)
            | NodeResponse::CreatedNode(_)
            | NodeResponse::SelectNode(_)
            | NodeResponse::DeleteNode(_)
            | NodeResponse::RaiseNode(_) => false,

            NodeResponse::User(user_event) => match user_event {
                Response::ImageFetched => true,
                Response::ScalarChanged => true,
            },
        });

        if must_refresh.is_some() {
            evaluate_graph(&mut state.graph)
        };
    }
}

pub fn show_state(state: &mut state::AppState, _ui: &mut egui::Ui, ctx: &egui::Context) {
    let output_label = |val: &ValueType| match val {
        ValueType::Image { value } => Some(format!(
            "Image of dimension {}x{}",
            value.size[0], value.size[1]
        )),
        ValueType::ImageFetcher { value } => Some(format!(
            "Image of dimension {}x{}",
            value.image.size[0], value.image.size[1]
        )),
        ValueType::Slice { value } => Some(format!(
            "Image slice of dimension {}x{}",
            value.size[0], value.size[1]
        )),
        ValueType::_Color { value } => Some(format!(
            "Color of value ({}, {}, {})",
            value.r(),
            value.g(),
            value.b()
        )),
        ValueType::Scalar { value } => Some(format!("Scalar of value {}", value)),
    };

    let outputs_cache = state
        .graph
        .user_state
        .outputs_cache
        .iter()
        .map(|(id, val)| format!("{:?} <=> {:?}", &id, output_label(val)))
        .reduce(|a, b| format!("{}\n{}", a, b));

    if let Some(debug) = outputs_cache {
        ctx.debug_painter().text(
            egui::pos2(30.0, 30.0),
            egui::Align2::LEFT_TOP,
            format!("GRAPH STATE\n{}", debug),
            TextStyle::Monospace.resolve(&ctx.style()),
            egui::Color32::RED,
        );
    }
}

pub fn show_background(ui: &mut egui::Ui) {
    let color = if ui.visuals().dark_mode {
        egui::Color32::from_additive_luminance(20)
    } else {
        egui::Color32::from_black_alpha(60)
    };

    let radius = 3.0;
    let spacing = 30.0;

    let mut shapes = vec![];

    egui::Frame::canvas(ui.style())
        .fill(egui::Color32::TRANSPARENT)
        .stroke(egui::Stroke::none())
        .show(ui, |ui| {
            ui.ctx().request_repaint();

            let next = ui.next_widget_position();
            let size = ui.available_size();
            let start = egui::pos2(next.x + spacing * 0.25, next.y + spacing * 0.25);

            let number_x = (size.x / spacing).ceil() as i32 + 1;
            let number_y = (size.y / spacing).ceil() as i32 + 1;

            for x in 0..number_x {
                for y in 0..number_y {
                    let pos_x = start.x + (x as f32) * spacing;
                    let pos_y = start.y + (y as f32) * spacing;
                    shapes.push(egui::epaint::Shape::circle_filled(
                        egui::pos2(pos_x, pos_y),
                        radius,
                        color,
                    ));
                }
            }

            ui.painter().extend(shapes);
        });
}
