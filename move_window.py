#!/usr/bin/env python
# encoding: utf-8

import argparse
import re
import subprocess

import _os_helper

import appscript as AS

def parse_args():
    parser = argparse.ArgumentParser(description='Move windows via AppleEvents')
    parser.add_argument('cmd', type=str, default="0",
                       help='<screen=0><xratio=1><xpos=0[-ypos]><yratio=1><ypos=0[-ypos]>')

    return parser.parse_args()

def get_resolutions():
    sp = subprocess.Popen(["system_profiler", "SPDisplaysDataType"], stdout=subprocess.PIPE)

    rv = []
    for line in sp.stdout:
        if "Resolution" in line:
            m = re.search(r"(\d+)\s*x\s*(\d+)", line)
            rv.append( [int(m.group(1)), int(m.group(2))] )
    return rv

def move_window(x, y, w, h):
    print "In move window"
    capp_name = _os_helper.frontmost_process()
    app = AS.Application(capp_name)
    print "capp_name: %s" % (capp_name)
    try:
        app.windows[1].position.set((x, y))
        app.windows[1].size.set((w, h))
    except AttributeError:
        app.windows[1].bounds.set((x, y, x + w, y + h))
    print "all done"

def main():
    print "Parsing args"
    cmd = parse_args().cmd
    print "getting resolutions"
    res = get_resolutions()
    print "Calculating"

    screen, cmd = (0, cmd) if not cmd else (int(cmd[0]), cmd[1:])
    xratio, cmd = (1, cmd) if not cmd else (int(cmd[0]), cmd[1:])
    xpos_s, cmd = (0, cmd) if not cmd else (int(cmd[0]), cmd[1:])
    xpos_e = xpos_s
    if cmd and cmd[0] == '-':
        xpos_e, cmd = (int(cmd[1]), cmd[2:])
    yratio, cmd = (1, cmd) if not cmd else (int(cmd[0]), cmd[1:])
    ypos_s, cmd = (0, cmd) if not cmd else (int(cmd[0]), cmd[1:])
    ypos_e = ypos_s
    if cmd and cmd[0] == '-':
        ypos_e, cmd = (int(cmd[1]), cmd[2:])

    if screen == 0:
        x, y = 0, 22
        res[0][1] -= 22
    else:
        x = -res[1][0]
        y = 0

    assert(screen in [0,1])

    one_width = (res[screen][0] / xratio)
    one_height = (res[screen][1] / yratio)

    w = one_width * (xpos_e - xpos_s + 1)
    h = one_height * (ypos_e - ypos_s + 1)
    x += one_width * xpos_s
    y += one_height * ypos_s

    move_window(x, y, w, h)



if __name__ == '__main__':
    main()
