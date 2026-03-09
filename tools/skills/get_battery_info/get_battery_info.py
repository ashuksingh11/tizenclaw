#!/usr/bin/env python3
import ctypes
import json
import os
import sys

# Add common directory to path to import tizen_capi_utils
sys.path.append(os.path.join(os.path.dirname(__file__), "..", "common"))
import tizen_capi_utils

def get_battery_info():
    try:
        lib = tizen_capi_utils.load_library(["libcapi-system-device.so.0", "libcapi-system-device.so.1"])
        
        # Define functions
        # int device_battery_get_percent(int *percent)
        device_battery_get_percent = lib.device_battery_get_percent
        device_battery_get_percent.argtypes = [ctypes.POINTER(ctypes.c_int)]
        
        # int device_battery_is_charging(bool *is_charging)
        device_battery_is_charging = lib.device_battery_is_charging
        device_battery_is_charging.argtypes = [ctypes.POINTER(ctypes.c_bool)]
        
        # int device_battery_get_level_status(device_battery_level_e *status)
        device_battery_get_level_status = lib.device_battery_get_level_status
        device_battery_get_level_status.argtypes = [ctypes.POINTER(ctypes.c_int)]
        
        # Start calls
        percent = ctypes.c_int()
        device_battery_get_percent(ctypes.byref(percent))
        
        charging = ctypes.c_bool()
        device_battery_is_charging(ctypes.byref(charging))
        
        level_status = ctypes.c_int()
        device_battery_get_level_status(ctypes.byref(level_status))
        
        # device_battery_level_e mapping
        levels = {1: "normal", 2: "low", 3: "critical", 4: "empty", 5: "full"}
        
        return {
            "percent": percent.value,
            "is_charging": charging.value,
            "level_status": levels.get(level_status.value, "unknown")
        }

    except Exception as e:
        return {"error": str(e)}

if __name__ == "__main__":
    print(json.dumps(get_battery_info()))
