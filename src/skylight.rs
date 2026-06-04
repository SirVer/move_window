use core_foundation::{
    array::CFArray,
    base::{CFType, TCFType},
    dictionary::CFDictionary,
    number::CFNumber,
    string::CFString,
};

/// The kind of a Mission Control space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SpaceType {
    /// A normal user space (the kind you create in Mission Control).
    User,
    /// A space hosting a single fullscreen application.
    Fullscreen,
    /// Any other space type reported by SkyLight.
    Other,
}

impl SpaceType {
    fn from_raw(raw: i64) -> Self {
        match raw {
            0 => SpaceType::User,
            4 => SpaceType::Fullscreen,
            _ => SpaceType::Other,
        }
    }
}

/// A single Mission Control space on a display.
#[derive(Debug, Clone)]
pub struct Space {
    /// The 64 bit managed space id (same family as `CGSGetActiveSpace`).
    pub id: u64,
    /// The space uuid as reported by SkyLight, if present.
    pub uuid: Option<String>,
    /// The kind of space (user, fullscreen, ...).
    pub space_type: SpaceType,
}

/// All spaces belonging to a single managed display.
#[derive(Debug, Clone)]
pub struct DisplaySpaces {
    /// The display identifier (e.g. "Main").
    pub display: String,
    /// The managed id of the currently active space on this display, if known.
    pub current_space_id: Option<u64>,
    /// All spaces on this display, in Mission Control order.
    pub spaces: Vec<Space>,
}

pub fn get_current_space_id() -> u64 {
    unsafe {
        let conn = SLSMainConnectionID();
        CGSGetActiveSpace(conn)
    }
}

/// Reads an `u64` from a CFNumber stored under `key` in `dict`.
fn dict_u64(dict: &CFDictionary<CFString, CFType>, key: &str) -> Option<u64> {
    dict.find(CFString::new(key))
        .and_then(|v| v.downcast::<CFNumber>())
        .and_then(|n| n.to_i64())
        .map(|n| n as u64)
}

/// Reads an `i64` from a CFNumber stored under `key` in `dict`.
fn dict_i64(dict: &CFDictionary<CFString, CFType>, key: &str) -> Option<i64> {
    dict.find(CFString::new(key))
        .and_then(|v| v.downcast::<CFNumber>())
        .and_then(|n| n.to_i64())
}

/// Reads a string from a CFString stored under `key` in `dict`.
fn dict_string(dict: &CFDictionary<CFString, CFType>, key: &str) -> Option<String> {
    dict.find(CFString::new(key))
        .and_then(|v| v.downcast::<CFString>())
        .map(|s| s.to_string())
}

/// Reads a nested CFDictionary stored under `key` in `dict`.
fn dict_dict(
    dict: &CFDictionary<CFString, CFType>,
    key: &str,
) -> Option<CFDictionary<CFString, CFType>> {
    dict.find(CFString::new(key)).map(|v| unsafe {
        CFDictionary::<CFString, CFType>::wrap_under_get_rule(v.as_CFTypeRef() as _)
    })
}

/// Parses a single space dictionary as returned inside the "Spaces" array.
fn space_from_cf_dict(dict: &CFDictionary<CFString, CFType>) -> Option<Space> {
    // The managed space id is reported under "ManagedSpaceID"; some macOS
    // versions only carry "id64". Accept either.
    let id = dict_u64(dict, "ManagedSpaceID").or_else(|| dict_u64(dict, "id64"))?;
    let uuid = dict_string(dict, "uuid");
    let space_type = SpaceType::from_raw(dict_i64(dict, "type").unwrap_or(0));
    Some(Space {
        id,
        uuid,
        space_type,
    })
}

/// Enumerates all Mission Control spaces grouped by managed display.
///
/// Wraps the private `SLSCopyManagedDisplaySpaces` SkyLight function. The
/// returned CF array follows the create rule, so we wrap it under the create
/// rule and let core-foundation release it when it goes out of scope.
pub fn managed_display_spaces() -> Vec<DisplaySpaces> {
    let array = unsafe {
        let conn = SLSMainConnectionID();
        let array_ref = SLSCopyManagedDisplaySpaces(conn);
        if array_ref.is_null() {
            return Vec::new();
        }
        CFArray::<CFDictionary<CFString, CFType>>::wrap_under_create_rule(array_ref as _)
    };

    let mut displays = Vec::new();
    for display_dict in array.iter() {
        let display =
            dict_string(&display_dict, "Display Identifier").unwrap_or_else(|| "Unknown".into());

        let current_space_id = dict_dict(&display_dict, "Current Space").and_then(|cur| {
            dict_u64(&cur, "ManagedSpaceID").or_else(|| dict_u64(&cur, "id64"))
        });

        let spaces = match display_dict.find(CFString::new("Spaces")) {
            None => Vec::new(),
            Some(value) => {
                let spaces_array = unsafe {
                    CFArray::<CFDictionary<CFString, CFType>>::wrap_under_get_rule(
                        value.as_CFTypeRef() as _,
                    )
                };
                spaces_array
                    .iter()
                    .filter_map(|d| space_from_cf_dict(&d))
                    .collect()
            }
        };

        displays.push(DisplaySpaces {
            display,
            current_space_id,
            spaces,
        });
    }
    displays
}

#[link(name = "SkyLight", kind = "framework")]
unsafe extern "C" {
    fn CGSGetActiveSpace(conn: u32) -> u64;
    fn SLSMainConnectionID() -> u32;
    fn SLSCopyManagedDisplaySpaces(conn: u32) -> core_foundation::array::CFArrayRef;
}
