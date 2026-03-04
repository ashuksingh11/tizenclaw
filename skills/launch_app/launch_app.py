#!/usr/bin/env python3
"""
TizenClaw Skill: Launch App
Uses Tizen CAPI app_control to launch applications.
"""
import ctypes
import sys

def launch_app(app_id):
    try:
        # Load the Tizen app_control library
        libappcontrol = ctypes.CDLL("libcapi-appfw-app-control.so.0")
    except OSError as e:
        print(f"Error loading libcapi-appfw-app-control.so.0: {e}")
        sys.exit(1)

    app_control_h = ctypes.c_void_p()
    
    # Initialize functions mapping
    app_control_create = libappcontrol.app_control_create
    app_control_create.argtypes = [ctypes.POINTER(ctypes.c_void_p)]
    app_control_create.restype = ctypes.c_int

    app_control_set_app_id = libappcontrol.app_control_set_app_id
    app_control_set_app_id.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
    app_control_set_app_id.restype = ctypes.c_int

    app_control_send_launch_request = libappcontrol.app_control_send_launch_request
    app_control_send_launch_request.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p]
    app_control_send_launch_request.restype = ctypes.c_int

    app_control_destroy = libappcontrol.app_control_destroy
    app_control_destroy.argtypes = [ctypes.c_void_p]
    app_control_destroy.restype = ctypes.c_int

    # 1. Create app_control
    ret = app_control_create(ctypes.byref(app_control_h))
    if ret != 0:
        print(f"app_control_create failed with code: {ret}")
        sys.exit(1)

    # 2. Set App ID
    b_app_id = app_id.encode('utf-8')
    ret = app_control_set_app_id(app_control_h, b_app_id)
    if ret != 0:
        print(f"app_control_set_app_id failed: {ret}")
        app_control_destroy(app_control_h)
        sys.exit(1)

    # 3. Send launch request
    ret = app_control_send_launch_request(app_control_h, None, None)
    if ret != 0:
        print(f"app_control_send_launch_request failed: {ret}")
        app_control_destroy(app_control_h)
        sys.exit(1)

    print(f"Successfully launched app: {app_id}")
    app_control_destroy(app_control_h)

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

    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <app_id>")
        sys.exit(1)
    
    target_app_id = sys.argv[1]
    launch_app(target_app_id)
