use crate::prelude::*;

use crate::display::world;
use crate::system::web::dom::Shape;
use crate::system::web::traits_no_js_cast::*;

use enso_frp::web;
use enso_web::binding::mock::MockData;
use enso_web::binding::mock::MockDefault;
use enso_web::Reflect;
use std::any::TypeId;

pub mod event;

pub mod untracked {
    pub use crate::system::web::*;
    pub type UntrackedJsValue = JsValue;
}
use untracked::UntrackedJsValue;

pub use crate::system::web::document;

pub mod traits {
    pub use super::Cast as TRAIT_Cast;
}

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



pub trait HasUntrackedRepr: AsRef<Self::UntrackedRepr> {
    type UntrackedRepr;
    fn untracked_repr(&self) -> &Self::UntrackedRepr {
        self.as_ref()
    }
}

pub type UntrackedRepr<T> = <T as HasUntrackedRepr>::UntrackedRepr;



// type JsValue = untracked::JsValue;

pub trait TrackingInitializer {
    fn init_tracking(&self);
}

impl<T: untracked::JsCast> TrackingInitializer for T {
    fn init_tracking(&self) {}
}

pub trait Cast
where Self: TrackingInitializer + AsRef<untracked::JsValue> + Into<untracked::JsValue> {
    // Required methods

    fn instanceof(val: &untracked::JsValue) -> bool;

    fn unchecked_from_js(val: untracked::JsValue) -> Self;

    fn unchecked_from_js_ref(val: &untracked::JsValue) -> &Self;

    // Default methods

    fn is_type_of(val: &untracked::JsValue) -> bool {
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
    where Self: TrackingInitializer + AsRef<untracked::JsValue> + Into<untracked::JsValue> {
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
            out.init_tracking();
            out
        }

        default fn unchecked_ref(&self) -> &T {
            T::unchecked_from_js_ref(self.as_ref())
        }

        default fn is_instance_of(&self) -> bool {
            T::instanceof(self.as_ref())
        }
    }

    impl<S: untracked::JsCast, T: untracked::JsCast> CastHelper<T> for S {
        fn has_type(&self) -> bool {
            <S as untracked::JsCast>::has_type::<T>(self)
        }

        fn dyn_into(self) -> Result<T, Self> {
            <S as untracked::JsCast>::dyn_into::<T>(self)
        }

        fn dyn_ref(&self) -> Option<&T> {
            <S as untracked::JsCast>::dyn_ref::<T>(self)
        }

        fn unchecked_into(self) -> T {
            <S as untracked::JsCast>::unchecked_into::<T>(self)
        }

        fn unchecked_ref(&self) -> &T {
            <S as untracked::JsCast>::unchecked_ref::<T>(self)
        }

        fn is_instance_of(&self) -> bool {
            <S as untracked::JsCast>::is_instance_of::<T>(self)
        }
    }
}

impl<S: untracked::JsCast> Cast for S
where S: TrackingInitializer + AsRef<untracked::JsValue> + Into<untracked::JsValue>
{
    fn instanceof(val: &untracked::JsValue) -> bool {
        <S as untracked::JsCast>::instanceof(val)
    }

    fn unchecked_from_js(val: untracked::JsValue) -> Self {
        <S as untracked::JsCast>::unchecked_from_js(val)
    }

    fn unchecked_from_js_ref(val: &untracked::JsValue) -> &Self {
        <S as untracked::JsCast>::unchecked_from_js_ref(val)
    }

    fn is_type_of(val: &untracked::JsValue) -> bool {
        <S as untracked::JsCast>::is_type_of(val)
    }
}



