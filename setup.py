from distutils.core import setup
from distutils.extension import Extension
from Cython.Distutils import build_ext

# extra_link_args are appended to LDLAGS which make frameworks appear to late
# on the commandline for gcc. Fix this problem by adding it to the environment
# which gets prepended to the LDFLAGS of distutils.
import os
os.environ['LDFLAGS'] = '-framework Carbon -framework ApplicationServices'


os_helper = Extension("_os_helper",
    ["_os_helper.pyx", "pid.cpp"],
    extra_link_args = ["-framework Carbon", "-framework ApplicationServices"],
    language="c++",
)

setup(
    cmdclass = {'build_ext': build_ext},
    ext_modules = [ os_helper ]
)

