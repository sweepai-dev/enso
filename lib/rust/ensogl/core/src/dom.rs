use crate::prelude::*;

use crate::display::world;
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


mod web {
    pub use crate::system::web::*;
    pub type UntrackedJsValue = JsValue;
}
use web::UntrackedJsValue;

macro_rules! wrapper {
    ($(#$meta:tt)* $name:ident [$base:ident $(, $bases:ident)*]) => {
        wrapper_no_web_conversions! { $(#$meta)* $name [$base $(,$bases)*] }
        wrapper_web_conversions! { $name [$name, $base $(,$bases)*] }
    }
}

macro_rules! wrapper_no_web_conversions {
    ($(#$meta:tt)* $name:ident [$base:ident $(, $bases:ident)*]) => {
        wrapper_struct! { $(#$meta)* $name [$base] }
        wrapper_conversions! { $name [$base $(,$bases)*] }
    }
}

macro_rules! wrapper_struct {
    ($(#$meta:tt)* $name:ident [$base:ident]) => {
        paste! {
            $(#$meta)*
            #[derive(Debug, Deref,)]
            pub struct $name {
                [<$base:snake>]: $base,
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

macro_rules! wrapper_conversions {
    ($name:ident [$($base:ident),*]) => {
        paste! {
            $(
                impl From<$name> for $base {
                    fn from(t: $name) -> Self {
                        t.unchecked_into()
                    }
                }

                impl AsRef<$base> for $name {
                    fn as_ref(&self) -> &$base {
                        &self.[<$base:snake>]
                    }
                }
            )*
        }
    };
}

macro_rules! wrapper_web_conversions {
    ($name:ident [$($base:ident),*]) => {
        impl From<web::$name> for $name {
            fn from(t: web::$name) -> Self {
                t.unchecked_into()
            }
        }

        paste! {
            $(
                impl From<$name> for web::$base {
                    fn from(t: $name) -> Self {
                        t.unchecked_into()
                    }
                }

                impl AsRef<web::$base> for $name {
                    fn as_ref(&self) -> &web::$base {
                        &self.untracked_js_value.unchecked_ref()
                    }
                }
            )*
        }
    };
}


// ===============
// === JsValue ===
// ===============

pub const VALUE_ID_KEY: &str = "ensoValueId";
pub type ValueId = usize;

thread_local! {
    pub static NEXT_VALUE_ID: Cell<ValueId> = default();
    pub static VALUE_REF_COUNT: RefCell<HashMap<ValueId, usize>> = default();
}

fn next_value_id() -> ValueId {
    NEXT_VALUE_ID.with(|next_id| {
        let id = next_id.get();
        next_id.set(id.checked_add(1).unwrap_or_else(|| panic!("Object ID overflow: {}", id)));
        id
    })
}

fn value_ref_count(id: ValueId) -> usize {
    VALUE_REF_COUNT.with(|ref_count| ref_count.borrow().get(&id).copied().unwrap_or(0))
}

fn inc_value_ref_count(id: ValueId) -> usize {
    VALUE_REF_COUNT.with(|ref_count| {
        let mut ref_count = ref_count.borrow_mut();
        let count = ref_count.entry(id).or_default();
        *count += 1;
        *count
    })
}

fn dec_value_ref_count(id: ValueId) -> usize {
    VALUE_REF_COUNT.with(|ref_count| {
        let mut ref_count = ref_count.borrow_mut();
        let count = ref_count.entry(id).or_default();
        *count = count.saturating_sub(1);
        *count
    })
}



wrapper_no_web_conversions! {
    JsValue [UntrackedJsValue]
}

impl Clone for JsValue {
    fn clone(&self) -> Self {
        inc_value_ref_count(self.value_id());
        Self { untracked_js_value: self.untracked_js_value.clone() }
    }
}

impl Drop for JsValue {
    fn drop(&mut self) {
        dec_value_ref_count(self.value_id());
    }
}

impl Wrapper for JsValue {
    fn init(&self) {
        inc_value_ref_count(self.value_id());
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
        // FIXME: slow VALUE_ID_KEY.into()
        let val = Reflect::get(&self, &VALUE_ID_KEY.into()).unwrap();
        let num = val.clone().dyn_into::<web::Number>();
        match num {
            Ok(num) => found(num),
            Err(_) => {
                let id = next_value_id();
                Reflect::set(&self, &VALUE_ID_KEY.into(), &web::Number::from(id as f64)).unwrap();
                console_log!("after set: {:?}", Reflect::get(&self, &VALUE_ID_KEY.into()).unwrap());
                not_found(id)
            }
        }
    }
}

// ==============
// === Object ===
// ==============

wrapper! {
    #[derive(Clone)]
    Object [JsValue]
}

impl Wrapper for Object {
    fn init(&self) {
        (**self).init()
    }
}

impl HasJsRepr for Object {
    type JsRepr = web::Object;
}



// ===================
// === EventTarget ===
// ===================

thread_local! {
    pub static LISTENERS: RefCell<HashMap<ValueId, HashMap<TypeId, Listener>>> = default();
}


#[derive(Debug)]
pub struct Listener {
    network:  frp::Network,
    callback: web::Closure<dyn Fn(web::JsValue)>,
    event:    Box<dyn Any>,
}


wrapper! {
    #[derive(Clone)]
    EventTarget [Object, JsValue]
}

impl Drop for EventTarget {
    fn drop(&mut self) {
        LISTENERS.with(|listeners| {
            listeners.borrow_mut().remove(&self.value_id());
            // We do not need to unregister listeners as the object is dropped.
        })
    }
}

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
        self.js_repr().add_event_listener_with_callback("mousedown", callback_js).unwrap();

        let listener = Listener { network, callback, event: Box::new(event.clone()) };
        LISTENERS.with(|listeners| {
            let mut listeners = listeners.borrow_mut();
            let listeners = listeners.entry(self.value_id()).or_default();
            listeners.insert(TypeId::of::<E>(), listener);
        });
        event
    }
}



// =============
// === Node ====
// =============

wrapper! {
    #[derive(Clone)]
    Node [EventTarget, Object, JsValue]
}

impl Node {}

impl Drop for Node {
    fn drop(&mut self) {
        if value_ref_count(self.value_id()) == 1 {
            self.remove_from_parent();
        }
    }
}

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

    pub fn parent(&self) -> Option<Node> {
        self.js_repr().parent_node().map(|parent| parent.unchecked_into())
    }

    pub fn remove_from_parent(&self) -> bool {
        self.parent().map(|parent| parent.remove_child(self)).unwrap_or(false)
    }
}


// ===============
// === Element ===
// ===============

wrapper! {
    #[derive(Clone)]
    Element [Node, EventTarget, Object, JsValue]
}

impl Wrapper for Element {
    fn init(&self) {
        (**self).init();
    }
}

impl HasJsRepr for Element {
    type JsRepr = web::Element;
}



// ===================
// === HtmlElement ===
// ===================

wrapper! {
    #[derive(Clone)]
    HtmlElement [Element, Node, EventTarget, Object, JsValue]
}

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



// ======================
// === HtmlDivElement ===
// ======================


pub type Div = HtmlDivElement;

wrapper! {
    #[derive(Clone)]
    HtmlDivElement [HtmlElement, Element, Node, EventTarget, Object, JsValue]
}

impl Wrapper for HtmlDivElement {
    fn init(&self) {
        (**self).init();
    }
}

impl HasJsRepr for HtmlDivElement {
    type JsRepr = web::HtmlDivElement;
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
