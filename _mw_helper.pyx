from libcpp.vector cimport vector
from libc.stdint cimport *

cdef extern:
    void _frontmost_process(char*, int32_t)
    void _display_resolutions(vector[vector[int32_t]]*)

def frontmost_process():
    """Return the name of the application that owns the frontmost window"""
    cdef char buffer[256]
    _frontmost_process(buffer, 256)
    return buffer

def get_resolutions():
    cdef vector[vector[int32_t]] resolutions
    cdef int32_t x,y,i

    rv = []
    _display_resolutions(&resolutions)
    for i in xrange(resolutions.size()):
        rv.append( [resolutions[i][0], resolutions[i][1]] )
    return rv




