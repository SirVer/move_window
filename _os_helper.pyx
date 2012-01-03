
cdef extern from "pid.h":
    void frontmost_process(char*, int)

def frontmost_process():
    cdef char buffer[256]
    frontmost_process(buffer, 256)
    return buffer


