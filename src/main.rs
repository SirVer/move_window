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
    ListSpaces(ListSpacesArgs),
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

/// List all Mission Control spaces grouped by display.
#[derive(FromArgs)]
#[argh(subcommand, name = "list_spaces")]
struct ListSpacesArgs {
    /// emit the listing as JSON instead of a human-readable table
    #[argh(switch)]
    json: bool,
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
    if args.verbose {
        let enhanced = axui::get_enhanced_user_interface(pid);
        eprintln!("AXEnhancedUserInterface: {enhanced:?}");
        eprintln!("target frame: {frame:?}");
    }
    // We try really hard to move the windo into the right position, but give up
    // after 10 times if it does not work.
    // Also, AX events are async and there are programs that eat events if they come to close to
    // each other (Firefox for example). This is an attempt to work around this; we alternate which
    // command we send first on every try, so if the 2nd is eaten, we get it through on the next
    // try.
    let mut order = axui::Order::MoveResize;
    for attempt in 0..10 {
        axui::move_frontmost_window(pid, &frame, order)?;
        order = order.swap();
        let actual = axui::get_window_position_and_size(pid)?;
        if args.verbose {
            eprintln!("attempt {attempt}: actual {actual:?}");
        }
        if actual == frame {
            break;
        }
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

/// One space as exposed to the user (table row or JSON object).
#[derive(Serialize)]
struct SpaceEntry {
    /// The Mission Control index of this space on its display, or `None` for
    /// fullscreen spaces which do not consume a user-space index.
    index: Option<u64>,
    /// The managed space id (same value `set_space_name` expects).
    id: u64,
    /// The SkyLight space uuid, if available.
    uuid: Option<String>,
    /// The kind of space (user, fullscreen, ...).
    #[serde(rename = "type")]
    space_type: skylight::SpaceType,
    /// The user-assigned name, or "Unnamed".
    name: String,
    /// Whether this is the currently active space.
    current: bool,
}

/// One display section with its spaces.
#[derive(Serialize)]
struct DisplayEntry {
    display: String,
    spaces: Vec<SpaceEntry>,
}

/// Collects all spaces per display and joins them with user-assigned names.
fn collect_display_entries() -> Result<Vec<DisplayEntry>> {
    let state = State::load()?;
    let active_id = skylight::get_current_space_id();
    let displays = skylight::managed_display_spaces();

    let mut out = Vec::new();
    for display in displays {
        let mut user_index = 0u64;
        let mut spaces = Vec::new();
        for space in &display.spaces {
            let is_user = space.space_type == skylight::SpaceType::User;
            let index = if is_user {
                user_index += 1;
                Some(user_index)
            } else {
                None
            };
            let current =
                space.id == active_id || display.current_space_id == Some(space.id);
            let name = match state.space_names.get(&space.id) {
                Some(s) => s.clone(),
                None => "Unnamed".to_string(),
            };
            spaces.push(SpaceEntry {
                index,
                id: space.id,
                uuid: space.uuid.clone(),
                space_type: space.space_type,
                name,
                current,
            });
        }
        out.push(DisplayEntry {
            display: display.display,
            spaces,
        });
    }
    Ok(out)
}

fn list_spaces(args: ListSpacesArgs) -> Result<()> {
    let entries = collect_display_entries()?;

    if args.json {
        println!("{}", serde_json::to_string(&entries)?);
        return Ok(());
    }

    for display in &entries {
        println!("Display {}", display.display);
        for space in &display.spaces {
            let index = match space.index {
                Some(i) => i.to_string(),
                None => "-".to_string(),
            };
            let kind = match space.space_type {
                skylight::SpaceType::User => "",
                skylight::SpaceType::Fullscreen => " (fullscreen)",
                skylight::SpaceType::Other => " (other)",
            };
            let marker = if space.current { "*" } else { " " };
            println!(
                "{} {:>3}  {:>6}  {}{}",
                marker, index, space.id, space.name, kind
            );
        }
    }
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
        Command::ListSpaces(args) => list_spaces(args)?,
        Command::Windows(args) => windows(args)?,
        Command::Frame(args) => print_frame(args)?,
    };

    Ok(())
}
