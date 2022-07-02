use std::borrow::Cow;
use std::collections::HashMap;

use eframe::egui::{self, DragValue};
use egui::epaint::{Color32, ColorImage};
use egui_extras::image::RetainedImage;
use egui_node_graph::*;

use crate::app::components::display;
use crate::app::components::input::image_fetcher::Fetcher;
use crate::app::math::fft;
use crate::app::math::image::{
    brighten_image, contrast_image, flip_image, hue_rotate_image, image_blur, image_to_gray,
    invert_colors_image, rotate_image, ImageSlice, SliceColor,
};
use crate::app::state;

const LABEL_IMAGE_IN: &str = "image_in";
const LABEL_IMAGE_OUT: &str = "image_out";

const LABEL_INPUT_IMAGE_OUT: &str = "input_image";

const LABEL_SLICE_R_IN: &str = "slice_r_in";
const LABEL_SLICE_G_IN: &str = "slice_g_in";
const LABEL_SLICE_B_IN: &str = "slice_b_in";
const LABEL_SLICE_S_IN: &str = "slice_s_in";

const LABEL_SLICE_R_OUT: &str = "slice_r_out";
const LABEL_SLICE_G_OUT: &str = "slice_g_out";
const LABEL_SLICE_B_OUT: &str = "slice_b_out";
const LABEL_SLICE_S_OUT: &str = "slice_s_out";

const LABEL_BOOLEAN_H_IN: &str = "input_h_in";
const LABEL_BOOLEAN_V_IN: &str = "input_v_in";

const LABEL_SCALAR_SIGMA_IN: &str = "scalar_sigma";
const _LABEL_COLOR_OUT: &str = "color_out";
const _LABEL_SCALAR_OUT: &str = "scala_out";

const LABEL_INTEGER_SIGMA_IN: &str = "integer_sigma";

pub type _Node = egui_node_graph::Node<NodeData>;
pub type NodeId = egui_node_graph::id_type::NodeId;
pub type OutputId = egui_node_graph::id_type::OutputId;
pub type _InputId = egui_node_graph::id_type::InputId;

// ========= First, define your user data types =============

/// The NodeData holds a custom data struct inside each node. It's useful to
/// store additional information that doesn't live in parameters. For this
/// example, the node data stores the template (i.e. the "type") of the node.
pub struct NodeData {
    pub template: NodeTemplate,
}

/// `DataType`s are what defines the possible range of connections when
/// attaching two ports together. The graph UI will make sure to not allow
/// attaching incompatible datatypes.
#[derive(PartialEq, Eq)]
pub enum DataType {
    Image,
    Slice,
    Color,
    Scalar,
    Integer,
    Boolean,
}

/// In the graph, input parameters can optionally have a constant value. This
/// value can be directly edited in a widget inside the node itself.
///
/// There will usually be a correspondence between DataTypes and ValueTypes. But
/// this library makes no attempt to check this consistency. For instance, it is
/// up to the user code in this example to make sure no parameter is created
/// with a DataType of Scalar and a ValueType of Vec2.
#[derive(Clone)]
pub enum ValueType {
    ImageFetcher { value: Fetcher },
    Image { value: ColorImage },
    Slice { value: ImageSlice },
    _Color { value: Color32 },
    Scalar { value: f32 },
    Integer { value: i32 },
    Boolean { value: bool },
}

impl ValueType {
    /// Tries to downcast this value type to an image
    pub fn try_to_image(self) -> anyhow::Result<ColorImage> {
        match self {
            ValueType::Image { value } => Ok(value),
            ValueType::Slice { value } => Ok(ImageSlice::to_image(&value)),
            ValueType::ImageFetcher { value } => Ok(value.image),
            _ => {
                anyhow::bail!("Invalid cast to ColorImage".to_string())
            }
        }
    }

    /// Tries to downcast this value type to a slice
    pub fn try_to_slice(self, color: Option<SliceColor>) -> anyhow::Result<ImageSlice> {
        match self {
            ValueType::Slice { value } => Ok(value),
            ValueType::Image { value } => {
                if let Some(slice_color) = color {
                    Ok(ImageSlice::from_image(value, slice_color))
                } else {
                    Ok(ImageSlice::from_image(value, SliceColor::Gray))
                }
            }
            _ => {
                anyhow::bail!("Invalid cast to Color32".to_string())
            }
        }
    }

