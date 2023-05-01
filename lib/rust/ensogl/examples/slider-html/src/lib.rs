//! An example scene showing the slider component usage.

// === Features ===
#![feature(associated_type_defaults)]
#![feature(drain_filter)]
#![feature(fn_traits)]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]
#![feature(unboxed_closures)]
// === Standard Linter Configuration ===
#![deny(non_ascii_idents)]
#![warn(unsafe_code)]
#![allow(clippy::bool_to_int_with_if)]
#![allow(clippy::let_and_return)]
// === Non-Standard Linter Configuration ===
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]

use ensogl_core::prelude::*;
use std::any::TypeId;

use enso_frp as frp;
use ensogl_core::application::shortcut;
use ensogl_core::application::Application;
use ensogl_core::application::View;
use ensogl_core::control::io::mouse;
use ensogl_core::data::color;
use ensogl_core::display;
use ensogl_core::display::navigation::navigator::Navigator;
use ensogl_core::display::symbol::DomSymbol;
use ensogl_core::display::world;
use ensogl_core::system::web;
use ensogl_core::system::web::binding::mock;
use ensogl_core::system::web::dom::Shape;
use ensogl_core::system::web::traits::*;
use ensogl_slider as slider;
use ensogl_text_msdf::run_once_initialized;


pub trait UncheckedFrom<T> {
    fn unchecked_from(t: T) -> Self;
}

impl<T, S: From<T>> UncheckedFrom<T> for S {
    fn unchecked_from(t: T) -> Self {
        Self::from(t)
    }
}

// pub trait UncheckedInto<T> {
//     fn unchecked_into(self) -> T;
// }

// impl<T, S> UncheckedInto<T> for S
// where T: UncheckedFrom<S>
// {
//     fn unchecked_into(self) -> T {
//         T::unchecked_from(self)
//     }
// }

// ========================
// === Model definition ===
// ========================

/// The slider collection model holds a set of sliders that can be instantiated and dropped.
#[derive(Debug, Clone, CloneRef)]
pub struct Model {
    /// Vector that holds example sliders until they are dropped.
    sliders:   Rc<RefCell<Vec<slider::Slider>>>,
    app:       Application,
    root:      display::object::Instance,
    navigator: Navigator,
}

impl Model {
    fn new(app: &Application) -> Self {
        let app = app.clone_ref();
        let world = app.display.clone();
        let scene = &world.default_scene;
        let camera = scene.camera().clone_ref();
        let navigator = Navigator::new(scene, &camera);
        let sliders = Rc::new(RefCell::new(Vec::new()));
        let root = display::object::Instance::new();
        let model = Self { app, sliders, root, navigator };
        model.init_sliders();
        model
    }

