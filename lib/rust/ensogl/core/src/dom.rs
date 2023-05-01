use crate::prelude::*;

use crate::display::world;
use crate::system::web;
use crate::system::web::dom::Shape;
use crate::system::web::traits::*;

use std::any::TypeId;

pub mod event;


pub trait UncheckedFrom<T> {
    fn unchecked_from(t: T) -> Self;
}

impl<T, S: From<T>> UncheckedFrom<T> for S {
    fn unchecked_from(t: T) -> Self {
        Self::from(t)
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
pub struct Object {
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
pub struct EventTarget {
    object: Rc<EventTargetModel>,
}

#[derive(Debug, Deref)]
pub struct EventTargetModel {
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
    where E: From<web::JsValue> {
        let network = frp::Network::new("event_listener");
        frp::extend! { network
            src <- source::<E>();
            event <- src.sampler();
            trace src;
        }

        let callback = web::Closure::<dyn Fn(web::JsValue)>::new(move |js_val: web::JsValue| {
            src.emit(E::from(js_val));
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
pub struct Node {
    model: Rc<NodeModel>,
}

#[derive(Debug, Clone)]
pub struct WeakNode {
    model: Weak<NodeModel>,
}

#[derive(Debug, Deref)]
pub struct NodeModel {
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
pub struct Element {
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
pub struct HtmlElement {
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


pub type Div = HtmlDivElement;

#[repr(transparent)]
#[derive(Debug, Clone, Deref)]
pub struct HtmlDivElement {
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
