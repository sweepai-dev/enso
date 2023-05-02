use crate::prelude::*;

use crate::display::world;
use crate::system::web;
use crate::system::web::dom::Shape;
use crate::system::web::traits_no_js_cast::*;

use enso_web::binding::mock::MockData;
use enso_web::binding::mock::MockDefault;
use enso_web::Reflect;
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



// type JsValue = web::JsValue;

pub trait Wrapper {
    fn init(&self);
}

impl<T: web::JsCast> Wrapper for T {
    fn init(&self) {}
}

pub trait Cast
where Self: Wrapper + AsRef<web::JsValue> + Into<web::JsValue> {
    // Required methods

    fn instanceof(val: &web::JsValue) -> bool;

    fn unchecked_from_js(val: web::JsValue) -> Self;

    fn unchecked_from_js_ref(val: &web::JsValue) -> &Self;

    // Default methods

    fn is_type_of(val: &web::JsValue) -> bool {
        Self::instanceof(val)
    }

    fn has_type<T>(&self) -> bool
    where Self: cast_helper::CastHelper<T> {
        <Self as cast_helper::CastHelper<T>>::has_type(self)
    }

    fn dyn_into<T>(self) -> Result<T, Self>
    where Self: cast_helper::CastHelper<T> {
        <Self as cast_helper::CastHelper<T>>::dyn_into(self)
    }

    fn dyn_ref<T>(&self) -> Option<&T>
    where Self: cast_helper::CastHelper<T> {
        <Self as cast_helper::CastHelper<T>>::dyn_ref(self)
    }

    fn unchecked_into<T>(self) -> T
    where Self: cast_helper::CastHelper<T> {
        <Self as cast_helper::CastHelper<T>>::unchecked_into(self)
    }

    fn unchecked_ref<T>(&self) -> &T
    where Self: cast_helper::CastHelper<T> {
        <Self as cast_helper::CastHelper<T>>::unchecked_ref(self)
    }

    fn is_instance_of<T>(&self) -> bool
    where Self: cast_helper::CastHelper<T> {
        <Self as cast_helper::CastHelper<T>>::is_instance_of(self)
    }
}

mod cast_helper {
    use super::*;

    pub trait CastHelper<T>
    where Self: Wrapper + AsRef<web::JsValue> + Into<web::JsValue> {
        fn has_type(&self) -> bool;
        fn dyn_into(self) -> Result<T, Self>;
        fn dyn_ref(&self) -> Option<&T>;
        fn unchecked_into(self) -> T;
        fn unchecked_ref(&self) -> &T;
        fn is_instance_of(&self) -> bool;
    }

    impl<S: Cast, T: Cast> CastHelper<T> for S {
        default fn has_type(&self) -> bool {
            T::is_type_of(self.as_ref())
        }

        default fn dyn_into(self) -> Result<T, Self> {
            if <S as CastHelper<T>>::has_type(&self) {
                Ok(<S as CastHelper<T>>::unchecked_into(self))
            } else {
                Err(self)
            }
        }

        default fn dyn_ref(&self) -> Option<&T> {
            if <S as CastHelper<T>>::has_type(&self) {
                Some(<S as CastHelper<T>>::unchecked_ref(self))
            } else {
                None
            }
        }

        default fn unchecked_into(self) -> T {
            let out = T::unchecked_from_js(self.into());
            out.init();
            out
        }

        default fn unchecked_ref(&self) -> &T {
            T::unchecked_from_js_ref(self.as_ref())
        }

        default fn is_instance_of(&self) -> bool {
            T::instanceof(self.as_ref())
        }
    }

    impl<S: web::JsCast, T: web::JsCast> CastHelper<T> for S {
        fn has_type(&self) -> bool {
            <S as web::JsCast>::has_type::<T>(self)
        }

        fn dyn_into(self) -> Result<T, Self> {
            <S as web::JsCast>::dyn_into::<T>(self)
        }

        fn dyn_ref(&self) -> Option<&T> {
            <S as web::JsCast>::dyn_ref::<T>(self)
        }

        fn unchecked_into(self) -> T {
            <S as web::JsCast>::unchecked_into::<T>(self)
        }

        fn unchecked_ref(&self) -> &T {
            <S as web::JsCast>::unchecked_ref::<T>(self)
        }

        fn is_instance_of(&self) -> bool {
            <S as web::JsCast>::is_instance_of::<T>(self)
        }
    }
}