    /// Add example sliders to scene.
    fn init_sliders(&self) {
        let slider1 = self.app.new_view::<slider::Slider>();
        slider1.set_size((200.0, 24.0));
        slider1.set_y(-120.0);
        slider1.frp.set_value_indicator_color(color::Lcha(0.4, 0.7, 0.7, 1.0));
        slider1.frp.set_label("Soft limits + tooltip");
        slider1.frp.set_lower_limit_type(slider::SliderLimit::Soft);
        slider1.frp.set_upper_limit_type(slider::SliderLimit::Soft);
        slider1.frp.set_tooltip("Slider information tooltip.");
        self.root.add_child(&slider1);
        self.sliders.borrow_mut().push(slider1);

        // # IMPORTANT
        // This code is commented because the slider implementation is not finished yet. Please
        // refer to the doc comments in the slider's module to learn more.

        //
        // let slider2 = self.app.new_view::<slider::Slider>();
        // slider2.set_size((400.0, 50.0));
        // slider2.set_y(-60.0);
        // slider2.frp.set_value_indicator_color(color::Lcha(0.4, 0.7, 0.7, 1.0));
        // slider2.frp.set_slider_disabled(true);
        // slider2.frp.set_label("Disabled");
        // self.root.add_child(&slider2);
        // self.sliders.borrow_mut().push(slider2);
        //
        // let slider3 = self.app.new_view::<slider::Slider>();
        // slider3.set_size((400.0, 50.0));
        // slider3.set_y(0.0);
        // slider3.frp.set_value_indicator_color(color::Lcha(0.4, 0.7, 0.7, 1.0));
        // slider3.frp.set_default_value(100.0);
        // slider3.frp.set_value(100.0);
        // slider3.frp.set_max_value(500.0);
        // slider3.frp.set_label("Adaptive lower limit");
        // slider3.frp.set_lower_limit_type(slider::SliderLimit::Adaptive);
        // self.root.add_child(&slider3);
        // self.sliders.borrow_mut().push(slider3);
        //
        // let slider4 = self.app.new_view::<slider::Slider>();
        // slider4.set_size((400.0, 50.0));
        // slider4.set_y(60.0);
        // slider4.frp.set_value_indicator_color(color::Lcha(0.4, 0.7, 0.7, 1.0));
        // slider4.frp.set_label("Adaptive upper limit");
        // slider4.frp.set_label_position(slider::LabelPosition::Inside);
        // slider4.frp.set_upper_limit_type(slider::SliderLimit::Adaptive);
        // self.root.add_child(&slider4);
        // self.sliders.borrow_mut().push(slider4);
        //
        // let slider5 = self.app.new_view::<slider::Slider>();
        // slider5.set_size((75.0, 230.0));
        // slider5.set_y(-35.0);
        // slider5.set_x(275.0);
        // slider5.frp.set_value_indicator_color(color::Lcha(0.4, 0.7, 0.7, 1.0));
        // slider5.frp.set_label("Hard limits");
        // slider5.frp.orientation(Axis2::Y);
        // slider5.frp.set_max_disp_decimal_places(4);
        // self.root.add_child(&slider5);
        // self.sliders.borrow_mut().push(slider5);
        //
        // let slider6 = self.app.new_view::<slider::Slider>();
        // slider6.set_size((75.0, 230.0));
        // slider6.set_y(-35.0);
        // slider6.set_x(375.0);
        // slider6.frp.set_value_indicator_color(color::Lcha(0.4, 0.7, 0.7, 1.0));
        // slider6.frp.set_label("Soft\nlimits");
        // slider6.frp.set_label_position(slider::LabelPosition::Inside);
        // slider6.frp.set_lower_limit_type(slider::SliderLimit::Soft);
        // slider6.frp.set_upper_limit_type(slider::SliderLimit::Soft);
        // slider6.frp.orientation(Axis2::Y);
        // slider6.frp.set_max_disp_decimal_places(4);
        // self.root.add_child(&slider6);
        // self.sliders.borrow_mut().push(slider6);
        //
        // let slider7 = self.app.new_view::<slider::Slider>();
        // slider7.set_size((400.0, 10.0));
        // slider7.set_y(-160.0);
        // slider7.frp.set_value_indicator_color(color::Lcha(0.4, 0.7, 0.7, 1.0));
        // slider7.frp.show_value(false);
        // slider7.frp.set_precision_adjustment_disabled(true);
        // slider7.frp.kind(slider::Kind::Scrollbar(0.1));
        // slider7.frp.set_thumb_size(0.1);
        // self.root.add_child(&slider7);
        // self.sliders.borrow_mut().push(slider7);
        //
        // let slider8 = self.app.new_view::<slider::Slider>();
        // slider8.set_size((400.0, 10.0));
        // slider8.set_y(-180.0);
        // slider8.frp.set_value_indicator_color(color::Lcha(0.4, 0.7, 0.7, 1.0));
        // slider8.frp.show_value(false);
        // slider8.frp.set_precision_adjustment_disabled(true);
        // slider8.frp.kind(slider::Kind::Scrollbar(0.25));
        // slider8.frp.set_thumb_size(0.25);
        // self.root.add_child(&slider8);
        // self.sliders.borrow_mut().push(slider8);
        //
        // let slider9 = self.app.new_view::<slider::Slider>();
        // slider9.set_size((400.0, 10.0));
        // slider9.set_y(-200.0);
        // slider9.frp.set_value_indicator_color(color::Lcha(0.4, 0.7, 0.7, 1.0));
        // slider9.frp.show_value(false);
        // slider9.frp.set_precision_adjustment_disabled(true);
        // slider9.frp.kind(slider::Kind::Scrollbar(0.5));
        // slider9.frp.set_thumb_size(0.5);
        // self.root.add_child(&slider9);
        // self.sliders.borrow_mut().push(slider9);
        //
        // let slider10 = self.app.new_view::<slider::Slider>();
        // slider10.set_size((10.0, 230));
        // slider10.set_y(-35.0);
        // slider10.set_x(430.0);
        // slider10.frp.set_value_indicator_color(color::Lcha(0.4, 0.7, 0.7, 1.0));
        // slider10.frp.show_value(false);
        // slider10.frp.set_precision_adjustment_disabled(true);
        // slider10.frp.kind(slider::Kind::Scrollbar(0.1));
        // slider10.frp.orientation(Axis2::Y);
        // self.root.add_child(&slider10);
        // self.sliders.borrow_mut().push(slider10);
    }