    /// Tries to downcast this value type to a color
    pub fn _try_to_color(self) -> anyhow::Result<Color32> {
        if let ValueType::_Color { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast to Color32".to_string())
        }
    }

    /// Tries to downcast this value type to a scalar
    pub fn try_to_scalar(self) -> anyhow::Result<f32> {
        if let ValueType::Scalar { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast to scalar".to_string())
        }
    }

    /// Tries to downcast this value type to a boolean
    pub fn try_to_boolean(self) -> anyhow::Result<bool> {
        if let ValueType::Boolean { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast to boolean".to_string())
        }
    }

    /// Tries to downcast this value type to an integer
    pub fn try_to_integer(self) -> anyhow::Result<i32> {
        if let ValueType::Integer { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast to integer".to_string())
        }
    }
}

/// NodeTemplate is a mechanism to define node templates. It's what the graph
/// will display in the "new node" popup. The user code needs to tell the
/// library how to convert a NodeTemplate into a Node.
#[derive(Clone, Copy)]
pub enum NodeTemplate {
    // Input
    ImageFetcher,

    // Transformation
    GrayScales,
    ImageToSlice,
    SliceToImage,

    // Processing
    GaussianBlur,
    FourierSpace,
    BrightenImage,
    ContrastImage,
    InvertImage,
    HueRotate,
    FlipImage,
    RotateImage,
}

/// The response type is used to encode side-effects produced when drawing a
/// node in the graph. Most side-effects (creating new nodes, deleting existing
/// nodes, handling connections...) are already handled by the library, but this
/// mechanism allows creating additional side effects from user code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Response {
    ImageFetched,
    ScalarChanged,
    IntegerChanged,
    BooleanChanged,
}

type OutputsCache = HashMap<OutputId, ValueType>;
type OutputsImages = HashMap<OutputId, RetainedImage>;

/// The graph 'global' state. This state struct is passed around to the node and
/// parameter drawing callbacks. The contents of this struct are entirely up to
/// the user. For this example, we use it to keep track of the 'active' node.
#[derive(Default)]
pub struct GraphState {
    pub outputs_cache: OutputsCache,
    pub outputs_images: OutputsImages,
}

// =========== Then, you need to implement some traits ============

// A trait for the data types, to tell the library how to display them
impl DataTypeTrait<GraphState> for DataType {
    fn data_type_color(&self, _user_state: &GraphState) -> egui::Color32 {
        match self {
            DataType::Image => Color32::from_rgb(38, 109, 211),
            DataType::Color => Color32::from_rgb(238, 207, 109),
            DataType::Slice => Color32::from_rgb(214, 65, 10),
            DataType::Scalar => Color32::from_rgb(24, 165, 37),
            DataType::Integer => Color32::from_rgb(24, 165, 37),
            DataType::Boolean => Color32::from_rgb(24, 165, 37),
        }
    }

    fn name(&self) -> Cow<'_, str> {
        match self {
            DataType::Image => Cow::Borrowed("image"),
            DataType::Color => Cow::Borrowed("color"),
            DataType::Slice => Cow::Borrowed("slice"),
            DataType::Scalar => Cow::Borrowed("scalar"),
            DataType::Integer => Cow::Borrowed("integer"),
            DataType::Boolean => Cow::Borrowed("boolean"),
        }
    }
}

// A trait for the node kinds, which tells the library how to build new nodes
// from the templates in the node finder
impl NodeTemplateTrait for NodeTemplate {
    type NodeData = NodeData;
    type DataType = DataType;
    type ValueType = ValueType;
    type UserState = GraphState;

    fn node_finder_label(&self) -> &str {
        match self {
            NodeTemplate::ImageFetcher => "Image fetcher",

            NodeTemplate::GrayScales => "Gray scales",
            NodeTemplate::ImageToSlice => "Image to RGB Slice",
            NodeTemplate::SliceToImage => "RGB Slice to Image",

            NodeTemplate::FourierSpace => "Fourier space",

            NodeTemplate::GaussianBlur => "Gaussian blur",
            NodeTemplate::BrightenImage => "Brighten Image",
            NodeTemplate::ContrastImage => "Contrast Image",

            NodeTemplate::InvertImage => "Invert Image",
            NodeTemplate::HueRotate => "Hue Rotate",

            NodeTemplate::FlipImage => "Flip Image",
            NodeTemplate::RotateImage => "Rotate Image",
        }
    }

