# Move Window

Move window is a tool that I use to rearrange windows under Mac OS X without
touching the mouse.

New windows open in Mac OS X wherever they want. Command line enthusiasts hate
to grab the mouse and placing windows is a pain. `move_window` to the rescue:
it lets you position your windows very quickly and cleanly via the commandline
-- most useful in conjunction with Launchbar or Quicksilver.

## Installation

The tool should work on every reasonably modern Mac Os. It uses the
Accessibility APIs to move windows around. You will need to make sure that
either your terminal (or LaunchBar or whatever you are using) is allowed to
control other programs.
Just drop the `move_window` tool into a `bin` directory, for example `/usr/local/bin`.

## Usage

The syntax is easy: `move_window` takes screen id, number of x partitions, a range,
number of y partitions, a range. Examples::

    move_window 0     # Move current window to first screen (0), fill entire screen
    move_window 021   # fill right half of first screen (screen id 0, 2 X partitions, fill second)
    move_window 02031 # first screen (0), left half (20), divide in 3 parts in y direction (3) and use middle (1)
    move_window 031-231 # first screen (0), right two third (31-2), divide in 3 parts in y direction (3) and use middle (1)

See also my [introductive blog post](http://www.sirver.net/blog/2012/01/04/move-window-done-right/) for more
information and examples.

The `contrib` directory contains an AppleScript that I use for Launchbar
Integration. Just drop into `~/Library/Application Support/LaunchBar/Actions`,
but make sure that `move_window` is in your path; otherwise edit the script.