    /// Drop all sliders from scene.
    fn drop_sliders(&self) {
        for slider in self.sliders.borrow_mut().drain(0..) {
            self.root.remove_child(&slider);
        }
    }
}

impl display::Object for Model {
    fn display_object(&self) -> &display::object::Instance {
        &self.root
    }
}



// ===================
// === FRP network ===
// ===================

mod slider_collection {
    use super::*;
    ensogl_core::define_endpoints! {
        Input {
            /// Add example sliders to scene.
            init_sliders(),
            /// Drop all sliders from scene.
            drop_sliders(),
        }
        Output {
        }
    }

    impl FrpNetworkProvider for SliderCollection {
        fn network(&self) -> &frp::Network {
            self.frp.network()
        }
    }


    // ==========================
    // === Slider collection ===
    // ==========================

    /// A component that stores an array of slider components. It receives shortcuts to either
    /// instantiate a new set of sliders or to drop the existing ones.
    #[derive(Clone, Debug, Deref)]
    pub struct SliderCollection {
        #[deref]
        frp:   Frp,
        app:   Application,
        model: Model,
    }

    impl SliderCollection {
        fn new(app: &Application) -> Self {
            let frp = Frp::new();
            let app = app.clone_ref();
            let model = Model::new(&app);
            Self { frp, app, model }.init()
        }

        fn init(self) -> Self {
            let network = self.frp.network();
            let input = &self.frp.input;
            let model = &self.model;

            frp::extend! { network
                eval_ input.init_sliders( model.init_sliders() );
                eval_ input.drop_sliders( model.drop_sliders() );
            }
            self
        }
    }

    impl display::Object for SliderCollection {
        fn display_object(&self) -> &display::object::Instance {
            self.model.display_object()
        }
    }

    impl View for SliderCollection {
        fn label() -> &'static str {
            "Slider Collection"
        }

        fn new(app: &Application) -> Self {
            Self::new(app)
        }

        fn app(&self) -> &Application {
            &self.app
        }

        fn default_shortcuts() -> Vec<shortcut::Shortcut> {
            use shortcut::ActionType::Press;
            vec![
                Self::self_shortcut(Press, "ctrl a", "init_sliders"),
                Self::self_shortcut(Press, "ctrl d", "drop_sliders"),
            ]
        }
    }
}

use slider_collection::SliderCollection;


// ===================
// === Entry Point ===
// ===================

/// Entry point for the example scene.
#[entry_point]
#[allow(dead_code)]
pub fn main() {
    run_once_initialized(|| {
        let app = Application::new("root");
        init(&app);
        mem::forget(app);
    });
}

ensogl_core::define_endpoints! {
    Input {}
    Output {}
}


trait JsBaseObject {
    fn default() -> Self;
}

impl JsBaseObject for web::HtmlDivElement {
    fn default() -> Self {
        web::document.create_div_or_panic()
    }
}

// struct Object {}
// struct EventTarget {}
// struct Node {}
// struct Element {}
// struct HtmlElement {}
// struct HtmlDivElement {}



pub trait HasJsRepr {
    type JsRepr;
    fn js_repr(&self) -> &Self::JsRepr
    where Self: AsRef<Self::JsRepr> {
        self.as_ref()
    }
}

pub type JsRepr<T> = <T as HasJsRepr>::JsRepr;



type JsValue = web::JsValue;


// ==============
// === Object ===
// ==============

#[derive(AsRef, Debug, Clone, Deref, Into)]
struct Object {
    repr: JsValue,
}

impl HasJsRepr for Object {
    type JsRepr = web::Object;
}

impl From<Object> for web::Object {
    fn from(t: Object) -> Self {
        t.repr.unchecked_into()
    }
}

impl From<web::Object> for Object {
    fn from(t: web::Object) -> Self {
        Self { repr: t.into() }
    }
}

