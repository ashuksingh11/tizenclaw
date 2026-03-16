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

_glibc_preloaded = False

def _preload_glibc():
    """Preload glibc libc.so.6 to make glibc-only symbols available.

    When running on glibc Python, RTLD_GLOBAL correctly exposes
    symbols like __isoc23_sscanf to subsequently loaded CAPI libs.
    On musl Python this is a no-op (musl ignores RTLD_GLOBAL).
    """
    global _glibc_preloaded
    if _glibc_preloaded:
        return
    _glibc_preloaded = True

    search_paths = [
        "/host_lib/libc.so.6",  # container: host /lib mounted
        "/lib/libc.so.6",       # usrmerge or host-direct
        "/lib64/libc.so.6",     # x86_64 multilib convention
        "/usr/lib/libc.so.6",   # usrmerge systems
        "/usr/lib64/libc.so.6", # x86_64 Tizen emulator
    ]
    for path in search_paths:
        if os.path.exists(path):
            try:
                ctypes.CDLL(path, mode=ctypes.RTLD_GLOBAL)
                return
            except OSError:
                pass

def load_library(libnames):
    if isinstance(libnames, str):
        libnames = [libnames]

    _preload_glibc()

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
