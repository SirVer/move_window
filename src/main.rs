// TODO(hrapp): this is needed to shut up clippy warnings from `objc::msg_send`.
#![allow(unexpected_cfgs)]

use anyhow::{Context, Result, bail};
use argh::FromArgs;
use cocoa::appkit::NSScreen;
use cocoa::base::nil;
use cocoa::foundation::NSArray;
use objc::runtime::Class;
use objc::runtime::Object;
use objc::{msg_send, sel, sel_impl};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

mod axui;
mod skylight;

/// Move windows or query spaces.
#[derive(FromArgs)]
struct Args {
    #[argh(subcommand)]
    subcommand: Command,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
#[derive(FromArgs)]
#[argh(subcommand)]
enum Command {
    GetSpaceName(GetSpaceNameArgs),
    SetSpaceName(SetSpaceNameArgs),
    MoveWindow(MoveWindowArgs),
    Windows(WindowsArgs),
    Frame(FrameArgs),
}

/// Get the name of a space.
#[derive(FromArgs)]
#[argh(subcommand, name = "get_space_name")]
struct GetSpaceNameArgs {
    /// optional space id (defaults to current space)
    #[argh(option, short = 'n')]
    id: Option<u64>,
}

/// Set the name of a space.
#[derive(FromArgs)]
#[argh(subcommand, name = "set_space_name")]
struct SetSpaceNameArgs {
    /// optional space id (defaults to current space)
    #[argh(option, short = 'n')]
    id: Option<u64>,

    #[argh(positional)]
    /// new name for the space. If not given, the name is reset.
    name: Option<String>,
}

/// Move the current top level Window.
#[derive(FromArgs)]
#[argh(subcommand, name = "move_window")]
struct MoveWindowArgs {
    #[argh(positional)]
    move_command: String,

    /// target the focused window of this pid instead of the frontmost application
    #[argh(option)]
    pid: Option<i32>,

    /// print each positioning attempt and its outcome to stderr
    #[argh(switch, short = 'v')]
    verbose: bool,
}

/// Print the AX frame of the focused window as JSON.
#[derive(FromArgs)]
#[argh(subcommand, name = "frame")]
struct FrameArgs {
    /// target the focused window of this pid instead of the frontmost application
    #[argh(option)]
    pid: Option<i32>,
}

/// List the currently open Windows.
#[derive(FromArgs)]
#[argh(subcommand, name = "windows")]
struct WindowsArgs {}

#[derive(Serialize, Deserialize, Default)]
struct State {
    /// mapping from space number to space name
    space_names: HashMap<u64, String>,
}

impl State {
    /// Loads the state from the JSON file.
    fn load() -> Result<Self> {
        let state_file = state_file_path()?;
        if !state_file.exists() {
            return Ok(State::default());
        }
        let data = fs::read_to_string(&state_file).context("Failed to read state file")?;
        let state: State =
            serde_json::from_str(&data).context("Failed to parse state file as JSON")?;
        Ok(state)
    }

