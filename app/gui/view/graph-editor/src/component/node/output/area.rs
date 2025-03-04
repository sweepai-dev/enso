//! Implements the segmented output port area.

use crate::prelude::*;
use ensogl::display::traits::*;

use crate::component::node;
use crate::component::node::input;
use crate::component::node::output::port;
use crate::tooltip;
use crate::Type;

use enso_config::ARGS;
use enso_frp as frp;
use enso_frp;
use ensogl::animation::hysteretic::HystereticAnimation;
use ensogl::application::Application;
use ensogl::data::color;
use ensogl::display;
use ensogl::display::shape::StyleWatch;
use ensogl_component::text;
use ensogl_hardcoded_theme as theme;



// =================
// === Constants ===
// =================

const HIDE_DELAY_DURATION_MS: f32 = 150.0;
const SHOW_DELAY_DURATION_MS: f32 = 150.0;



// ================
// === SpanTree ===
// ================

use span_tree::node::Ref as PortRef;
use span_tree::PortId;
use span_tree::SpanTree;



// ==================
// === Expression ===
// ==================

/// Specialized version of `node::Expression`, containing the port information.
#[derive(Default)]
#[allow(missing_docs)]
pub struct Expression {
    pub code:            Option<String>,
    pub span_tree:       SpanTree,
    /// This field contains the type of the whole input expression. This is needed due to a bug in
    /// engine: https://github.com/enso-org/enso/issues/1038.
    pub whole_expr_type: Option<Type>,
    pub whole_expr_id:   Option<ast::Id>,
}

impl Expression {
    #[allow(missing_docs)] // FIXME[everyone] All pub functions should have docs.
    pub fn code(&self) -> String {
        self.code.clone().unwrap_or_default()
    }
}

impl Deref for Expression {
    type Target = SpanTree;
    fn deref(&self) -> &Self::Target {
        &self.span_tree
    }
}

impl DerefMut for Expression {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.span_tree
    }
}

impl Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Expression({})", self.code.clone().unwrap_or_default())
    }
}


// === Conversions ===

impl From<node::Expression> for Expression {
    #[profile(Debug)]
    fn from(expr: node::Expression) -> Self {
        let code = expr.pattern.clone();
        let whole_expr_type = expr.input_span_tree.root.tp().map(|t| t.to_owned().into());
        let whole_expr_id = expr.whole_expression_id;
        let span_tree = expr.output_span_tree;
        Expression { code, span_tree, whole_expr_type, whole_expr_id }
    }
}



// =============
// === Model ===
// =============

// FIXME: Update to `define_endpoints_2`. Note that `Model` must not own the `api::Private`,
// because `api::Private` owns the network, which contains (strong) references to the model.
ensogl::define_endpoints! {
    Input {
        set_size                  (Vector2),
        set_hover                 (bool),
        set_expression            (node::Expression),
        set_expression_visibility (bool),
        set_type_label_visibility (bool),

        /// Set the expression USAGE type. This is not the definition type, which can be set with
        /// `set_expression` instead. In case the usage type is set to None, ports still may be
        /// colored if the definition type was present.
        set_expression_usage_type (ast::Id,Option<Type>),
    }

    Output {
        on_port_press               (PortId),
        on_port_hover               (Switch<PortId>),
        port_size_multiplier        (f32),
        body_hover                  (bool),
        type_label_visibility       (bool),
        expression_label_visibility (bool),
        tooltip                     (tooltip::Style),
        size                        (Vector2),
    }
}

/// Internal model of the port area.
#[derive(Debug, display::Object)]
pub struct Model {
    app:            Application,
    display_object: display::object::Instance,
    ports:          display::object::Instance,
    port_models:    RefCell<Vec<port::Model>>,
    label:          text::Text,
    expression:     RefCell<Expression>,
    id_ports_map:   RefCell<HashMap<ast::Id, usize>>,
    styles:         StyleWatch,
    frp:            FrpEndpoints,
}

impl Model {
    /// Constructor.
    #[profile(Debug)]
    pub fn new(app: &Application, frp: &Frp) -> Self {
        let display_object = display::object::Instance::new_named("output");
        let ports = display::object::Instance::new();
        let port_models = default();
        let label = app.new_view::<text::Text>();
        let id_ports_map = default();
        let expression = default();
        let styles = StyleWatch::new(&app.display.default_scene.style_sheet);
        let frp = frp.output.clone_ref();
        display_object.add_child(&label);
        display_object.add_child(&ports);
        Self {
            app: app.clone_ref(),
            display_object,
            ports,
            port_models,
            label,
            expression,
            id_ports_map,
            styles,
            frp,
        }
        .init(app)
    }

