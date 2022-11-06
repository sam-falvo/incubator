//! ROSE Events
//! 
//! Events are generated from a centralized ROSE event loop.
//! This event loop tends to be platform specific,
//! and so is abstracted from the ROSE application.
//! This allows an application developed under a hosted OS
//! (e.g., Linux using the SDL2 library and event loop)
//! to be deployed on a target which uses lower-level facilities.
//!
//! Because ROSE has no idea what application it's running,
//! events are low-level and represent individual interactions
//! with the operator.
//! It is up to the application to interpret these events
//! into something actionable.
//! Thankfully, a gadget library exists to make this easier
//! for the most commonly used types of gadgets.

use crate::types::Point;
use crate::mediator::Mediator;

/// Application controller.
///
/// Calls to `init_root` will always return a `Box<dyn AppController>`.
///
/// In general,
/// the top-level application state structure
/// will also be the global application controller.
pub trait AppController: MouseEventSink<()> + AppEventSink {}

/// A sink for application events.
pub trait AppEventSink {
    /// Called when something requests the application to shut down.
    ///
    /// Answers `true` if it's safe to quit the application.
    /// Otherwise, `false` is returned.
    ///
    /// If `false` is returned, the application *will not* quit.
    /// This provides an application an opportunity
    /// to save any unsaved state to storage before
    /// actually quitting.
    ///
    /// **NOTE:** Don't abuse this function by always returning `false`.
    /// The host operating system will almost certainly
    /// have a way to force the termination of your program regardless.
    ///
    /// See also [[Mediator::quit]].
    fn request_quit(&self) -> bool;
}

/// A sink for raw mouse events.
///
/// Although this trait can technically be implemented by anything,
/// it is intended to be implemented by views.
///
/// All trait methods return a value of type T.
/// If `T` is not `()`,
/// then `T` must represent some type capable of
/// expressing higher-level or more semantically-relevant events.
/// For example,
/// `PropGadgetView` implements this trait setting
/// T to `PropGadgetEvent`.
/// This allows the proportional gadget
/// to convert raw mouse motion events
/// into a higher-level "knob moved" event, when appropriate.
pub trait MouseEventSink<T> {
    /// Called when the mouse pointer moves.
    fn pointer_moved(&mut self, med: &mut dyn Mediator, to: Point) -> T;
    /// Called when the mouse button is released.
    fn button_up(&mut self, med: &mut dyn Mediator) -> T;
    /// Called when the mouse button is pressed.
    fn button_down(&mut self, med: &mut dyn Mediator) -> T;
    /// Called when the mouse enters into a view's boundary.
    fn enter(&mut self, med: &mut dyn Mediator, at: Point) -> T;
    /// Called when the mouse leaves a view's boundary.
    fn leave(&mut self, med: &mut dyn Mediator) -> T;
}

