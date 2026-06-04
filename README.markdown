# Move Window

Move window is a tool that I use to rearrange windows under Mac OS X without touching the mouse.

New windows open in Mac OS X wherever they want. Command line enthusiasts hate to grab the mouse and
placing windows is a pain. `move_window` to the rescue: it lets you position your windows very quickly and
cleanly via the commandline -- most useful in conjunction with Launchbar or Quicksilver.

## Installation

The tool should work on every reasonably modern Mac Os. It uses the Accessibility APIs to move windows
around. You will need to make sure that either your terminal (or LaunchBar or whatever you are using) is
allowed to control other programs. Just drop the `move_window` tool into a `bin` directory, for example
`/usr/local/bin`.

## Usage

The syntax is easy: `move_window` takes screen id, number of x partitions, a range, number of y partitions,
a range. Examples::

    move_window 0     # Move current window to first screen (0), fill entire screen
    move_window 021   # fill right half of first screen (screen id 0, 2 X partitions, fill second)
    move_window 02031 # first screen (0), left half (20), divide in 3 parts in y direction (3) and use middle (1)
    move_window 031-231 # first screen (0), right two third (31-2), divide in 3 parts in y direction (3) and use middle (1)

See also my [introductive blog post](http://www.sirver.net/blog/2012/01/04/move-window-done-right/) for
more information and examples.

The `contrib` directory contains an AppleScript that I use for Launchbar Integration. Just drop into
`~/Library/Application Support/LaunchBar/Actions`, but make sure that `move_window` is in your path;
otherwise edit the script. It also has Hammerspoon integration.

### Hammerspoon chooser

`contrib/spaces_chooser.lua` is a self-contained Hammerspoon module that opens an `hs.chooser` listing all
spaces (by name) and jumps to the selected one via `hs.spaces.gotoSpace`. Copy it to `~/.hammerspoon/`,
then in your `init.lua`:

    local spacesChooser = require("spaces_chooser")
    -- spacesChooser.binary = "/usr/local/bin/move_window"  -- if not ~/bin/move_window
    hs.hotkey.bind({ "ctrl", "alt" }, "s", function()
        spacesChooser.show()
    end)

The top of the file documents how to bind it into an existing `hs.hotkey.modal` as well.

## Spaces

`move_window` can also attach names to your Mission Control spaces and list them. Names are stored in
`~/Library/Application Support/move_window/state.json`, keyed by the macOS managed space id.

    move_window get_space_name        # print the name of the current space ("Unnamed" if none)
    move_window get_space_name -n 5   # print the name of space with id 5
    move_window set_space_name Mail   # name the current space "Mail"
    move_window set_space_name -n 5 Mail  # name the space with id 5 "Mail"
    move_window set_space_name        # clear the name of the current space

### Listing spaces

`list_spaces` enumerates every Mission Control space, grouped by display, joined with the names you
assigned. The leading `*` marks the current space. The index column is the number you see in Mission
Control and only counts normal user spaces; fullscreen-app spaces are still listed but labeled and do not
consume an index.

    move_window list_spaces

    Display Main
    *   1       1  Work
        2       4  Mail
        3       5  Unnamed
        -       9  Unnamed (fullscreen)

Pass `--json` for machine-readable output (one object per display, each with a `spaces` array of
`{index, id, uuid, type, name, current}`):

    move_window list_spaces --json
