use crate::Rect;
use anyhow::{Result, bail};
use cocoa::base::{id, nil};
use core_foundation::{
    base::{CFTypeRef, TCFType},
    string::CFString,
};
use core_graphics::geometry::{CGPoint, CGSize};

/// Returns true if the binary can access the accesibility APIs.
pub fn check_accessibility_permission() -> bool {
    unsafe { AXIsProcessTrusted() }
}

fn frontmost_window_id(pid: i32) -> Result<id> {
    unsafe {
        // Create AXUIElement reference to the app
        let ax_app = AXUIElementCreateApplication(pid);
        if ax_app == nil {
            bail!("Unable to create AXUIElement reference to front app.");
        }

        // Get the focused window from the app
        let mut focused_window: id = nil;
        let result: i32 = AXUIElementCopyAttributeValue(
            ax_app,
            CFString::new("AXFocusedWindow").as_concrete_TypeRef() as CFTypeRef,
            &mut focused_window,
        );
        if result != 0 || focused_window == nil {
            bail!("Unable to retrieve focused window. Accessibility permission required!",);
        }
        Ok(focused_window)
    }
}

/// Moves and resizes the focused window of the app with the given `pid` to `rect` using the macOS Accessibility API (native).
pub fn move_frontmost_window(pid: i32, rect: &Rect) -> Result<()> {
    unsafe {
        let focused_window = frontmost_window_id(pid)?;

        // Set window size
        let new_size = CGSize::new(rect.width as f64, rect.height as f64);
        let size_value = AXValueCreate(AXValueType::CGSize, &new_size as *const _ as *const _);

        let set_size_result = AXUIElementSetAttributeValue(
            focused_window,
            CFString::new("AXSize").as_concrete_TypeRef() as CFTypeRef,
            size_value,
        );
        if set_size_result != 0 {
            bail!("Failed to set window size.");
        }

        // Set window position (top-left)
        let new_position = CGPoint::new(rect.x as f64, rect.y as f64);
        let pos_value = AXValueCreate(AXValueType::CGPoint, &new_position as *const _ as *const _);
        let set_pos_result = AXUIElementSetAttributeValue(
            focused_window,
            CFString::new("AXPosition").as_concrete_TypeRef() as CFTypeRef,
            pos_value,
        );
        if set_pos_result != 0 {
            bail!("Failed to set window position.");
        }

        Ok(())
    }
}

pub fn get_window_position_and_size(pid: i32) -> Result<Rect> {
    unsafe {
        let focused_window = frontmost_window_id(pid)?;
        let mut pos_ref: CFTypeRef = std::ptr::null();
        let mut size_ref: CFTypeRef = std::ptr::null();

        // Get AXPosition
        let result_pos = AXUIElementCopyAttributeValue(
            focused_window,
            CFString::new("AXPosition").as_concrete_TypeRef() as CFTypeRef,
            &mut pos_ref as *mut _ as *mut _,
        );

        // Get AXSize
        let result_size = AXUIElementCopyAttributeValue(
            focused_window,
            CFString::new("AXSize").as_concrete_TypeRef() as CFTypeRef,
            &mut size_ref as *mut _ as *mut _,
        );

        if result_pos != 0 || result_size != 0 {
            bail!("Unable to get pos or size");
        }

        // Extract values
        let mut position = CGPoint::new(0.0, 0.0);
        let mut size = CGSize::new(0.0, 0.0);

        let ok_pos = AXValueGetValue(
            pos_ref,
            AXValueType::CGPoint,
            &mut position as *mut _ as *mut _,
        );
        if !ok_pos {
            bail!("Failed to get window position.");
        }
        let ok_size = AXValueGetValue(size_ref, AXValueType::CGSize, &mut size as *mut _ as *mut _);
        if !ok_size {
            bail!("Failed to get window size.");
        }

        Ok(Rect {
            x: position.x as i32,
            y: position.y as i32,
            width: size.width as i32,
            height: size.height as i32,
        })
    }
}

#[repr(C)]
enum AXValueType {
    CGPoint = 1,
    CGSize = 2,
}

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXUIElementCopyAttributeValue(element: id, attribute: CFTypeRef, value_out: *mut id) -> i32;
    fn AXUIElementSetAttributeValue(element: id, attribute: CFTypeRef, value: CFTypeRef) -> i32;
    fn AXValueCreate(typ: AXValueType, value_ptr: *const std::ffi::c_void) -> CFTypeRef;
    fn AXUIElementCreateApplication(pid: i32) -> id;
    fn AXIsProcessTrusted() -> bool;
    // fn AXValueGetType(value: CFTypeRef) -> AXValueType;
    fn AXValueGetValue(
        value: CFTypeRef,
        the_type: AXValueType,
        data: *mut std::ffi::c_void,
    ) -> bool;

}
