from distutils.core import setup
from distutils.extension import Extension

# extra_link_args are appended to LDLAGS which make frameworks appear to late
# on the commandline for gcc. Fix this problem by adding it to the environment
# which gets prepended to the LDFLAGS of distutils.
import os
os.environ['LDFLAGS'] = '-framework Carbon -framework ApplicationServices'


extra_setup_args = {}
os_helper = Extension("_mv_helper",
    ["_os_helper.cpp", "_os_interface.cpp"],
    extra_link_args = ["-framework Carbon", "-framework ApplicationServices"],
    language="c++",
)

setup(
        name = 'move_window',
        version = '1.0',
        url = 'https://github.com/sirver/move_window',
        author = 'Holger "SirVer" Rapp',
        author_email = 'sirver@gmx.de',
        description = "Move_window let's you move your windows on Mac OS X via the cmdline.",
        long_description = """\
New windows open in Mac OS X wherever they want. Command line enthusiasts hate
to grab the mouse and placing windows is a pain. move_window.py to the rescue:
it lets you position your windows very quickly and cleanly via the commandline
- most useful in conjunction with Launchbar or Quicksilver.

The syntax is easy: move_window takes screen id, number of x partitions, a range,
number of y partitions, a range done. Examples:

move_window 0 # Move current window to first screen (0), fill entire screen
move_window 021 # fill left half of first screen (screen id 0, 2 X partitions, fill first)
move_window 02031 # first screen (0), left half (20), divide in 3 parts in y direction (3) and use middle (1)
  """,
  classifiers = [
      "Development Status :: 5 - Production/Stable",
      "License :: OSI Approved :: GPLv3",
      "Operating System :: OS Independent",
      "Programming Language :: Python",
      "Programming Language :: Python :: 2",
      "Programming Language :: Cython",
      ],
  install_requires=[
      'appscript',
  ],
  scripts = [
      'move_window',
  ],
  ext_modules = [ os_helper ],
  **extra_setup_args
)

