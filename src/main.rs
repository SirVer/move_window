// TODO(hrapp): this is needed to shut up clippy warnings from `objc::msg_send`.
#![allow(unexpected_cfgs)]

use anyhow::{Result, bail};
use argh::FromArgs;
use cocoa::appkit::NSScreen;
use cocoa::base::nil;
use cocoa::foundation::NSArray;
use objc::runtime::Class;
use objc::runtime::Object;
use objc::{msg_send, sel, sel_impl};

mod axui;
mod skylight;

#[derive(FromArgs)]
/// Move windows or query spaces.
struct Args {
    /// prints the current space id and exits.
    #[argh(switch)]
    print_space: bool,

    /// move command. Required unless `print_space`.
    #[argh(positional)]
    move_command: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug)]
enum ScreenSelector {
    Index(usize),
    Char(char),
}

#[derive(Debug)]
struct MoveParameters {
    screen: ScreenSelector,
    x_ratio: i32,
    y_ratio: i32,
    x_start: i32,
    x_end: i32,
    y_start: i32,
    y_end: i32,
}

#[derive(Debug)]
struct Screen {
    index: u64,
    visible_frame: Rect,
    frame: Rect,
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

        let screen = {
            let c = i.next().ok_or_else(|| "No more items".to_string())?;
            match c {
                '0'..='9' => ScreenSelector::Index(c.to_digit(10).expect("by design.") as usize),
                c => ScreenSelector::Char(c),
            }
        };

        let mut params = MoveParameters {
            screen,
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
            Some(_) => params.x_end,
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
            Some(_) => return Err("No more input expected.".to_string()),
            None => return Ok(params),
        };
        Ok(params)
    }
}

fn get_screens() -> Vec<Screen> {
    let mut rv = Vec::new();
    unsafe {
        let screens: *mut Object = NSScreen::screens(nil);
        for index in 0..NSArray::count(screens) {
            let screen: *mut Object = msg_send![screens, objectAtIndex: index];
            let visible_frame = screen.visibleFrame();
            let frame = screen.frame();
            rv.push(Screen {
                index,
                visible_frame: Rect {
                    x: visible_frame.origin.x as i32,
                    y: visible_frame.origin.y as i32,
                    width: visible_frame.size.width as i32,
                    height: visible_frame.size.height as i32,
                },
                frame: Rect {
                    x: frame.origin.x as i32,
                    y: frame.origin.y as i32,
                    width: frame.size.width as i32,
                    height: frame.size.height as i32,
                },
            })
        }
    };
    // The window frames have their origins in the bottom left of the screen, y going upwards.
    // However, screen bounds have the origin at the top left going down. We need to convert here
    // to get them in the screen space.
    for idx in 1..rv.len() {
        let y = rv[0].frame.height - rv[idx].visible_frame.height - rv[idx].visible_frame.y;
        rv[idx].visible_frame.y = y;
    }
    rv
}

fn frontmost_application_pid() -> Option<i32> {
    unsafe {
        let workspace_class = Class::get("NSWorkspace").expect("always there.");
        let wspace: *mut Object = msg_send![workspace_class, sharedWorkspace];
        let front_app: *mut Object = msg_send![wspace, frontmostApplication];
        if front_app == nil {
            return None;
        }
        Some(msg_send![front_app, processIdentifier])
    }
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();

    if !axui::check_accessibility_permission() {
        panic!("Accessibility permissions not granted. Please enable them in System Settings.");
    }

    if args.print_space {
        let id = skylight::get_current_space_id();
        println!("{id}");
        return Ok(());
    }

    let Some(move_command) = args.move_command else {
        bail!("Usage: move_window <move_command>");
    };

    let params = match MoveParameters::from_command(&move_command) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let screens = get_screens();
    let get_screen_by_index = |index| {
        let mut screen = None;
        for s in &screens {
            if s.index == index as u64 {
                screen = Some(s)
            }
        }
        screen.expect("Unknown screen index.")
    };
    let screen = match params.screen {
        ScreenSelector::Index(index) => get_screen_by_index(index),
        ScreenSelector::Char(c) => match c {
            'm' | 'c' => get_screen_by_index(0),
            'l' => screens
                .iter()
                .min_by_key(|s| s.frame.x)
                .expect("there are always screens."),
            'r' => screens
                .iter()
                .max_by_key(|s| s.frame.x)
                .expect("there are always screens."),
            't' | 'u' => screens
                .iter()
                .max_by_key(|s| s.frame.y)
                .expect("there are always screens."),
            'b' | 'd' => screens
                .iter()
                .min_by_key(|s| s.frame.y)
                .expect("there are always screens."),
            _ => panic!("Unknown character for screen selection: {}", c),
        },
    };

    let width = f64::from(screen.visible_frame.width) / f64::from(params.x_ratio);
    let height = f64::from(screen.visible_frame.height) / f64::from(params.y_ratio);
    let frame = Rect {
        x: (f64::from(screen.visible_frame.x) + width * f64::from(params.x_start)).round() as i32,
        y: (f64::from(screen.visible_frame.y) + height * f64::from(params.y_start)).round() as i32,
        width: (width * f64::from(params.x_end - params.x_start + 1)).round() as i32,
        height: (height * f64::from(params.y_end - params.y_start + 1)).round() as i32,
    };

    let pid = match frontmost_application_pid() {
        Some(p) => p,
        None => bail!("Frontmost application has no PID."),
    };
    // We try really hard to move the windo into the right position, but give up
    // after 10 times if it does not work.
    for _ in 0..10 {
        axui::move_frontmost_window(pid, &frame)?;
        if axui::get_window_position_and_size(pid)? == frame {
            break;
        }
    }
    Ok(())
}