    #[profile(Debug)]
    fn init(self, app: &Application) -> Self {
        // FIXME[WD]: Depth sorting of labels to in front of the mouse pointer. Temporary solution.
        // It needs to be more flexible once we have proper depth management.
        let scene = &app.display.default_scene;
        scene.layers.main.remove(&self.label);
        self.label.add_to_scene_layer(&scene.layers.label);

        let text_color = self.styles.get_color(theme::graph_editor::node::text);
        self.label.set_single_line_mode(true);
        app.commands.set_command_enabled(&self.label, "cursor_move_up", false);
        app.commands.set_command_enabled(&self.label, "cursor_move_up", false);
        self.label.set_property_default(text_color);
        self.label.set_property_default(text::Size(input::area::TEXT_SIZE));
        self.label.remove_all_cursors();

        self.label.set_y(input::area::TEXT_SIZE / 2.0);

        self
    }

    /// Return a list of Node's output ports.
    pub fn ports(&self) -> Vec<port::Model> {
        self.port_models.borrow().clone()
    }

    #[profile(Debug)]
    fn set_label_layer(&self, layer: &display::scene::Layer) {
        self.label.add_to_scene_layer(layer);
    }

    #[profile(Debug)]
    fn set_label(&self, content: impl Into<String>) {
        let node_labels = ARGS.groups.style.options.node_labels.value;
        let str = if node_labels { content.into() } else { default() };
        self.label.set_content(str);
    }

    /// Update expression type for the particular `ast::Id`.
    #[profile(Debug)]
    fn set_expression_usage_type(&self, id: ast::Id, tp: &Option<Type>) {
        let id_ports_map = self.id_ports_map.borrow();
        let Some(index)  = id_ports_map.get(&id).copied() else { return };

        // When this port is the only port, only accept type of the whole expression.
        if index == 0
            && let Some(whole_expr_id) = self.expression.borrow().whole_expr_id
            && whole_expr_id != id
            && id_ports_map.contains_key(&whole_expr_id)
        {
            return
        }

        let port_models = self.port_models.borrow();
        let Some(port) = port_models.get(index) else { return };
        let Some(frp) = &port.frp else { return };
        frp.set_usage_type(tp);
    }

    /// Traverse all span tree nodes that are considered ports.
    #[profile(Debug)]
    fn traverse_borrowed_expression(&self, mut f: impl FnMut(bool, &PortRef)) {
        self.expression.borrow().root_ref().dfs(|node| {
            let is_leaf = node.children.is_empty();
            let is_this = node.is_this();
            let is_argument = node.is_argument();
            let is_a_port = (is_this || is_argument) && is_leaf;
            f(is_a_port, node);
        });
    }

    #[profile(Debug)]
    fn set_size(&self, size: Vector2) {
        self.ports.set_x(size.x / 2.0);
    }

    #[profile(Debug)]
    fn set_label_on_new_expression(&self, expression: &Expression) {
        self.set_label(expression.code());
    }

    #[profile(Debug)]
    fn build_port_shapes_on_new_expression(&self) {
        let mut id_ports_map = HashMap::new();
        let whole_expr_id = self.expression.borrow().whole_expr_id;
        let whole_expr_type = self.expression.borrow().whole_expr_type.clone();

        let mut port_count = 0;
        self.traverse_borrowed_expression(|is_a_port, _| {
            if is_a_port {
                port_count += 1
            }
        });

        let mut models = Vec::new();
        self.traverse_borrowed_expression(|is_a_port, node| {
            let is_a_port = is_a_port || port_count == 0;

            if is_a_port {
                let port_index = models.len();

                if port_count == 0 && let Some(id) = whole_expr_id {
                    id_ports_map.insert(id, port_index);
                }
                if let Some(id) = node.ast_id {
                    id_ports_map.insert(id, port_index);
                }

                let node_tp: Option<Type> = node.tp().cloned().map(|t| t.into());
                let node_tp = if port_count != 0 {
                    node_tp
                } else {
                    node_tp.or_else(|| whole_expr_type.clone())
                };

                let mut model = port::Model::default();
                let span = node.span();
                model.index = span.start.into();
                model.length = span.size();

                let (port_shape, port_frp) =
                    model.init_shape(&self.app, &self.styles, port_index, port_count);

                let port_network = &port_frp.network;
                let source = &self.frp.source;
                let port_id = node.port_id.unwrap_or_default();
                frp::extend! { port_network
                    port_frp.set_size_multiplier <+ self.frp.port_size_multiplier;
                    port_frp.set_type_label_visibility <+ self.frp.type_label_visibility;
                    source.tooltip <+ port_frp.tooltip;
                    port_frp.set_size <+ self.frp.size;
                    source.on_port_hover <+ port_frp.on_hover.map(move |&t| Switch::new(port_id,t));
                    source.on_port_press <+ port_frp.on_press.constant(port_id);
                }

                port_frp.set_type_label_visibility.emit(self.frp.type_label_visibility.value());
                port_frp.set_size.emit(self.frp.size.value());
                port_frp.set_definition_type.emit(node_tp);
                self.ports.add_child(&port_shape);
                models.push(model);
            }
        });
        *self.port_models.borrow_mut() = models;
        *self.id_ports_map.borrow_mut() = id_ports_map;
    }