    fn node_graph_label(&self) -> String {
        // It's okay to delegate this to node_finder_label if you don't want to
        // show different names in the node finder and the node itself.
        self.node_finder_label().into()
    }

    fn user_data(&self) -> NodeData {
        NodeData { template: *self }
    }

    fn build_node(
        &self,
        graph: &mut Graph<NodeData, DataType, ValueType>,
        _user_state: &Self::UserState,
        node_id: NodeId,
    ) {
        // The nodes are created empty by default. This function needs to take
        // care of creating the desired inputs and outputs based on the template

        // We define some closures here to avoid boilerplate. Note that this is
        // entirely optional.

        let input_fetcher_image = |graph: &mut ProcessGraph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(), // This is the name of the parameter
                DataType::Image,  // The data type for this input
                ValueType::ImageFetcher {
                    value: Fetcher::default(),
                }, // The value type for this input
                InputParamKind::ConstantOnly, // The input parameter kind.
                true,
            );
        };

        let input_image = |graph: &mut ProcessGraph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(), // This is the name of the parameter
                DataType::Image,  // The data type for this input
                ValueType::Image {
                    value: ColorImage::new([1, 1], Color32::BLACK),
                }, // The value type for this input
                InputParamKind::ConnectionOnly, // The input parameter kind.
                true,
            );
        };

        let input_slice = |graph: &mut ProcessGraph, name: &str, color: SliceColor| {
            graph.add_input_param(
                node_id,
                name.to_string(), // This is the name of the parameter
                DataType::Slice,  // The data type for this input
                ValueType::Slice {
                    value: ImageSlice::new(color, [1, 1]),
                }, // The value type for this input
                InputParamKind::ConnectionOnly, // The input parameter kind.
                true,
            );
        };
        /*
        let input_color = |graph: &mut ProcessGraph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                DataType::Color,
                ValueType::Color { value: Color32::BLACK },
                InputParamKind::ConnectionOrConstant,
                true,
            );
        };
        */

        let input_scalar = |graph: &mut ProcessGraph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                DataType::Scalar,
                ValueType::Scalar { value: 0.0 },
                InputParamKind::ConstantOnly,
                true,
            );
        };

        let input_integer = |graph: &mut ProcessGraph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                DataType::Integer,
                ValueType::Integer { value: 0 },
                InputParamKind::ConstantOnly,
                true,
            );
        };

        let input_boolean = |graph: &mut ProcessGraph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(),  // This is the name of the parameter
                DataType::Boolean, // The data type for this input
                ValueType::Boolean { value: false }, // The value type for this input
                InputParamKind::ConstantOnly, // The input parameter kind.
                true,
            );
        };

        let _output_image_flat = |graph: &mut ProcessGraph, _name: &str| {
            graph.add_output_param(node_id, "".to_string(), DataType::Image);
        };

        let output_image = |graph: &mut ProcessGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), DataType::Image);
        };

        let output_slice = |graph: &mut ProcessGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), DataType::Slice);
        };

        let _output_color = |graph: &mut ProcessGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), DataType::Color);
        };

        let _output_scalar = |graph: &mut ProcessGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), DataType::Scalar);
        };

        match self {
            NodeTemplate::ImageFetcher => {
                input_fetcher_image(graph, LABEL_IMAGE_IN);
                output_image(graph, LABEL_INPUT_IMAGE_OUT);
            }
            NodeTemplate::GrayScales => {
                input_image(graph, LABEL_IMAGE_IN);
                output_slice(graph, LABEL_SLICE_S_OUT);
            }
            NodeTemplate::GaussianBlur => {
                input_image(graph, LABEL_IMAGE_IN);
                input_scalar(graph, LABEL_SCALAR_SIGMA_IN);
                output_image(graph, LABEL_IMAGE_OUT);
            }
            NodeTemplate::FourierSpace => {
                input_slice(graph, LABEL_SLICE_S_IN, SliceColor::Gray);
                output_slice(graph, LABEL_SLICE_S_OUT);
            }
            NodeTemplate::SliceToImage => {
                input_slice(graph, LABEL_SLICE_R_IN, SliceColor::Red);
                input_slice(graph, LABEL_SLICE_G_IN, SliceColor::Green);
                input_slice(graph, LABEL_SLICE_B_IN, SliceColor::Blue);
                output_image(graph, LABEL_IMAGE_OUT);
            }
            NodeTemplate::ImageToSlice => {
                input_image(graph, LABEL_IMAGE_IN);
                output_slice(graph, LABEL_SLICE_R_OUT);
                output_slice(graph, LABEL_SLICE_G_OUT);
                output_slice(graph, LABEL_SLICE_B_OUT);
            }
            NodeTemplate::BrightenImage => {
                input_image(graph, LABEL_IMAGE_IN);
                input_scalar(graph, LABEL_SCALAR_SIGMA_IN);
                output_image(graph, LABEL_IMAGE_OUT);
            }
            NodeTemplate::ContrastImage => {
                input_image(graph, LABEL_IMAGE_IN);
                input_scalar(graph, LABEL_SCALAR_SIGMA_IN);
                output_image(graph, LABEL_IMAGE_OUT);
            }
            NodeTemplate::InvertImage => {
                input_image(graph, LABEL_IMAGE_IN);
                output_image(graph, LABEL_IMAGE_OUT);
            }
            NodeTemplate::HueRotate => {
                input_image(graph, LABEL_IMAGE_IN);
                input_scalar(graph, LABEL_SCALAR_SIGMA_IN);
                output_image(graph, LABEL_IMAGE_OUT);
            }
            NodeTemplate::FlipImage => {
                input_image(graph, LABEL_IMAGE_IN);
                input_boolean(graph, LABEL_BOOLEAN_H_IN);
                input_boolean(graph, LABEL_BOOLEAN_V_IN);
                output_image(graph, LABEL_IMAGE_OUT);
            }
            NodeTemplate::RotateImage => {
                input_image(graph, LABEL_IMAGE_IN);
                input_integer(graph, LABEL_INTEGER_SIGMA_IN);
                output_image(graph, LABEL_IMAGE_OUT);
            }
        }
    }
}