impl UncheckedFrom<web::EventTarget> for Object {
    fn unchecked_from(t: web::EventTarget) -> Self {
        Self::from(JsCast::unchecked_into::<web::Object>(t))
    }
}

impl UncheckedFrom<web::Node> for Object {
    fn unchecked_from(t: web::Node) -> Self {
        Self::from(JsCast::unchecked_into::<web::Object>(t))
    }
}

impl UncheckedFrom<web::Element> for Object {
    fn unchecked_from(t: web::Element) -> Self {
        Self::from(JsCast::unchecked_into::<web::Object>(t))
    }
}

impl UncheckedFrom<web::HtmlElement> for Object {
    fn unchecked_from(t: web::HtmlElement) -> Self {
        Self::from(JsCast::unchecked_into::<web::Object>(t))
    }
}

impl UncheckedFrom<web::HtmlDivElement> for Object {
    fn unchecked_from(t: web::HtmlDivElement) -> Self {
        Self::from(JsCast::unchecked_into::<web::Object>(t))
    }
}


// ===================
// === EventTarget ===
// ===================

thread_local! {
    pub static LISTENERS: RefCell<HashMap<EventTargetId, HashMap<TypeId, Listener>>> = default();
}



fn add_listener_for<E>(target: &EventTarget) -> frp::Sampler<E>
where E: frp::Data + From<(web::JsValue, Shape)> {
    // let network = frp::Network::new("event_listener");
    // frp::extend! { network
    //     src <- source::<E>();
    //     event <- src.sampler();
    //     trace src;
    // }
    //
    // let scene = world::scene();
    // let html_root = &scene.dom.html_root;
    // let shape = html_root.shape.clone_ref();
    // let callback = web::Closure::<dyn Fn(web::JsValue)>::new(move |js_val: web::JsValue| {
    //     let shape = shape.value();
    //     let event = E::from((js_val, shape));
    //     src.emit(event);
    // });
    // let callback_js = callback.as_ref().unchecked_ref();
    // target.js_repr().add_event_listener_with_callback("mousedown", callback_js);
    //
    // let listener = Listener { network, callback, event: Box::new(event.clone()) };
    // LISTENERS.with(|listeners| {
    //     let mut listeners = listeners.borrow_mut();
    //     let listeners = listeners.entry(target.id()).or_default();
    //     listeners.insert(TypeId::of::<E>(), listener);
    // });
    // event
    panic!()
}


thread_local! {
    pub static LAST_EVENT_TARGET_ID: Cell<EventTargetId> = default();
}

fn next_event_target_id() -> EventTargetId {
    LAST_EVENT_TARGET_ID.with(|id| {
        let id = id.get();
        id.checked_add(1).unwrap_or_else(|| panic!("Object ID overflow: {}", id))
    })
}

type EventTargetId = usize;



#[derive(Debug)]
pub struct Listener {
    network:  frp::Network,
    callback: web::Closure<dyn Fn(web::JsValue)>,
    event:    Box<dyn Any>,
}

#[derive(Debug, Clone, Deref)]
struct EventTarget {
    object: Rc<EventTargetModel>,
}

#[derive(Debug, Deref)]
struct EventTargetModel {
    #[deref]
    object:          Object,
    event_target_id: EventTargetId,
    listeners:       RefCell<Vec<Listener>>,
}

impl HasJsRepr for EventTarget {
    type JsRepr = web::EventTarget;
}


impl EventTarget {
    pub fn on_event<E: frp::Data>(&self) -> frp::Sampler<E>
    where E: From<(web::JsValue, Shape)> {
        let network = frp::Network::new("event_listener");
        frp::extend! { network
            src <- source::<E>();
            event <- src.sampler();
            trace src;
        }

        let scene = world::scene();
        let html_root = &scene.dom.html_root;
        let shape = html_root.shape.clone_ref();
        let callback = web::Closure::<dyn Fn(web::JsValue)>::new(move |js_val: web::JsValue| {
            let shape = shape.value();
            let event = E::from((js_val, shape));
            src.emit(event);
        });
        let callback_js = callback.as_ref().unchecked_ref();
        self.js_repr().add_event_listener_with_callback("mousedown", callback_js);

        let listener = Listener { network, callback, event: Box::new(event.clone()) };
        LISTENERS.with(|listeners| {
            let mut listeners = listeners.borrow_mut();
            let listeners = listeners.entry(self.event_target_id).or_default();
            listeners.insert(TypeId::of::<E>(), listener);
        });
        event
    }
}