impl<S: web::JsCast> Cast for S
where S: Wrapper + AsRef<web::JsValue> + Into<web::JsValue>
{
    fn instanceof(val: &web::JsValue) -> bool {
        <S as web::JsCast>::instanceof(val)
    }

    fn unchecked_from_js(val: web::JsValue) -> Self {
        <S as web::JsCast>::unchecked_from_js(val)
    }

    fn unchecked_from_js_ref(val: &web::JsValue) -> &Self {
        <S as web::JsCast>::unchecked_from_js_ref(val)
    }

    fn is_type_of(val: &web::JsValue) -> bool {
        <S as web::JsCast>::is_type_of(val)
    }
}



type WebJsValue = web::JsValue;

macro_rules! wrapper {
    ($name:ident [$base:ident]) => {
        paste! {
            #[derive(Debug, Clone, Deref,)]
            pub struct $name {
                [<$base:snake>]: $base,
            }

            impl From<$name> for web::JsValue {
                fn from(t: $name) -> Self {
                    t.[<$base:snake>].into()
                }
            }

            impl AsRef<web::JsValue> for $name {
                fn as_ref(&self) -> &web::JsValue {
                    self.[<$base:snake>].as_ref()
                }
            }

            impl Cast for $name {
                fn instanceof(val: &web::JsValue) -> bool {
                    <web::$name as web::JsCast>::instanceof(val)
                }

                fn unchecked_from_js(val: web::JsValue) -> Self {
                    Self { [<$base:snake>]: Cast::unchecked_from_js(val) }
                }

                fn unchecked_from_js_ref(val: &web::JsValue) -> &Self {
                    unsafe { &*(val as *const web::JsValue as *const Self) }
                }
            }
        }
    };
}


// ===============
// === JsValue ===
// ===============

pub const ValueIdKey: &str = "enso_value_id";
pub type ValueId = usize;

thread_local! {
    pub static NEXT_VALUE_ID: Cell<ValueId> = default();
}

fn next_value_id() -> ValueId {
    NEXT_VALUE_ID.with(|next_id| {
        let id = next_id.get();
        next_id.set(id.checked_add(1).unwrap_or_else(|| panic!("Object ID overflow: {}", id)));
        id
    })
}


wrapper! { JsValue [WebJsValue] }

impl Wrapper for JsValue {
    fn init(&self) {
        console_log!("INIT! {:?}", self.web_js_value);
        self.init_value_id();
    }
}

impl JsValue {
    pub fn value_id(&self) -> ValueId {
        self.with_raw_value_id(|num| f64::from(num) as usize, |id| id)
    }

    pub fn init_value_id(&self) {
        self.with_raw_value_id(
            |v| console_log!("value found: {:?}", v),
            |t| console_log!("value not found (new: {})", t),
        );
    }

    fn with_raw_value_id<T>(
        &self,
        found: impl FnOnce(web::Number) -> T,
        not_found: impl FnOnce(ValueId) -> T,
    ) -> T {
        // FIXME: slow ValueIdKey.into()
        let val = Reflect::get(&self, &ValueIdKey.into()).unwrap();
        console_log!("check: {:?}", val);

        let test = <web::Number as Cast>::is_type_of(&val.clone());
        let test2 = <web::Number as web::JsCast>::is_type_of(&val.clone());
        // let num: Result<web::Number, _> = val.clone().dyn_into();
        let num = val.clone().dyn_into::<web::Number>();
        let num2 = web::JsCast::dyn_into::<web::Number>(val);
        console_log!("test: {:?}", test);
        console_log!("test2: {:?}", test2);
        console_log!("num: {:?}", num);
        console_log!("num2: {:?}", num2);

        match num {
            Ok(num) => found(num),
            Err(_) => {
                let id = next_value_id();
                Reflect::set(&self, &ValueIdKey.into(), &web::Number::from(id as f64)).unwrap();
                console_log!("after set: {:?}", Reflect::get(&self, &ValueIdKey.into()).unwrap());
                not_found(id)
            }
        }
    }
}

impl From<web::Object> for JsValue {
    fn from(t: web::Object) -> Self {
        Self { web_js_value: t.into() }
    }
}

// ==============
// === Object ===
// ==============

wrapper! { Object [JsValue] }

impl Wrapper for Object {
    fn init(&self) {
        (**self).init()
    }
}

impl HasJsRepr for Object {
    type JsRepr = web::Object;
}


impl From<Object> for web::Object {
    fn from(t: Object) -> Self {
        t.js_value.web_js_value.unchecked_into()
    }
}

impl From<web::Object> for Object {
    fn from(t: web::Object) -> Self {
        Self { js_value: t.into() }
    }
}

impl UncheckedFrom<web::EventTarget> for Object {
    fn unchecked_from(t: web::EventTarget) -> Self {
        Self::from(web::JsCast::unchecked_into::<web::Object>(t))
    }
}