    /// Saves the state to the JSON file.
    fn save(self: &State) -> Result<()> {
        let state_file = state_file_path()?;
        let data = serde_json::to_string_pretty(self).context("Failed to serialize state")?;
        fs::write(&state_file, data).context("Failed to write state file")?;
        Ok(())
    }
}

/// Constructs the path to the state file in a typical macOS location.
fn state_file_path() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("Could not find HOME directory")?;
    let mut path = PathBuf::from(home).join("Library/Application Support/move_window");
    fs::create_dir_all(&path).context("Failed to create state directory")?;
    path.push("state.json");
    Ok(path)
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
    // to get them in the screen space. This must be applied to every screen, including index 0:
    // on the main screen the menu bar offset means the visible frame's top edge is not at y=0.
    for idx in 0..rv.len() {
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

fn move_window(args: MoveWindowArgs) -> Result<()> {
    let params = match MoveParameters::from_command(&args.move_command) {
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

    let pid = match args.pid.or_else(frontmost_application_pid) {
        Some(p) => p,
        None => bail!("Frontmost application has no PID."),
    };

    // Some apps (Firefox, anything with AXEnhancedUserInterface set by assistive software such as
    // Hammerspoon) honor only the last of two rapid AX requests when that attribute is true,
    // so position and size never both land. We temporarily disable it for the duration of the move
    // and restore it afterwards on every exit path, mirroring what Rectangle.app does.
    let enhanced_was_on = axui::get_enhanced_user_interface(pid).unwrap_or(false);
    if args.verbose {
        eprintln!("AXEnhancedUserInterface: {enhanced_was_on}");
        eprintln!("target frame: {frame:?}");
    }
    if enhanced_was_on {
        // A failure to disable is not fatal; we still try the move below.
        let _ = axui::set_enhanced_user_interface(pid, false);
    }

    let result = converge_window(pid, &frame, args.verbose);

    if enhanced_was_on {
        // Restore the original value even if the move failed, before propagating the error.
        let _ = axui::set_enhanced_user_interface(pid, true);
    }
    result
}

/// Drives the focused window of `pid` to `frame`, retrying because AX move/resize is async and
/// some apps drop or reorder requests.
fn converge_window(pid: i32, frame: &Rect, verbose: bool) -> Result<()> {
    // We try really hard to move the window into the right position, but give up after a handful of
    // attempts. AX events are async and there are programs that eat events if they come too close
    // to each other (Firefox for example), so we alternate which command we send first on every
    // try; if the 2nd is eaten, it gets through on the next try. With AXEnhancedUserInterface
    // disabled (see caller) well-behaved apps land in the very first attempt.
    let mut order = axui::Order::MoveResize;
    let mut previous: Option<Rect> = None;
    for attempt in 0..10 {
        // Give async apps a moment to catch up between attempts. There is no delay before the first
        // attempt so well-behaved apps stay as fast as they were.
        if attempt > 0 {
            std::thread::sleep(std::time::Duration::from_millis(15));
        }
        axui::move_frontmost_window(pid, frame, order)?;
        order = order.swap();
        let actual = axui::get_window_position_and_size(pid)?;
        if verbose {
            eprintln!("attempt {attempt}: actual {actual:?}");
        }
        if actual == *frame {
            if verbose {
                eprintln!("stop: reached target after {} attempt(s).", attempt + 1);
            }
            break;
        }
        // If the window did not change since the previous attempt it is stable but constrained
        // (e.g. an app minimum size), so retrying further is pointless. Only from the second
        // attempt onward, since the first attempt has no previous frame to compare against.
        if let Some(prev) = &previous
            && *prev == actual
        {
            if verbose {
                eprintln!("stop: window stable at {actual:?} (not equal to target), accepting.");
            }
            break;
        }
        if verbose && attempt == 9 {
            eprintln!("stop: giving up after 10 attempts; last actual {actual:?}.");
        }
        previous = Some(actual);
    }
    Ok(())
}

fn print_frame(args: FrameArgs) -> Result<()> {
    let pid = match args.pid.or_else(frontmost_application_pid) {
        Some(p) => p,
        None => bail!("Frontmost application has no PID."),
    };
    let frame = axui::get_window_position_and_size(pid)?;
    println!("{}", serde_json::to_string(&frame).unwrap());
    Ok(())
}

fn set_space_name(args: SetSpaceNameArgs) -> Result<()> {
    let mut state = State::load()?;
    let id = args.id.unwrap_or_else(skylight::get_current_space_id);
    match args.name {
        None => {
            state.space_names.remove(&id);
        }
        Some(s) => {
            state.space_names.insert(id, s);
        }
    }

    state.save()?;
    Ok(())
}

fn get_space_name(args: GetSpaceNameArgs) -> Result<()> {
    let state = State::load()?;
    let id = args.id.unwrap_or_else(skylight::get_current_space_id);
    let name = match state.space_names.get(&id) {
        Some(s) => s as &str,
        None => "Unnamed",
    };
    println!("{name}");
    Ok(())
}

fn windows(_args: WindowsArgs) -> Result<()> {
    let out = axui::window_list(true);
    let o = serde_json::to_string(&out).unwrap();
    println!("{o}");
    Ok(())
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();
    if !axui::check_accessibility_permission() {
        panic!("Accessibility permissions not granted. Please enable them in System Settings.");
    }

    match args.subcommand {
        Command::MoveWindow(args) => move_window(args)?,
        Command::GetSpaceName(args) => get_space_name(args)?,
        Command::SetSpaceName(args) => set_space_name(args)?,
        Command::Windows(args) => windows(args)?,
        Command::Frame(args) => print_frame(args)?,
    };

    Ok(())
}