pub struct AllNodeTemplates;

impl NodeTemplateIter for AllNodeTemplates {
    type Item = NodeTemplate;

    fn all_kinds(&self) -> Vec<Self::Item> {
        // This function must return a list of node kinds, which the node finder
        // will use to display it to the user. Crates like strum can reduce the
        // boilerplate in enumerating all variants of an enum.
        vec![
            NodeTemplate::ImageFetcher,
            NodeTemplate::GrayScales,
            NodeTemplate::GaussianBlur,
            NodeTemplate::FourierSpace,
            NodeTemplate::SliceToImage,
            NodeTemplate::ImageToSlice,
            NodeTemplate::BrightenImage,
            NodeTemplate::ContrastImage,
            NodeTemplate::InvertImage,
            NodeTemplate::HueRotate,
        ]
    }
}

impl WidgetValueTrait for ValueType {
    type Response = Response;
    fn value_widget(&mut self, param_name: &str, ui: &mut egui::Ui) -> Vec<Response> {
        let mut responses: Vec<Response> = Vec::new();

        // This trait is used to tell the library which UI to display for the
        // inline parameter widgets.
        match self {
            ValueType::ImageFetcher { value } => {
                if value.show(ui) {
                    responses.push(Response::ImageFetched); // Notify when input image changes
                }
            }
            ValueType::Image { value: _ } => {}
            ValueType::_Color { value: _ } => {}
            ValueType::Slice { value: _ } => {}
            ValueType::Scalar { value } => {
                ui.horizontal(|ui| {
                    ui.label(param_name);

                    let drag_value =
                        ui.add(DragValue::new(value).speed(0.1).clamp_range(0.0..=1000.0));
                    if drag_value.drag_released() || drag_value.lost_focus() {
                        responses.push(Response::ScalarChanged); // Notify when scalar changes
                    }
                });
            }
            ValueType::Integer { value } => {
                ui.horizontal(|ui| {
                    ui.label(param_name);

                    let drag_value = ui.add(DragValue::new(value).clamp_range(0.0..=1000.0));
                    if drag_value.drag_released() || drag_value.lost_focus() {
                        responses.push(Response::IntegerChanged); // Notify when scalar changes
                    }
                });
            }
            ValueType::Boolean { value } => {
                if ui.checkbox(value, param_name).changed() {
                    responses.push(Response::BooleanChanged); // Notify when boolean changes
                }
            }
        }
        responses
    }
}

impl UserResponseTrait for Response {}
impl NodeDataTrait for NodeData {
    type Response = Response;
    type UserState = GraphState;
    type DataType = DataType;
    type ValueType = ValueType;

