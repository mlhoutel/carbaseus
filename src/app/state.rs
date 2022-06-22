use crate::app::components::graph::node;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AppState {
    #[serde(skip)] // opt-out serialization
    pub graph: node::EditorState,

    #[serde(skip)] // opt-out serialization
    pub first_loop: bool,

    pub auto_compute: bool,

    // Display
    pub d_settings: bool,
    pub d_about: bool,
    pub d_state: bool,
    pub i_pannel: InputPanel,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            graph: node::EditorState::new(1.0, node::GraphState::default()),
            first_loop: true,
            auto_compute: false,

            // Display
            d_settings: false,
            d_about: false,
            d_state: true,
            i_pannel: InputPanel::default(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug)]
pub enum InputPanel {
    Fetcher,
    Painter,
    Uploader,
}

impl Default for InputPanel {
    fn default() -> Self {
        Self::Fetcher
    }
}