impl UncheckedFrom<web::EventTarget> for EventTarget {
    fn unchecked_from(t: web::EventTarget) -> Self {
        let object = Object::unchecked_from(t);
        let event_target_id = next_event_target_id();
        let listeners = default();
        let model = EventTargetModel { object, event_target_id, listeners };
        Self { object: Rc::new(model) }
    }
}

impl UncheckedFrom<web::HtmlDivElement> for EventTarget {
    fn unchecked_from(t: web::HtmlDivElement) -> Self {
        Self::unchecked_from(web::EventTarget::from(t))
    }
}

impl AsRef<web::EventTarget> for EventTarget {
    fn as_ref(&self) -> &web::EventTarget {
        self.unchecked_ref()
    }
}

// =============
// === Node ====
// =============


#[derive(Debug, Clone, Deref)]
struct Node {
    model: Rc<NodeModel>,
}

#[derive(Debug, Clone)]
struct WeakNode {
    model: Weak<NodeModel>,
}

#[derive(Debug, Deref)]
struct NodeModel {
    #[deref]
    event_target: EventTarget,
    parent:       RefCell<Option<WeakNode>>,
    children:     RefCell<HashSet<WeakNode>>,
}

impl HasJsRepr for Node {
    type JsRepr = web::Node;
}

impl HasJsRepr for NodeModel {
    type JsRepr = web::Node;
}

impl Node {
    pub fn downgrade(&self) -> WeakNode {
        WeakNode { model: Rc::downgrade(&self.model) }
    }

    pub fn append_child(&self, child: &Node) {
        self.js_repr().append_child(child.js_repr()).unwrap();
        self.children.borrow_mut().insert(child.downgrade());
        child.parent.borrow_mut().replace(self.downgrade());
    }

    pub fn remove_child(&self, child: &Node) -> bool {
        if self.js_repr().remove_child(child.js_repr()).is_ok() {
            self.children.borrow_mut().remove(&child.downgrade());
            child.parent.borrow_mut().take();
            true
        } else {
            false
        }
    }

    pub fn remove_from_parent(&self) -> bool {
        self.parent
            .borrow()
            .as_ref()
            .and_then(|parent| parent.upgrade())
            .map(|parent| parent.remove_child(self))
            .unwrap_or(false)
    }
}

impl NodeModel {
    fn remove_child_in_js_only(&self, child: &web::Node) -> bool {
        self.js_repr().remove_child(child).is_ok()
    }
}

impl WeakNode {
    pub fn upgrade(&self) -> Option<Node> {
        self.model.upgrade().map(|model| Node { model })
    }
}

impl AsRef<web::Node> for Node {
    fn as_ref(&self) -> &web::Node {
        self.event_target.unchecked_ref()
    }
}

impl AsRef<web::Node> for NodeModel {
    fn as_ref(&self) -> &web::Node {
        self.event_target.unchecked_ref()
    }
}

impl UncheckedFrom<web::HtmlDivElement> for Node {
    fn unchecked_from(t: web::HtmlDivElement) -> Self {
        let event_target = EventTarget::unchecked_from(t);
        let parent = default();
        let children = default();
        Self { model: Rc::new(NodeModel { event_target, parent, children }) }
    }
}

impl Eq for Node {}
impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.model, &other.model)
    }
}

impl Eq for WeakNode {}
impl PartialEq for WeakNode {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.model, &other.model)
    }
}

impl Hash for WeakNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Weak::as_ptr(&self.model).hash(state);
    }
}

impl Drop for NodeModel {
    fn drop(&mut self) {
        if let Some(parent) = self.parent.borrow().as_ref().and_then(|t| t.upgrade()) {
            parent.remove_child_in_js_only(self.js_repr());
        }
    }
}

// ===============
// === Element ===
// ===============

#[repr(transparent)]
#[derive(Debug, Clone, Deref)]
struct Element {
    node: Node,
}

impl HasJsRepr for Element {
    type JsRepr = web::Element;
}

impl AsRef<web::Element> for Element {
    fn as_ref(&self) -> &web::Element {
        self.unchecked_ref()
    }
}

