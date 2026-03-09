import ctypes
import os

# Tizen Common Error Codes (as per tizen_error.h)
TIZEN_ERROR_NONE = 0
TIZEN_ERROR_PERMISSION_DENIED = -13
TIZEN_ERROR_NOT_SUPPORTED = -15
TIZEN_ERROR_INVALID_PARAMETER = -22

class TizenError(Exception):
    def __init__(self, code, message):
        self.code = code
        self.message = message
        super().__init__(f"{message} (error code: {code})")

def check_return(ret, message):
    if ret != TIZEN_ERROR_NONE:
        raise TizenError(ret, message)

def load_library(libnames):
    if isinstance(libnames, str):
        libnames = [libnames]
    
    errors = []
    for libname in libnames:
        try:
            return ctypes.CDLL(libname)
        except OSError as e:
            errors.append(f"{libname}: {e}")
    
    raise ImportError(f"Failed to load any of the libraries {libnames}. Errors: {'; '.join(errors)}")

def get_char_ptr():
    return ctypes.c_char_p()

def decode_ptr(ptr):
    return ptr.value.decode('utf-8') if ptr.value else ""