    // This method will be called when drawing each node. This allows adding
    // extra ui elements inside the nodes. In this case, we create an "active"
    // button which introduces the concept of having an active node in the
    // graph. This is done entirely from user code with no modifications to the
    // node graph library.
    fn bottom_ui(
        &self,
        ui: &mut egui::Ui,
        node_id: NodeId,
        _graph: &Graph<NodeData, DataType, ValueType>,
        user_state: &Self::UserState,
    ) -> std::vec::Vec<NodeResponse<Response, NodeData>> {
        // This logic is entirely up to the user. In this case, we check if the
        // current node we're drawing is the active one, by comparing against
        // the value stored in the global user state, and draw different button
        // UIs based on that.
        let responses: Vec<NodeResponse<Response, NodeData>> = vec![];

        let find_node = _graph.nodes.iter().find(|(id, _data)| *id == node_id);

        let mut first_header = true;

        // We try to find the current node to evaluate its outputs
        if let Some((_node_id, node_data)) = find_node {
            // Check if we have an output with an image or slice output
            for (label, id) in node_data.outputs.iter() {
                if let Some(image) = user_state.outputs_images.get(id) {
                    egui::CollapsingHeader::new(format!("Image for {label}"))
                        .default_open(first_header)
                        .show(ui, |ui| {
                            display::image_frame::show(ui, image);
                        });

                    first_header = false;
                }
            }
        }
        responses
    }

    fn titlebar_color(
        &self,
        _ui: &egui::Ui,
        _node_id: NodeId,
        _graph: &Graph<Self, Self::DataType, Self::ValueType>,
        _user_state: &Self::UserState,
    ) -> Option<egui::Color32> {
        None
    }
}

pub type ProcessGraph = Graph<NodeData, DataType, ValueType>;
pub type EditorState = GraphEditorState<NodeData, DataType, ValueType, NodeTemplate, GraphState>;

pub fn create_node(state: &mut state::AppState, node_kind: NodeTemplate, pos: egui::Pos2) {
    let new_node = state.graph.graph.add_node(
        node_kind.node_graph_label(),
        node_kind.user_data(),
        |graph, node_id| node_kind.build_node(graph, &state.graph.user_state, node_id),
    );

    state.graph.node_positions.insert(new_node, pos);

    state.graph.node_order.push(new_node);
}

pub fn evaluate_graph(state: &mut EditorState) {
    // Reset the computed cache & images
    state.user_state.outputs_cache.clear();
    state.user_state.outputs_images.clear();

    // Compute and store the result for each node
    for (id, _node) in state.graph.nodes.iter() {
        let _result = evaluate_node(&state.graph, id, &mut state.user_state.outputs_cache);
    }

    // Then process all the resulting image to prepare the rendering
    for (key, value) in state.user_state.outputs_cache.clone().into_iter() {
        if let Ok(image) = ValueType::try_to_image(value) {
            state.user_state.outputs_images.insert(
                key,
                RetainedImage::from_color_image(
                    format!("Retained image for the output {:?}", key),
                    image,
                ),
            );
        }
    }
}

