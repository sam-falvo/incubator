//! Mediator

use crate::stencil::Stencil;

/// Provides an interface
/// to the host application environment.
///
/// For example,
/// on a Linux system,
/// the host application environment will be controlled
/// from the SDL2 event loop.
/// This trait provides a way to interact with that event loop.
pub trait Mediator {
    /// Sets the framebuffer dirty rectangle to the whole stencil.
    /// The entire contents of the desktop stencil will be
    /// transferred to the backing frame buffer as soon as possible.
    ///
    /// **NOTE:** This method might not perform the framebuffer update
    /// right away.
    fn repaint_all(&mut self);

    /// Request that the application quit.
    /// Note that this *does not* quit the application immediately.
    /// See also [[AppEventSink::request_quit]].
    fn quit(&mut self);

    /// Returns a mutable reference to a stencil representing the desktop.
    /// From this, you may draw things on it,
    /// query its dimensions,
    /// etc.
    fn borrow_mut_desktop(&mut self) -> &mut Stencil;
}

