#!/usr/bin/env python3
"""
TizenClaw Skill: Vibrate Device
Uses Tizen CAPI system device haptic functions to trigger a vibration.
"""
import ctypes
import sys
import json

def vibrate(duration_ms=1000, feedback=100):
    try:
        libdevice = ctypes.CDLL("libcapi-system-device.so.0")
    except OSError as e:
        return {"error": f"Error loading libcapi-system-device: {e}"}

    # Set up function signatures
    device_haptic_open = libdevice.device_haptic_open
    device_haptic_open.argtypes = [ctypes.c_int, ctypes.POINTER(ctypes.c_void_p)]
    device_haptic_open.restype = ctypes.c_int

    device_haptic_vibrate = libdevice.device_haptic_vibrate
    device_haptic_vibrate.argtypes = [ctypes.c_void_p, ctypes.c_int, ctypes.c_int, ctypes.POINTER(ctypes.c_void_p)]
    device_haptic_vibrate.restype = ctypes.c_int

    device_haptic_close = libdevice.device_haptic_close
    device_haptic_close.argtypes = [ctypes.c_void_p]
    device_haptic_close.restype = ctypes.c_int

    haptic_handle = ctypes.c_void_p()
    # 0 implies the primary haptic device
    ret = device_haptic_open(0, ctypes.byref(haptic_handle))
    if ret != 0:
        return {"error": f"Failed to open haptic device, code: {ret}"}

    effect_handle = ctypes.c_void_p()
    ret = device_haptic_vibrate(haptic_handle, duration_ms, feedback, ctypes.byref(effect_handle))
    if ret != 0:
        device_haptic_close(haptic_handle)
        return {"error": f"Failed to vibrate, code: {ret}"}

    # Clean up (we don't wait for completion here, we let the system vibrate asynchronously and return immediately)
    device_haptic_close(haptic_handle)
    return {"status": "success", "duration_ms": duration_ms}

if __name__ == "__main__":
    import os
    claw_args = os.environ.get("CLAW_ARGS")
    if claw_args:
        try:
            parsed = json.loads(claw_args)
            duration = parsed.get("duration_ms", 1000)
            print(json.dumps(vibrate(duration)))
            sys.exit(0)
        except Exception as e:
            print(json.dumps({"error": f"Failed to parse CLAW_ARGS: {e}"}))
            sys.exit(1)

    dur = 1000
    if len(sys.argv) > 1:
        try:
            dur = int(sys.argv[1])
        except ValueError:
            pass
    print(json.dumps(vibrate(dur)))
