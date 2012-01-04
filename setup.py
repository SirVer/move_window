from distutils.core import setup
from distutils.extension import Extension
from Cython.Distutils import build_ext

# extra_link_args are appended to LDLAGS which make frameworks appear to late
# on the commandline for gcc. Fix this problem by adding it to the environment
# which gets prepended to the LDFLAGS of distutils.
import os
os.environ['LDFLAGS'] = '-framework Carbon'

setup(
    cmdclass = {'build_ext': build_ext},
    ext_modules = [
        Extension("_os_helper", ["_os_helper.pyx", "pid.c"],
            extra_link_args = ["-framework Carbon"],
        )]
)

