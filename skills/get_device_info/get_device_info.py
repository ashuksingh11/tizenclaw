#!/usr/bin/env python3
"""
TizenClaw Skill: Get Device Info
Queries Tizen Device C-API to get device information like battery charge percentage.
"""
import ctypes
import sys
import json

def get_battery_info():
    try:
        # Load the Tizen system-device library
        libdevice = ctypes.CDLL("libcapi-system-device.so.0")
    except OSError as e:
        return {"error": f"Error loading libcapi-system-device: {e}"}
        
    # int device_battery_get_percent(int *status);
    device_battery_get_percent = libdevice.device_battery_get_percent
    device_battery_get_percent.argtypes = [ctypes.POINTER(ctypes.c_int)]
    device_battery_get_percent.restype = ctypes.c_int
    
    battery_level = ctypes.c_int(0)
    ret = device_battery_get_percent(ctypes.byref(battery_level))
    
    if ret == 0:
        return {"battery_percent": battery_level.value}
    else:
        return {"error": f"Failed to get battery info, code: {ret}"}

if __name__ == "__main__":
    import os, json
    claw_args = os.environ.get("CLAW_ARGS")
    if claw_args:
        try:
            parsed = json.loads(claw_args)
            for k, v in parsed.items():
                globals()[k] = v # crude but effective mapping for args
            
            # Simple wrapper mapping based on script name
            script_name = os.path.basename(__file__)
            if "launch_app" in script_name:
                launch_app(parsed.get("app_id", ""))
                sys.exit(0)
            elif "vibrate_device" in script_name:
                print(json.dumps(vibrate(parsed.get("duration_ms", 1000))))
                sys.exit(0)
            elif "schedule_alarm" in script_name:
                print(json.dumps(schedule_prompt(parsed.get("delay_sec", 600), parsed.get("prompt_text", ""))))
                sys.exit(0)
            elif "web_search" in script_name:
                print(json.dumps(search_wikipedia(parsed.get("query", ""))))
                sys.exit(0)
        except Exception as e:
            print(json.dumps({"error": f"Failed to parse CLAW_ARGS: {e}"}))

    result = get_battery_info()
    print(json.dumps(result))
