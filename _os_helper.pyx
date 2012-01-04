
cdef extern:
    void get_frontmost_process(char*, int)

def frontmost_process():
    """Return the name of the application that owns the frontmost window"""
    cdef char buffer[256]
    get_frontmost_process(buffer, 256)
    return buffer


