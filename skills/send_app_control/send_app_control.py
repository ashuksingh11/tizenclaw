#!/usr/bin/env python3
"""
TizenClaw Skill: send_app_control
Send an app control request to launch an application.
- If app_id is provided, launches that specific app (explicit intent).
- If app_id is omitted, uses operation/URI/MIME to find the best match (implicit intent).
- At least one of app_id or operation must be provided.
"""
import ctypes
import json
import os
import sys

sys.path.append(os.path.join(os.path.dirname(__file__), "..", "common"))
import tizen_capi_utils


def send_app_control():
    try:
        args = json.loads(os.environ.get("CLAW_ARGS", "{}"))
        app_id = args.get("app_id", "")
        operation = args.get("operation", "")
        uri = args.get("uri", "")
        mime = args.get("mime", "")
        extra_data = args.get("extra_data", {})

        if not app_id and not operation:
            return {"error": "Invalid argument: at least one of app_id or operation must be provided"}

        ac_lib = tizen_capi_utils.load_library(
            ["libcapi-appfw-app-control.so.0", "libcapi-appfw-app-control.so"]
        )

        # Function signatures
        ac_lib.app_control_create.argtypes = [ctypes.POINTER(ctypes.c_void_p)]
        ac_lib.app_control_create.restype = ctypes.c_int
        ac_lib.app_control_destroy.argtypes = [ctypes.c_void_p]
        ac_lib.app_control_destroy.restype = ctypes.c_int
        ac_lib.app_control_set_operation.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
        ac_lib.app_control_set_operation.restype = ctypes.c_int
        ac_lib.app_control_set_app_id.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
        ac_lib.app_control_set_app_id.restype = ctypes.c_int
        ac_lib.app_control_set_uri.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
        ac_lib.app_control_set_uri.restype = ctypes.c_int
        ac_lib.app_control_set_mime.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
        ac_lib.app_control_set_mime.restype = ctypes.c_int
        ac_lib.app_control_add_extra_data.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
        ac_lib.app_control_add_extra_data.restype = ctypes.c_int
        ac_lib.app_control_send_launch_request.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p]
        ac_lib.app_control_send_launch_request.restype = ctypes.c_int

        # Create app_control handle
        handle = ctypes.c_void_p()
        ret = ac_lib.app_control_create(ctypes.byref(handle))
        if ret != 0:
            return {"error": f"app_control_create failed: {ret}"}

        # Set operation (use default if app_id is given but operation is not)
        if operation:
            ac_lib.app_control_set_operation(handle, operation.encode("utf-8"))
        elif app_id:
            ac_lib.app_control_set_operation(
                handle,
                b"http://tizen.org/appcontrol/operation/default",
            )

        # Set app_id if provided (explicit intent)
        if app_id:
            ac_lib.app_control_set_app_id(handle, app_id.encode("utf-8"))

        # Set URI if provided
        if uri:
            ac_lib.app_control_set_uri(handle, uri.encode("utf-8"))

        # Set MIME if provided
        if mime:
            ac_lib.app_control_set_mime(handle, mime.encode("utf-8"))

        # Add extra data if provided
        if extra_data and isinstance(extra_data, dict):
            for k, v in extra_data.items():
                ac_lib.app_control_add_extra_data(
                    handle, k.encode("utf-8"), str(v).encode("utf-8")
                )

        # Send launch request
        ret = ac_lib.app_control_send_launch_request(handle, None, None)
        ac_lib.app_control_destroy(handle)

        if ret != 0:
            error_map = {
                -22: "INVALID_PARAMETER",
                -38: "APP_NOT_FOUND",
                -12: "OUT_OF_MEMORY",
                -13: "PERMISSION_DENIED",
            }
            return {
                "error": f"Launch failed: {error_map.get(ret, ret)}",
                "code": ret,
                "app_id": app_id or None,
                "operation": operation or None,
            }

        result = {"result": "launched"}
        if app_id:
            result["app_id"] = app_id
        if operation:
            result["operation"] = operation
        if uri:
            result["uri"] = uri
        if mime:
            result["mime"] = mime
        return result

    except Exception as e:
        return {"error": str(e)}


if __name__ == "__main__":
    print(json.dumps(send_app_control()))
