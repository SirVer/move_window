// TODO(sirver): The crate does not seem to work with Rust 2018 yet, so we need this old school
// import.
#[macro_use]
extern crate objc;

use cocoa::appkit::NSScreen;
use cocoa::base::nil;
use cocoa::foundation::{NSDictionary, NSString};
use objc::runtime::Class;
use objc::runtime::Object;
use osascript::JavaScript;
use serde_derive::Serialize;
use std::ffi::CStr;

#[derive(Serialize)]
struct MoveWindowParams {
    app_name: String,
    r: Rect,
}

#[derive(Debug, Serialize)]
struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

#[derive(Debug)]
struct MoveParameters {
    screen_index: usize,
    x_ratio: i32,
    y_ratio: i32,
    x_start: i32,
    x_end: i32,
    y_start: i32,
    y_end: i32,
}

fn next_integer(a: &mut ::std::iter::Peekable<impl Iterator<Item = char>>) -> Result<i32, String> {
    let c = a.next().ok_or_else(|| "No more items".to_string())?;
    let v = c
        .to_digit(10)
        .ok_or_else(|| "Next not an integer.".to_string())?;
    Ok(v as i32)
}

impl MoveParameters {
    pub fn from_command(s: &str) -> Result<Self, String> {
        let mut i = s.chars().peekable();

        let mut params = MoveParameters {
            screen_index: next_integer(&mut i)? as usize,
            x_ratio: 1,
            y_ratio: 1,
            x_start: 0,
            x_end: 0,
            y_start: 0,
            y_end: 0,
        };

        params.x_ratio = match i.peek() {
            Some(_) => next_integer(&mut i)?,
            None => return Ok(params),
        };
        params.x_start = match i.peek() {
            Some(_) => next_integer(&mut i)?,
            None => return Ok(params),
        };
        params.x_end = params.x_start;
        params.x_end = match i.peek() {
            Some(&'-') => {
                i.next();
                next_integer(&mut i)?
            }
            Some(_) => return Err("Expected number.".to_string()),
            None => return Ok(params),
        };

        params.y_ratio = match i.peek() {
            Some(_) => next_integer(&mut i)?,
            None => return Ok(params),
        };
        params.y_start = match i.peek() {
            Some(_) => next_integer(&mut i)?,
            None => return Ok(params),
        };
        params.y_end = params.y_start;
        params.y_end = match i.peek() {
            Some(&'-') => {
                i.next();
                next_integer(&mut i)?
            }
            Some(_) => return Err("Expected number.".to_string()),
            None => return Ok(params),
        };

        Ok(params)
    }
}

fn get_screen_dimensions(screen_index: usize) -> Rect {
    let s = unsafe {
        let screens = NSScreen::screens(nil);
        let screen: *mut Object = msg_send![screens, objectAtIndex: screen_index];
        screen.visibleFrame()
    };
    Rect {
        x: s.origin.x as i32,
        y: s.origin.x as i32,
        width: s.size.width as i32,
        height: s.size.height as i32,
    }
}

fn get_frontmost_application_name() -> String {
    let app_name = unsafe {
        let workspace_class = Class::get("NSWorkspace").unwrap();
        let wspace: *mut Object = msg_send![workspace_class, sharedWorkspace];
        let active_app: *mut Object = msg_send![wspace, activeApplication];
        let v = active_app.objectForKey_(NSString::alloc(nil).init_str("NSApplicationName"));
        let k = v.UTF8String();
        CStr::from_ptr(k)
    };
    app_name.to_str().unwrap().to_string()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: move_window <move_command>");
        std::process::exit(1);
    }
    let params = match MoveParameters::from_command(&args[1]) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let screen = get_screen_dimensions(params.screen_index);
    let width = f64::from(screen.width) / f64::from(params.x_ratio);
    let height = f64::from(screen.height) / f64::from(params.y_ratio);
    let frame = Rect {
        x: (f64::from(screen.x) + width * f64::from(params.x_start)).round() as i32,
        y: (f64::from(screen.y) + height * f64::from(params.y_start)).round() as i32,
        width: (width * f64::from(params.x_end - params.x_start + 1)).round() as i32,
        height: (height * f64::from(params.y_end - params.y_start + 1)).round() as i32,
    };

    let app_name = get_frontmost_application_name();
    let script = JavaScript::new(
        "
        var app = Application($params.app_name);
        app.windows[0].bounds = {
            x: $params.r.x,
            y: $params.r.y,
            width: $params.r.width,
            height: $params.r.height,
        };
    ",
    );

    script
        .execute_with_params::<_, ()>(MoveWindowParams { app_name, r: frame })
        .unwrap();
}
