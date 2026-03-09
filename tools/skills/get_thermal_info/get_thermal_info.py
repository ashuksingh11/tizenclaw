#!/usr/bin/env python3
import ctypes
import json
import os
import sys

sys.path.append(os.path.join(os.path.dirname(__file__), "..", "common"))
import tizen_capi_utils


def get_thermal_info():
    try:
        lib = tizen_capi_utils.load_library(
            ["libcapi-system-device.so.0", "libcapi-system-device.so.1"]
        )

        # enum device_thermal_e { DEVICE_THERMAL_AP = 0, DEVICE_THERMAL_CP, DEVICE_THERMAL_BATTERY }
        DEVICE_THERMAL_AP = 0

        # int device_thermal_get_temperature(device_thermal_e position, int *temp)
        lib.device_thermal_get_temperature.argtypes = [
            ctypes.c_int, ctypes.POINTER(ctypes.c_int)
        ]
        lib.device_thermal_get_temperature.restype = ctypes.c_int

        results = {}
        thermal_types = {
            0: "ap",       # Application Processor
            1: "cp",       # Communication Processor
            2: "battery",  # Battery
        }

        for idx, name in thermal_types.items():
            temp = ctypes.c_int()
            ret = lib.device_thermal_get_temperature(idx, ctypes.byref(temp))
            if ret == 0:
                results[name] = {"celsius": temp.value / 10.0 if temp.value > 100 else float(temp.value)}

        if not results:
            return {"error": "No thermal data available"}

        return {"thermal": results}

    except Exception as e:
        return {"error": str(e)}


if __name__ == "__main__":
    print(json.dumps(get_thermal_info()))
