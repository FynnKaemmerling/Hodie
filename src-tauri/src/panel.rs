//! Turns the quick entry window into a non-activating panel on macOS, like
//! Spotlight. A normal window activates Hodie when shown, and macOS then
//! switches to Hodie's space instead of showing it over the current app.

use objc2::runtime::{AnyObject, NSObject};
use objc2::{ClassType, MainThreadOnly, define_class};
use objc2_app_kit::{
    NSPanel, NSResponder, NSWindow, NSWindowCollectionBehavior, NSWindowStyleMask,
};
use tauri::WebviewWindow;

define_class!(
    #[unsafe(super(NSPanel, NSWindow, NSResponder, NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "HodieQuickAddPanel"]
    struct QuickAddPanel;

    impl QuickAddPanel {
        // Borderless windows refuse keyboard focus by default.
        #[unsafe(method(canBecomeKeyWindow))]
        fn can_become_key_window(&self) -> bool {
            true
        }

        #[unsafe(method(canBecomeMainWindow))]
        fn can_become_main_window(&self) -> bool {
            false
        }
    }
);

fn ns_window(window: &WebviewWindow) -> Option<&NSWindow> {
    let ptr = window.ns_window().ok()? as *const NSWindow;
    // SAFETY: Tauri returns the window's live NSWindow; it outlives the call.
    unsafe { ptr.as_ref() }
}

/// Must run on the main thread, once, before the window is first shown.
pub fn make_panel(window: &WebviewWindow) {
    let Some(ns_window) = ns_window(window) else {
        return;
    };
    // SAFETY: Tao's window class only adds methods and one ivar to NSWindow,
    // and NSPanel adds no ivars, so the object's memory fits the new class.
    unsafe {
        AnyObject::set_class(ns_window, QuickAddPanel::class());
    }
    ns_window.setStyleMask(ns_window.styleMask() | NSWindowStyleMask::NonactivatingPanel);
    ns_window.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::FullScreenAuxiliary,
    );
    ns_window.setHidesOnDeactivate(false);
}

/// Shows the panel above the current app without activating Hodie.
pub fn show(window: &WebviewWindow) {
    if let Some(ns_window) = ns_window(window) {
        ns_window.orderFrontRegardless();
        ns_window.makeKeyWindow();
    }
}