macro_rules! wrapper {
    ($(#$meta:tt)* $name:ident [$base:ident $(, $bases:ident)*]) => {
        starting_wrapper! { $(#$meta)* $name [$base $(,$bases)*] }
        wrapper_web_conversions! { $name [$name, $base $(,$bases)*] }

        impl TrackingInitializer for $name {
            fn init_tracking(&self) {
                (**self).init_tracking();
            }
        }
    }
}

macro_rules! starting_wrapper {
    ($(#$meta:tt)* $name:ident [$base:ident $(, $bases:ident)*]) => {
        wrapper_struct! { $(#$meta)* $name [$base] }
        wrapper_down_conversions! { $name [$base $(,$bases)*] }
        wrapper_up_conversions! { $name [$base $(,$bases)*] }
    }
}

macro_rules! wrapper_struct {
    ($(#$meta:tt)* $name:ident [$base:ident]) => {
        paste! {
            $(#$meta)*
            #[derive(Debug, Deref)]
            #[repr(transparent)]
            pub struct $name {
                #[allow(missing_docs)]
                pub [<$base:snake>]: $base,
            }

            impl CloneRef for $name {
                fn clone_ref(&self) -> Self {
                    self.clone()
                }
            }

            impl HasUntrackedRepr for $name {
                type UntrackedRepr = untracked::$name;
            }

            impl HasUntrackedRepr for &$name {
                type UntrackedRepr = untracked::$name;
            }

            impl AsRef<$name> for $name {
                fn as_ref(&self) -> &Self {
                    self
                }
            }

            impl Cast for $name {
                fn instanceof(val: &untracked::JsValue) -> bool {
                    <untracked::$name as untracked::JsCast>::instanceof(val)
                }

                fn unchecked_from_js(val: untracked::JsValue) -> Self {
                    Self { [<$base:snake>]: Cast::unchecked_from_js(val) }
                }

                #[allow(trivial_casts)]
                #[allow(unsafe_code)]
                fn unchecked_from_js_ref(val: &untracked::JsValue) -> &Self {
                    unsafe { &*(val as *const untracked::JsValue as *const Self) }
                }
            }
        }
    };
}

macro_rules! wrapper_down_conversions {
    ($name:ident [$($base:ident),*]) => {
        paste! {
            $(
                impl From<$name> for $base {
                    fn from(t: $name) -> Self {
                        t.unchecked_into()
                    }
                }

                impl From<&$name> for $base {
                    fn from(t: &$name) -> Self {
                        t.clone().unchecked_into()
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

macro_rules! wrapper_up_conversions {
    ($name:ident [$($base:ident),*]) => {
        paste! {
            $(
                impl TryFrom<$base> for $name {
                    type Error = $base;
                    fn try_from(t: $base) -> Result<Self, Self::Error> {
                        t.dyn_into()
                    }
                }

                impl TryFrom<&$base> for $name {
                    type Error = $base;
                    fn try_from(t: &$base) -> Result<Self, Self::Error> {
                        t.clone().dyn_into()
                    }
                }
            )*
        }
    };
}

macro_rules! wrapper_web_conversions {
    ($name:ident [$($base:ident),*]) => {
        impl From<untracked::$name> for $name {
            fn from(t: untracked::$name) -> Self {
                t.unchecked_into()
            }
        }

        paste! {
            $(
                impl From<$name> for untracked::$base {
                    fn from(t: $name) -> Self {
                        t.unchecked_into()
                    }
                }

                impl AsRef<untracked::$base> for $name {
                    fn as_ref(&self) -> &untracked::$base {
                        &self.untracked_js_value.unchecked_ref()
                    }
                }
            )*
        }
    };
}



// ===============
// === ValueId ===
// ===============

pub const VALUE_ID_KEY: &str = "ENSO_VALUE_ID";
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



// ===============
// === JsValue ===
// ===============

starting_wrapper! {
    /// Any JavaScript value that references are tracked in Rust, as opposed to
    /// [`untracked::JsValue`], which references are not tracked. The tracking is used to
    /// automatically remove elements, such as divs, when all Rust references to them are dropped.
    ///
    /// # Warning
    /// If you convert a tracked value to an untracked one and all the tracked values will be
    /// dropped, the target dom elements will be removed from their parents even if untracked
    /// references still exist.
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

impl TrackingInitializer for JsValue {
    fn init_tracking(&self) {
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
        found: impl FnOnce(untracked::Number) -> T,
        not_found: impl FnOnce(ValueId) -> T,
    ) -> T {
        // FIXME: slow VALUE_ID_KEY.into()
        let val = Reflect::get(&self, &VALUE_ID_KEY.into()).unwrap();
        let num = val.clone().dyn_into::<untracked::Number>();
        match num {
            Ok(num) => found(num),
            Err(_) => {
                let id = next_value_id();
                Reflect::set(&self, &VALUE_ID_KEY.into(), &untracked::Number::from(id as f64))
                    .unwrap();
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
    /// The Object type represents one of JavaScript's data types. It is used to store various keyed
    /// collections and more complex entities. Objects can be created using the [`Object`]
    /// constructor or the object initializer / literal syntax in JavaScript.
    ///
    /// To learn more, see:
    /// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object
    #[derive(Clone)]
    Object [JsValue]
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
    callback: untracked::Closure<dyn Fn(untracked::JsValue)>,
    event:    Box<dyn Any>,
}

wrapper! {
    /// The [`EventTarget`] interface is implemented by objects that can receive events and may have
    /// listeners for them. In other words, any target of events implements the three methods
    /// associated with this interface.
    ///
    /// Element, and its children, as well as Document and Window, are the most common event
    /// targets, but other objects can be event targets, too. For example XMLHttpRequest, AudioNode,
    /// and AudioContext are also event targets.
    ///
    /// Many event targets (including elements, documents, and windows) also support setting event
    /// handlers via onevent properties and attributes.
    ///
    /// To learn more, see: https://developer.mozilla.org/en-US/docs/Web/API/EventTarget
    #[derive(Clone)]
    EventTarget [Object, JsValue]
}

impl Drop for EventTarget {
    fn drop(&mut self) {
        if value_ref_count(self.value_id()) == 1 {
            LISTENERS.with(|listeners| {
                listeners.borrow_mut().remove(&self.value_id());
                // We do not need to unregister listeners as the object is dropped.
            })
        }
    }
}

impl EventTarget {
    pub fn on_event<E: frp::Data>(&self) -> frp::Sampler<E>
    where E: From<untracked::JsValue> {
        let network = frp::Network::new("event_listener");
        frp::extend! { network
            src <- source::<E>();
            event <- src.sampler();
            trace src;
        }

        let callback = untracked::Closure::<dyn Fn(untracked::JsValue)>::new(
            move |js_val: untracked::JsValue| {
                src.emit(E::from(js_val));
            },
        );
        let callback_js = callback.as_ref().unchecked_ref();
        self.untracked_repr().add_event_listener_with_callback("mousedown", callback_js).unwrap();

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
    /// The DOM [`Node`] interface is an abstract base class upon which many other DOM API objects
    /// are based, thus letting those object types to be used similarly and often interchangeably.
    /// As an abstract class, there is no such thing as a plain [`Node`] object. All objects that
    /// implement [`Node`] functionality are based on one of its subclasses. Most notable are
    /// Document, Element, and DocumentFragment.
    ///
    /// In addition, every kind of DOM node is represented by an interface based on [`Node`]. These
    /// include Attr, CharacterData (which Text, Comment, CDATASection and ProcessingInstruction are
    /// all based on), and DocumentType.
    ///
    /// In some cases, a particular feature of the base Node interface may not apply to one of its
    /// child interfaces; in that case, the inheriting node may return null or throw an exception,
    /// depending on circumstances. For example, attempting to add children to a node type that
    /// cannot have children will throw an exception.
    ///
    /// To learn more, see: https://developer.mozilla.org/en-US/docs/Web/API/Node
    #[derive(Clone)]
    Node [EventTarget, Object, JsValue]
}

impl Drop for Node {
    fn drop(&mut self) {
        if value_ref_count(self.value_id()) == 1 {
            self.remove_from_parent();
        }
    }
}

impl Node {
    pub fn append_child(&self, child: &Node) {
        self.untracked_repr().append_child(child.untracked_repr()).unwrap();
    }

    pub fn remove_child(&self, child: &Node) -> bool {
        self.untracked_repr().remove_child(child.untracked_repr()).is_ok()
    }

    pub fn parent(&self) -> Option<Node> {
        self.untracked_repr().parent_node().map(|parent| parent.unchecked_into())
    }

    pub fn remove_from_parent(&self) -> bool {
        self.parent().map(|parent| parent.remove_child(self)).unwrap_or(false)
    }
}


// ===============
// === Element ===
// ===============

wrapper! {
    /// [`Element`] is the most general base class from which all element objects (i.e. objects that
    /// represent elements) in a Document inherit. It only has methods and properties common to all
    /// kinds of elements. More specific classes inherit from [`Element`].
    ///
    /// For example, the [`HtmlElement`] interface is the base interface for HTML elements, while
    /// the SVGElement interface is the basis for all SVG elements. Most functionality is specified
    /// further down the class hierarchy.
    ///
    /// Languages outside the realm of the Web platform, like XUL through the XULElement interface,
    /// also implement [`Element`].
    ///
    /// To learn more, see: https://developer.mozilla.org/en-US/docs/Web/API/Element
    #[derive(Clone)]
    Element [Node, EventTarget, Object, JsValue]
}



// ===================
// === HtmlElement ===
// ===================

wrapper! {
    /// The [`HtmlElement`] interface represents any HTML element. Some elements directly implement
    /// this interface, while others implement it via an interface that inherits it.
    ///
    /// To learn more, see: https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement
    #[derive(Clone)]
    HtmlElement [Element, Node, EventTarget, Object, JsValue]
}

impl HtmlElement {
    pub fn set_width(&self, width: f64) -> &Self {
        self.untracked_repr().set_style_or_warn("width", &format!("{}px", width));
        self
    }

    pub fn set_height(&self, width: f64) -> &Self {
        self.untracked_repr().set_style_or_warn("height", &format!("{}px", width));
        self
    }

    pub fn set_background(&self, background: &str) -> &Self {
        self.untracked_repr().set_style_or_warn("background", background);
        self
    }

    pub fn set_display(&self, display: &str) -> &Self {
        self.untracked_repr().set_style_or_warn("display", display);
        self
    }

    pub fn set_border_radius(&self, radius: f64) -> &Self {
        self.untracked_repr().set_style_or_warn("border-radius", &format!("{}px", radius));
        self
    }
}



// ======================
// === HtmlDivElement ===
// ======================

/// Short version of [`HtmlDivElement`].
pub type Div = HtmlDivElement;

wrapper! {
    /// The [`HtmlDivElement`] interface provides special properties (beyond the regular
    /// [`HtmlElement`] interface it also has available to it by inheritance) for manipulating
    /// `<div>` elements.
    ///
    /// To learn more, see: https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement
    #[derive(Clone)]
    HtmlDivElement [HtmlElement, Element, Node, EventTarget, Object, JsValue]
}

impl Default for HtmlDivElement {
    fn default() -> Self {
        Self::new()
    }
}

impl HtmlDivElement {
    pub fn new() -> Self {
        Self::from(document.create_div_or_panic())
    }
}

impl HtmlDivElement {
    fn init_tracking(&self) {}
}
