#!/usr/bin/env python3
"""
TizenClaw Skill: Schedule Alarm
Uses Tizen Alarm API to wake a specific application at a given time
by registering an alarm with an app_control targeting that app.
"""
import ctypes
import json
import os
import sys
from datetime import datetime


class StructTm(ctypes.Structure):
    """C struct tm for alarm_schedule_once_at_date."""

    _fields_ = [
        ("tm_sec", ctypes.c_int),
        ("tm_min", ctypes.c_int),
        ("tm_hour", ctypes.c_int),
        ("tm_mday", ctypes.c_int),
        ("tm_mon", ctypes.c_int),
        ("tm_year", ctypes.c_int),
        ("tm_wday", ctypes.c_int),
        ("tm_yday", ctypes.c_int),
        ("tm_isdst", ctypes.c_int),
    ]


def _datetime_to_struct_tm(dt):
    """Convert a Python datetime to a StructTm instance."""
    tm = StructTm()
    tm.tm_sec = dt.second
    tm.tm_min = dt.minute
    tm.tm_hour = dt.hour
    tm.tm_mday = dt.day
    tm.tm_mon = dt.month - 1       # struct tm: 0-11
    tm.tm_year = dt.year - 1900    # struct tm: years since 1900
    tm.tm_wday = (dt.weekday() + 1) % 7  # struct tm: Sun=0
    tm.tm_yday = dt.timetuple().tm_yday - 1
    tm.tm_isdst = -1               # let system decide
    return tm


_ALARM_ERRORS = {
    -0x01100000: "INVALID_PARAMETER",
    -0x01100001: "INVALID_TIME",
    -0x01100002: "INVALID_DATE",
    -0x01100003: "CONNECTION_FAIL",
    -0x01100004: "NOT_PERMITTED_APP",
    -0x01100005: "OUT_OF_MEMORY",
    -0x01100006: "PERMISSION_DENIED",
}


def schedule_alarm(app_id, datetime_str):
    try:
        dt = datetime.fromisoformat(datetime_str)
    except (ValueError, TypeError) as e:
        return {"error": f"Invalid datetime format: {e}"}

    try:
        libappcontrol = ctypes.CDLL("libcapi-appfw-app-control.so.0")
        libalarm = ctypes.CDLL("libcapi-appfw-alarm.so.0")
    except OSError as e:
        return {"error": f"Error loading Tizen libraries: {e}"}

    # --- app_control bindings ---
    libappcontrol.app_control_create.argtypes = [
        ctypes.POINTER(ctypes.c_void_p),
    ]
    libappcontrol.app_control_create.restype = ctypes.c_int

    libappcontrol.app_control_set_app_id.argtypes = [
        ctypes.c_void_p,
        ctypes.c_char_p,
    ]
    libappcontrol.app_control_set_app_id.restype = ctypes.c_int

    libappcontrol.app_control_destroy.argtypes = [ctypes.c_void_p]
    libappcontrol.app_control_destroy.restype = ctypes.c_int

    # --- alarm bindings ---
    # int alarm_schedule_once_at_date(
    #     app_control_h app_control, struct tm *date, int *alarm_id)
    libalarm.alarm_schedule_once_at_date.argtypes = [
        ctypes.c_void_p,
        ctypes.POINTER(StructTm),
        ctypes.POINTER(ctypes.c_int),
    ]
    libalarm.alarm_schedule_once_at_date.restype = ctypes.c_int

    # 1. Create app_control
    app_control_h = ctypes.c_void_p()
    ret = libappcontrol.app_control_create(ctypes.byref(app_control_h))
    if ret != 0:
        return {"error": f"app_control_create failed: {ret}"}

    # 2. Set target app ID
    b_app_id = app_id.encode("utf-8")
    ret = libappcontrol.app_control_set_app_id(app_control_h, b_app_id)
    if ret != 0:
        libappcontrol.app_control_destroy(app_control_h)
        return {"error": f"app_control_set_app_id failed: {ret}"}

    # 3. Schedule alarm at the specified time
    tm = _datetime_to_struct_tm(dt)
    alarm_id = ctypes.c_int(0)
    ret = libalarm.alarm_schedule_once_at_date(
        app_control_h, ctypes.byref(tm), ctypes.byref(alarm_id)
    )

    libappcontrol.app_control_destroy(app_control_h)

    if ret != 0:
        err_name = _ALARM_ERRORS.get(ret, "UNKNOWN")
        return {"error": f"alarm_schedule_once_at_date failed: {err_name} ({ret})"}

    return {
        "status": "success",
        "alarm_id": alarm_id.value,
        "app_id": app_id,
        "scheduled_time": datetime_str,
        "message": (
            f"Alarm set: {app_id} will be launched at {datetime_str}"
        ),
    }


if __name__ == "__main__":
    claw_args = os.environ.get("CLAW_ARGS")
    if claw_args:
        try:
            parsed = json.loads(claw_args)
            aid = parsed.get("app_id", "")
            dt_str = parsed.get("datetime", "")
            if aid and dt_str:
                print(json.dumps(schedule_alarm(aid, dt_str)))
                sys.exit(0)
        except Exception as e:
            print(json.dumps({"error": f"Failed to parse CLAW_ARGS: {e}"}))
            sys.exit(1)

    if len(sys.argv) < 3:
        print(json.dumps({
            "error": (
                f"Usage: {sys.argv[0]} <app_id> "
                "<datetime: YYYY-MM-DDTHH:MM:SS>"
            ),
        }))
        sys.exit(1)

    print(json.dumps(schedule_alarm(sys.argv[1], sys.argv[2])))