// impl From<web::JsValue> for Element {
//     fn from(js_value: web::JsValue) -> Self {
//         Self { node: js_value.into() }
//     }
// }

impl UncheckedFrom<web::HtmlDivElement> for Element {
    fn unchecked_from(t: web::HtmlDivElement) -> Self {
        Self { node: Node::unchecked_from(t) }
    }
}


// ===================
// === HtmlElement ===
// ===================

#[repr(transparent)]
#[derive(Debug, Clone, Deref)]
struct HtmlElement {
    element: Element,
}

impl HasJsRepr for HtmlElement {
    type JsRepr = web::HtmlElement;
}

impl HtmlElement {
    pub fn set_width(&self, width: f64) -> &Self {
        self.js_repr().set_style_or_warn("width", &format!("{}px", width));
        self
    }

    pub fn set_height(&self, width: f64) -> &Self {
        self.js_repr().set_style_or_warn("height", &format!("{}px", width));
        self
    }

    pub fn set_background(&self, background: &str) -> &Self {
        self.js_repr().set_style_or_warn("background", background);
        self
    }

    pub fn set_display(&self, display: &str) -> &Self {
        self.js_repr().set_style_or_warn("display", display);
        self
    }

    pub fn set_border_radius(&self, radius: f64) -> &Self {
        self.js_repr().set_style_or_warn("border-radius", &format!("{}px", radius));
        self
    }
}

impl AsRef<web::HtmlElement> for HtmlElement {
    fn as_ref(&self) -> &web::HtmlElement {
        self.unchecked_ref()
    }
}

// impl From<web::JsValue> for HtmlElement {
//     fn from(js_value: web::JsValue) -> Self {
//         Self { element: js_value.into() }
//     }
// }

impl UncheckedFrom<web::HtmlDivElement> for HtmlElement {
    fn unchecked_from(t: web::HtmlDivElement) -> Self {
        Self { element: Element::unchecked_from(t) }
    }
}

// ======================
// === HtmlDivElement ===
// ======================



type Div = HtmlDivElement;

#[repr(transparent)]
#[derive(Debug, Clone, Deref)]
struct HtmlDivElement {
    html_element: HtmlElement,
}

impl HasJsRepr for HtmlDivElement {
    type JsRepr = web::HtmlDivElement;
}

impl AsRef<web::HtmlDivElement> for HtmlDivElement {
    fn as_ref(&self) -> &web::HtmlDivElement {
        self.unchecked_ref()
    }
}

impl Default for HtmlDivElement {
    fn default() -> Self {
        Self::new()
    }
}

impl HtmlDivElement {
    pub fn new() -> Self {
        Self::from(web::document.create_div_or_panic())
    }
}

impl HtmlDivElement {
    fn init(self) -> Self {
        // self.set_display("flex");
        self
    }
}

impl From<web::HtmlDivElement> for HtmlDivElement {
    fn from(t: web::HtmlDivElement) -> Self {
        Self { html_element: HtmlElement::unchecked_from(t) }
    }
}


// ========================
// === Init Application ===
// ========================

mod glob {
    use super::*;
    ensogl_core::define_endpoints! {
        Input {}
        Output {}
    }
}

/// Initialize a `SliderCollection` and do not drop it.
fn init(app: &Application) {
    let slider_collection = app.new_view::<SliderCollection>().leak();
    app.display.add_child(&slider_collection);

    let world = app.display.clone();
    let scene = &world.default_scene;
    let dom_front_layer = &scene.dom.layers.front;

    let root = Div::from(
        web::document
            .get_element_by_id("html-root")
            .unwrap()
            .unchecked_into::<web::HtmlDivElement>(),
    );
    let div1 = Div::new();
    div1.set_width(100.0).set_height(100.0).set_background("red").set_border_radius(10.0);
    let div2 = Div::new();
    div2.set_width(100.0).set_height(100.0).set_background("green").set_border_radius(10.0);
    root.append_child(&div1);
    root.append_child(&div2);

    let on_down = div1.on_event::<mouse::Down>();

    let width = Rc::new(Cell::new(100.0));

    let frp = glob::Frp::new();
    let network = frp.network();
    frp::extend! { network
        trace on_down;
        eval_ on_down ({
            width.set(width.get() + 10.0);
            div1.set_width(width.get());
        });
    }

    mem::forget(div2);
    mem::forget(frp);
}
