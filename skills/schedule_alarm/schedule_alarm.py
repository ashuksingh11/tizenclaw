#!/usr/bin/env python3
"""
TizenClaw Skill: Schedule Alarm (Cron/Timer)
Uses Tizen Appfw Alarm API to schedule a delayed task (prompt sent back to TizenClaw).
"""
import ctypes
import sys
import json

def schedule_prompt(delay_sec, prompt_text):
    try:
        libappcontrol = ctypes.CDLL("libcapi-appfw-app-control.so.0")
        libalarm = ctypes.CDLL("libcapi-appfw-alarm.so.0")
    except OSError as e:
        return {"error": f"Error loading Tizen libraries: {e}"}

    # Prepare AppControl
    app_control_h = ctypes.c_void_p()
    
    app_control_create = libappcontrol.app_control_create
    app_control_create.argtypes = [ctypes.POINTER(ctypes.c_void_p)]
    
    app_control_set_app_id = libappcontrol.app_control_set_app_id
    app_control_set_app_id.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
    
    app_control_add_extra_data = libappcontrol.app_control_add_extra_data
    app_control_add_extra_data.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
    
    app_control_destroy = libappcontrol.app_control_destroy
    app_control_destroy.argtypes = [ctypes.c_void_p]

    libappcontrol.app_control_create(ctypes.byref(app_control_h))
    libappcontrol.app_control_set_app_id(app_control_h, b"org.tizen.tizenclaw")
    # TizenClaw's Service waits for the "prompt" extra_data to trigger agent actions
    libappcontrol.app_control_add_extra_data(app_control_h, b"prompt", prompt_text.encode('utf-8'))

    # Prepare Alarm API
    # int alarm_schedule_once_after_delay(app_control_h app_control, int delay, int *alarm_id)
    alarm_schedule = libalarm.alarm_schedule_once_after_delay
    alarm_schedule.argtypes = [ctypes.c_void_p, ctypes.c_int, ctypes.POINTER(ctypes.c_int)]
    alarm_schedule.restype = ctypes.c_int

    alarm_id = ctypes.c_int(0)
    ret = alarm_schedule(app_control_h, delay_sec, ctypes.byref(alarm_id))

    libappcontrol.app_control_destroy(app_control_h)

    if ret == 0:
        return {"status": "success", "alarm_id": alarm_id.value, "delay_sec": delay_sec, "scheduled_prompt": prompt_text}
    else:
        return {"error": f"Failed to schedule alarm (Error code: {ret})"}

if __name__ == "__main__":
    import os
    claw_args = os.environ.get("CLAW_ARGS")
    if claw_args:
        try:
            parsed = json.loads(claw_args)
            delay = parsed.get("delay_sec", 600)
            prompt_text = parsed.get("prompt_text", "")
            print(json.dumps(schedule_prompt(delay, prompt_text)))
            sys.exit(0)
        except Exception as e:
            print(json.dumps({"error": f"Failed to parse CLAW_ARGS: {e}"}))
            sys.exit(1)

    if len(sys.argv) < 3:
        print(json.dumps({"error": "Usage: schedule_alarm.py <delay_seconds> <prompt_text>"}))
        sys.exit(1)

    try:
        delay = int(sys.argv[1])
        # Enforce minimum delay of 10 minutes (600 seconds) for Tizen alarm API
        if delay < 600:
            print(json.dumps({"error": "Tizen alarm API minimum delay is 10 minutes (600 seconds)."}))
            sys.exit(1)
    except ValueError:
        print(json.dumps({"error": "Invalid delay specified, must be an integer."}))
        sys.exit(1)

    prompt = " ".join(sys.argv[2:])
    print(json.dumps(schedule_prompt(delay, prompt)))