impl UncheckedFrom<web::Node> for Object {
    fn unchecked_from(t: web::Node) -> Self {
        Self::from(web::JsCast::unchecked_into::<web::Object>(t))
    }
}

impl UncheckedFrom<web::Element> for Object {
    fn unchecked_from(t: web::Element) -> Self {
        Self::from(web::JsCast::unchecked_into::<web::Object>(t))
    }
}

impl UncheckedFrom<web::HtmlElement> for Object {
    fn unchecked_from(t: web::HtmlElement) -> Self {
        Self::from(web::JsCast::unchecked_into::<web::Object>(t))
    }
}

impl UncheckedFrom<web::HtmlDivElement> for Object {
    fn unchecked_from(t: web::HtmlDivElement) -> Self {
        Self::from(web::JsCast::unchecked_into::<web::Object>(t))
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


wrapper! { EventTarget [Object] }

impl Wrapper for EventTarget {
    fn init(&self) {
        (**self).init();
    }
}

impl HasJsRepr for EventTarget {
    type JsRepr = web::EventTarget;
}


impl EventTarget {
    pub fn on_event<E: frp::Data>(&self) -> frp::Sampler<E>
    where E: From<web::JsValue> {
        panic!()
        // let network = frp::Network::new("event_listener");
        // frp::extend! { network
        //     src <- source::<E>();
        //     event <- src.sampler();
        //     trace src;
        // }
        //
        // let callback = web::Closure::<dyn Fn(web::JsValue)>::new(move |js_val: web::JsValue| {
        //     src.emit(E::from(js_val));
        // });
        // let callback_js = callback.as_ref().unchecked_ref();
        // self.js_repr().add_event_listener_with_callback("mousedown", callback_js);
        //
        // let listener = Listener { network, callback, event: Box::new(event.clone()) };
        // LISTENERS.with(|listeners| {
        //     let mut listeners = listeners.borrow_mut();
        //     let listeners = listeners.entry(self.event_target_id).or_default();
        //     listeners.insert(TypeId::of::<E>(), listener);
        // });
        // event
    }
}

impl UncheckedFrom<web::EventTarget> for EventTarget {
    fn unchecked_from(t: web::EventTarget) -> Self {
        t.unchecked_into()
        // let object = Object::unchecked_from(t);
        // let event_target_id = next_event_target_id();
        // let listeners = default();
        // let model = EventTargetModel { object, event_target_id, listeners };
        // Self { object: Rc::new(model) }
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


thread_local! {
    pub static NODE_COUNT: RefCell<HashMap<ValueId, usize>> = default();
}


wrapper! { Node [EventTarget] }

impl Wrapper for Node {
    fn init(&self) {
        (**self).init();
    }
}

impl HasJsRepr for Node {
    type JsRepr = web::Node;
}



impl Node {
    pub fn append_child(&self, child: &Node) {
        self.js_repr().append_child(child.js_repr()).unwrap();
    }

    pub fn remove_child(&self, child: &Node) -> bool {
        self.js_repr().remove_child(child.js_repr()).is_ok()
    }

    // pub fn remove_from_parent(&self) -> bool {
    //     self.parent
    //         .borrow()
    //         .as_ref()
    //         .and_then(|parent| parent.upgrade())
    //         .map(|parent| parent.remove_child(self))
    //         .unwrap_or(false)
    // }
}


impl AsRef<web::Node> for Node {
    fn as_ref(&self) -> &web::Node {
        self.event_target.unchecked_ref()
    }
}


impl UncheckedFrom<web::HtmlDivElement> for Node {
    fn unchecked_from(t: web::HtmlDivElement) -> Self {
        t.unchecked_into()
        // let event_target = EventTarget::unchecked_from(t);
        // let parent = default();
        // let children = default();
        // Self { model: Rc::new(NodeModel { event_target, parent, children }) }
    }
}



// ===============
// === Element ===
// ===============

wrapper! { Element [Node] }

impl Wrapper for Element {
    fn init(&self) {
        (**self).init();
    }
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

wrapper! { HtmlElement [Element] }

impl Wrapper for HtmlElement {
    fn init(&self) {
        (**self).init();
    }
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

wrapper! { HtmlDivElement [HtmlElement] }

impl Wrapper for HtmlDivElement {
    fn init(&self) {
        (**self).init();
    }
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
    fn init(&self) {}
}

impl From<web::HtmlDivElement> for HtmlDivElement {
    fn from(t: web::HtmlDivElement) -> Self {
        Self { html_element: HtmlElement::unchecked_from(t) }
    }
}