/// Recursively evaluates all dependencies of this node, then evaluates the node itself.
pub fn evaluate_node(
    graph: &ProcessGraph,
    node_id: NodeId,
    outputs_cache: &mut OutputsCache,
) -> anyhow::Result<ValueType> {
    // To solve a similar problem as creating node types above, we define an
    // Evaluator as a convenience. It may be overkill for this small example,
    // but something like this makes the code much more readable when the
    // number of nodes starts growing.

    struct Evaluator<'a> {
        graph: &'a ProcessGraph,
        outputs_cache: &'a mut OutputsCache,
        node_id: NodeId,
    }

    impl<'a> Evaluator<'a> {
        fn new(
            graph: &'a ProcessGraph,
            outputs_cache: &'a mut OutputsCache,
            node_id: NodeId,
        ) -> Self {
            Self {
                graph,
                outputs_cache,
                node_id,
            }
        }

        fn evaluate_input(&mut self, name: &str) -> anyhow::Result<ValueType> {
            // Calling `evaluate_input` recursively evaluates other nodes in the
            // graph until the input value for a paramater has been computed.
            evaluate_input(self.graph, self.node_id, name, self.outputs_cache)
        }

        fn populate_output(&mut self, name: &str, value: ValueType) -> anyhow::Result<ValueType> {
            // After computing an output, we don't just return it, but we also
            // populate the outputs cache with it. This ensures the evaluation
            // only ever computes an output once.
            //
            // The return value of the function is the "final" output of the
            // node, the thing we want to get from the evaluation. The example
            // would be slightly more contrived when we had multiple output
            // values, as we would need to choose which of the outputs is the
            // one we want to return. Other outputs could be used as
            // intermediate values.
            //
            // Note that this is just one possible semantic interpretation of
            // the graphs, you can come up with your own evaluation semantics!
            populate_output(self.graph, self.outputs_cache, self.node_id, name, value)
        }
        fn input_image(&mut self, name: &str) -> anyhow::Result<ColorImage> {
            self.evaluate_input(name)?.try_to_image()
        }
        fn input_slice(
            &mut self,
            name: &str,
            color: Option<SliceColor>,
        ) -> anyhow::Result<ImageSlice> {
            self.evaluate_input(name)?.try_to_slice(color)
        }
        fn input_scalar(&mut self, name: &str) -> anyhow::Result<f32> {
            self.evaluate_input(name)?.try_to_scalar()
        }
        fn input_integer(&mut self, name: &str) -> anyhow::Result<i32> {
            self.evaluate_input(name)?.try_to_integer()
        }
        fn input_boolean(&mut self, name: &str) -> anyhow::Result<bool> {
            self.evaluate_input(name)?.try_to_boolean()
        }
        fn output_image(&mut self, name: &str, value: ColorImage) -> anyhow::Result<ValueType> {
            self.populate_output(name, ValueType::Image { value })
        }
        fn output_slice(&mut self, name: &str, value: ImageSlice) -> anyhow::Result<ValueType> {
            self.populate_output(name, ValueType::Slice { value })
        }
        /*
        fn output_scalar(&mut self, name: &str, value: f32) -> anyhow::Result<ValueType> {
            self.populate_output(name, ValueType::Scalar { value })
        }
        */
    }

    let node = &graph[node_id];
    let mut evaluator = Evaluator::new(graph, outputs_cache, node_id);
    match node.user_data.template {
        NodeTemplate::ImageFetcher => {
            let image = evaluator.input_image(LABEL_IMAGE_IN)?;
            evaluator.output_image(LABEL_INPUT_IMAGE_OUT, image)
        }
        NodeTemplate::FourierSpace => {
            let image = evaluator.input_slice(LABEL_SLICE_S_IN, None)?;

            let computed = fft::mat_fft(image);

            evaluator.output_slice(LABEL_SLICE_S_OUT, computed)
        }
        NodeTemplate::GaussianBlur => {
            let image = evaluator.input_image(LABEL_IMAGE_IN)?;
            let sigma = evaluator.input_scalar(LABEL_SCALAR_SIGMA_IN)?;

            let blurred = image_blur(&image, sigma);

            evaluator.output_image(LABEL_IMAGE_OUT, blurred)
        }
        NodeTemplate::GrayScales => {
            let image = evaluator.input_image(LABEL_IMAGE_IN)?;

            let gray_scale = image_to_gray(&image);

            evaluator.output_slice(LABEL_SLICE_S_OUT, gray_scale)
        }
        NodeTemplate::SliceToImage => {
            let slice_r = evaluator.input_slice(LABEL_SLICE_R_IN, Some(SliceColor::Red))?;
            let slice_g = evaluator.input_slice(LABEL_SLICE_G_IN, Some(SliceColor::Green))?;
            let slice_b = evaluator.input_slice(LABEL_SLICE_B_IN, Some(SliceColor::Blue))?;

            let widths = [slice_r.size[0], slice_g.size[0], slice_b.size[0]];
            let heights = [slice_r.size[1], slice_g.size[1], slice_b.size[1]];

            let max_width = widths.iter().max().unwrap();
            let max_height = heights.iter().max().unwrap();

            let size = [*max_width, *max_height];
            let mut image = ColorImage::new(size, Color32::BLACK);

            for (id, px) in slice_r.pixels.iter().enumerate() {
                image.pixels[id][0] = *px
            }
            for (id, px) in slice_g.pixels.iter().enumerate() {
                image.pixels[id][1] = *px
            }
            for (id, px) in slice_b.pixels.iter().enumerate() {
                image.pixels[id][2] = *px
            }

            evaluator.output_image(LABEL_IMAGE_OUT, image)
        }
        NodeTemplate::ImageToSlice => {
            let image = evaluator.input_image(LABEL_IMAGE_IN)?;

            let mut slice_r = ImageSlice::new(SliceColor::Red, image.size);
            let mut slice_g = ImageSlice::new(SliceColor::Green, image.size);
            let mut slice_b = ImageSlice::new(SliceColor::Blue, image.size);

            for (id, px) in image.pixels.iter().enumerate() {
                slice_r.pixels[id] = px.r();
                slice_g.pixels[id] = px.g();
                slice_b.pixels[id] = px.b();
            }

            let _res_r = evaluator.output_slice(LABEL_SLICE_R_OUT, slice_r);
            let _res_g = evaluator.output_slice(LABEL_SLICE_G_OUT, slice_g);
            evaluator.output_slice(LABEL_SLICE_B_OUT, slice_b)
        }
        NodeTemplate::BrightenImage => {
            let image = evaluator.input_image(LABEL_IMAGE_IN)?;
            let sigma = evaluator.input_scalar(LABEL_SCALAR_SIGMA_IN)?;

            let brightened = brighten_image(&image, sigma);

            evaluator.output_image(LABEL_IMAGE_OUT, brightened)
        }
        NodeTemplate::ContrastImage => {
            let image = evaluator.input_image(LABEL_IMAGE_IN)?;
            let sigma = evaluator.input_scalar(LABEL_SCALAR_SIGMA_IN)?;

            let contrasted = contrast_image(&image, sigma);

            evaluator.output_image(LABEL_IMAGE_OUT, contrasted)
        }
        NodeTemplate::InvertImage => {
            let image = evaluator.input_image(LABEL_IMAGE_IN)?;

            let inverted = invert_colors_image(&image);

            evaluator.output_image(LABEL_IMAGE_OUT, inverted)
        }
        NodeTemplate::HueRotate => {
            let image = evaluator.input_image(LABEL_IMAGE_IN)?;
            let sigma = evaluator.input_scalar(LABEL_SCALAR_SIGMA_IN)?;

            let rotated = hue_rotate_image(&image, sigma);

            evaluator.output_image(LABEL_IMAGE_OUT, rotated)
        }
        NodeTemplate::FlipImage => {
            let image = evaluator.input_image(LABEL_IMAGE_IN)?;
            let horizontal = evaluator.input_boolean(LABEL_BOOLEAN_H_IN)?;
            let vertical = evaluator.input_boolean(LABEL_BOOLEAN_V_IN)?;

            let flipped = flip_image(&image, horizontal, vertical);

            evaluator.output_image(LABEL_IMAGE_OUT, flipped)
        }
        NodeTemplate::RotateImage => {
            let image = evaluator.input_image(LABEL_IMAGE_IN)?;
            let sigma = evaluator.input_integer(LABEL_INTEGER_SIGMA_IN)?;

            let rotated = rotate_image(&image, sigma);

            evaluator.output_image(LABEL_IMAGE_OUT, rotated)
        }
    }
}

