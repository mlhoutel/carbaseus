use std::collections::HashMap;

use egui::ColorImage;
use egui_extras::RetainedImage;
use egui_node_graph::NodeId;

use crate::app::components::graph::node;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AppState {
    #[serde(skip)] // opt-out serialization
    pub graph: node::EditorState,

    #[serde(skip)] // opt-out serialization
    pub selected_node: SelectedNode,

    #[serde(skip)] // opt-out serialization
    pub first_loop: bool,

    pub auto_compute: bool,

    // Display
    pub d_settings: bool,
    pub d_about: bool,
    pub d_state: bool,
    pub o_pannel: OutputPanel,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            graph: node::EditorState::new(1.0, node::GraphState::default()),
            selected_node: SelectedNode::default(),
            first_loop: true,
            auto_compute: true,

            // Display
            d_settings: false,
            d_about: false,
            d_state: false,
            o_pannel: OutputPanel::default(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug)]
pub enum OutputPanel {
    Image,
    Spectrum,
}

impl Default for OutputPanel {
    fn default() -> Self {
        Self::Image
    }
}

pub struct SelectedNode {
    pub node_id: Option<NodeId>,
    pub color_image: Option<ColorImage>,
    pub retained_image: Option<RetainedImage>,
    pub spectrum: Vec<HashMap<u8, usize>>,
}

impl Default for SelectedNode {
    fn default() -> Self {
        Self {
            node_id: None,
            color_image: None,
            retained_image: None,
            spectrum: vec![HashMap::default(); 3],
        }
    }
}