    #[profile(Debug)]
    fn set_expression(&self, new_expression: impl Into<node::Expression>) {
        let new_expression = Expression::from(new_expression.into());
        self.set_label_on_new_expression(&new_expression);
        *self.expression.borrow_mut() = new_expression;
        self.build_port_shapes_on_new_expression();
    }
}



// ============
// === Area ===
// ============

/// Implements the segmented output port area. Provides shapes that can be attached to a `Node` to
/// add an interactive area with output ports.
///
/// The `Area` facilitate the falling behaviour:
///  * when one of the output ports is hovered, after a set time, all ports are show and the hovered
///    port is highlighted.
///  * when a different port is hovered, it is highlighted immediately.
///  * when none of the ports is hovered all of the `Area` disappear. Note: there is a very small
///    delay for disappearing to allow for smooth switching between ports.
///
/// ## Origin
/// Please note that the origin of the node is on its left side, centered vertically. To learn more
/// about this design decision, please read the docs for the [`node::Node`].
#[derive(Clone, CloneRef, Debug, Deref, display::Object)]
#[allow(missing_docs)]
pub struct Area {
    #[deref]
    pub frp:   Frp,
    #[display_object]
    pub model: Rc<Model>,
}

impl Area {
    #[allow(missing_docs)] // FIXME[everyone] All pub functions should have docs.
    pub fn new(app: &Application) -> Self {
        let frp = Frp::new();
        let model = Rc::new(Model::new(app, &frp));
        let network = &frp.network;
        let label_color = color::Animation::new(network);

        let hysteretic_transition =
            HystereticAnimation::new(network, SHOW_DELAY_DURATION_MS, HIDE_DELAY_DURATION_MS);

        frp::extend! { network

            // === Ports Show / Hide ===

            on_hover_out <- frp.on_port_hover.map(|t| t.is_off()).on_true();
            on_hover_in  <- frp.on_port_hover.map(|t| t.is_on()).on_true();

            hysteretic_transition.to_start <+ on_hover_in;
            hysteretic_transition.to_end   <+ on_hover_out;

            frp.source.port_size_multiplier <+ hysteretic_transition.value;
            eval frp.set_size ((t) model.set_size(*t));
            frp.source.size <+ frp.set_size;

            expr_label_x <- model.label.width.map(|width| -width - input::area::TEXT_OFFSET);
            eval expr_label_x ((x) model.label.set_x(*x));

            frp.source.type_label_visibility <+ frp.set_type_label_visibility;


            // === Expression ===

            eval frp.set_expression            ((a)     model.set_expression(a));
            eval frp.set_expression_usage_type (((a,b)) model.set_expression_usage_type(*a,b));


            // === Label Color ===

            port_hover                             <- frp.on_port_hover.map(|t| t.is_on());
            frp.source.body_hover                  <+ frp.set_hover || port_hover;
            expr_vis                               <- frp.body_hover || frp.set_expression_visibility;
            frp.source.expression_label_visibility <+ expr_vis;

            let label_vis_color = color::Lcha::from(model.styles.get_color(theme::graph_editor::node::text));
            let label_vis_alpha = label_vis_color.alpha;
            label_alpha_tgt          <- expr_vis.map(move |t| if *t {label_vis_alpha} else {0.0} );
            label_color.target_alpha <+ label_alpha_tgt;
            label_color_on_change    <- label_color.value.sample(&frp.set_expression);
            new_label_color          <- any(&label_color.value,&label_color_on_change);
            eval new_label_color ((color) model.label.set_property(.., color::Rgba::from(color)));
        }

        label_color.target_alpha(0.0);
        label_color.target_color(label_vis_color.opaque);

        Self { frp, model }
    }

    /// Set a scene layer for text rendering.
    pub fn set_label_layer(&self, layer: &display::scene::Layer) {
        self.model.set_label_layer(layer);
    }

    #[allow(missing_docs)] // FIXME[everyone] All pub functions should have docs.
    pub fn port_type(&self, port: PortId) -> Option<Type> {
        match port {
            PortId::Ast(id) => {
                let index = *self.model.id_ports_map.borrow().get(&id)?;
                self.model.port_models.borrow().get(index)?.frp.as_ref()?.tp.value()
            }
            _ => None,
        }
    }

    /// Get the expression code for the specified port.
    pub fn port_expression(&self, port: PortId) -> Option<String> {
        match port {
            PortId::Ast(id) => {
                let index = *self.model.id_ports_map.borrow().get(&id)?;
                let port_models = self.model.port_models.borrow();
                let model = port_models.get(index)?;
                let span = enso_text::Range::new(model.index, model.index + model.length);
                Some(self.model.expression.borrow().code.as_ref()?[span].to_owned())
            }
            _ => None,
        }
    }

    #[allow(missing_docs)] // FIXME[everyone] All pub functions should have docs.
    pub fn whole_expr_id(&self) -> Option<ast::Id> {
        self.model.expression.borrow().whole_expr_id
    }
}