fn populate_output(
    graph: &ProcessGraph,
    outputs_cache: &mut OutputsCache,
    node_id: NodeId,
    param_name: &str,
    value: ValueType,
) -> anyhow::Result<ValueType> {
    let output_id = graph[node_id].get_output(param_name)?;
    outputs_cache.insert(output_id, value.clone());

    Ok(value)
}

// Evaluates the input value of
fn evaluate_input(
    graph: &ProcessGraph,
    node_id: NodeId,
    param_name: &str,
    outputs_cache: &mut OutputsCache,
) -> anyhow::Result<ValueType> {
    let input_id = graph[node_id].get_input(param_name)?;

    // The output of another node is connected.
    if let Some(other_output_id) = graph.connection(input_id) {
        // The value was already computed due to the evaluation of some other
        // node. We simply return value from the cache.
        if let Some(other_value) = outputs_cache.get(&other_output_id) {
            Ok(other_value.clone())
        }
        // This is the first time encountering this node, so we need to
        // recursively evaluate it.
        else {
            // Calling this will populate the cache
            evaluate_node(graph, graph[other_output_id].node, outputs_cache)?;

            // Now that we know the value is cached, return it
            Ok(outputs_cache
                .get(&other_output_id)
                .expect("Cache should be populated")
                .clone())
        }
    }
    // No existing connection, take the inline value instead.
    else {
        Ok(graph[input_id].value.clone())
    }
}
