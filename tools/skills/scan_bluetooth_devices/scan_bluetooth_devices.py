#!/usr/bin/env python3
"""
scan_bluetooth_devices - BT device discovery using tizen-core event loop.

Supports two actions:
- 'bonded': List paired/bonded devices (synchronous, using bt_adapter_foreach_bonded_device)
- 'scan': Discover nearby devices (asynchronous, using bt_adapter_start_device_discovery + tizen-core)
"""
import ctypes
import json
import os
import sys

sys.path.append(os.path.join(os.path.dirname(__file__), "..", "common"))
import tizen_capi_utils


class BtDeviceInfo(ctypes.Structure):
    """bt_device_info_s structure."""
    _fields_ = [
        ("remote_address", ctypes.c_char_p),
        ("remote_name", ctypes.c_char_p),
        ("bt_class", ctypes.c_int * 3),  # major_device_class, minor_device_class, major_service_class
        ("service_uuid", ctypes.POINTER(ctypes.c_char_p)),
        ("service_count", ctypes.c_int),
        ("is_bonded", ctypes.c_bool),
        ("is_connected", ctypes.c_bool),
        ("is_authorized", ctypes.c_bool),
    ]


class BtDiscoveryInfo(ctypes.Structure):
    """bt_adapter_device_discovery_info_s structure."""
    _fields_ = [
        ("remote_address", ctypes.c_char_p),
        ("remote_name", ctypes.c_char_p),
        ("bt_class", ctypes.c_int * 3),
        ("rssi", ctypes.c_int),
        ("is_bonded", ctypes.c_bool),
        ("service_uuid", ctypes.POINTER(ctypes.c_char_p)),
        ("service_count", ctypes.c_int),
        ("appearance", ctypes.c_int),
        ("manufacturer_data_len", ctypes.c_int),
        ("manufacturer_data", ctypes.c_char_p),
    ]


# Callback types
# bool (*bt_adapter_bonded_device_cb)(bt_device_info_s *device_info, void *user_data)
BT_BONDED_DEVICE_CB = ctypes.CFUNCTYPE(ctypes.c_bool, ctypes.POINTER(BtDeviceInfo), ctypes.c_void_p)
# void (*bt_adapter_device_discovery_state_changed_cb)(int result, bt_adapter_device_discovery_state_e state, bt_adapter_device_discovery_info_s *info, void *user_data)
BT_DISCOVERY_STATE_CB = ctypes.CFUNCTYPE(None, ctypes.c_int, ctypes.c_int, ctypes.POINTER(BtDiscoveryInfo), ctypes.c_void_p)
TIZEN_CORE_TASK_CB = ctypes.CFUNCTYPE(ctypes.c_bool, ctypes.c_void_p)


def scan_bluetooth_devices():
    try:
        args = json.loads(os.environ.get("CLAW_ARGS", "{}"))
        action = args.get("action", "bonded")

        bt_lib = tizen_capi_utils.load_library(
            ["libcapi-network-bluetooth.so.0", "libcapi-network-bluetooth.so"]
        )

        # bt_initialize / bt_deinitialize
        bt_lib.bt_initialize.argtypes = []
        bt_lib.bt_initialize.restype = ctypes.c_int
        bt_lib.bt_deinitialize.argtypes = []
        bt_lib.bt_deinitialize.restype = ctypes.c_int

        ret = bt_lib.bt_initialize()
        if ret != 0:
            return {"error": f"bt_initialize failed: {ret}"}

        if action == "bonded":
            # Synchronous: list bonded devices
            bt_lib.bt_adapter_foreach_bonded_device.argtypes = [BT_BONDED_DEVICE_CB, ctypes.c_void_p]
            bt_lib.bt_adapter_foreach_bonded_device.restype = ctypes.c_int

            devices = []

            def on_bonded_device(device_info, user_data):
                if device_info:
                    info = device_info.contents
                    devices.append({
                        "address": info.remote_address.decode("utf-8") if info.remote_address else "",
                        "name": info.remote_name.decode("utf-8") if info.remote_name else "(unknown)",
                        "is_connected": info.is_connected,
                        "is_authorized": info.is_authorized,
                    })
                return True

            cb = BT_BONDED_DEVICE_CB(on_bonded_device)
            bt_lib.bt_adapter_foreach_bonded_device(cb, None)
            bt_lib.bt_deinitialize()

            return {"action": "bonded", "devices": devices, "count": len(devices)}

        elif action == "scan":
            # Asynchronous: discover nearby devices via tizen-core
            core_lib = tizen_capi_utils.load_library(
                ["libtizen-core.so.0", "libtizen-core.so"]
            )

            core_lib.tizen_core_init.argtypes = []
            core_lib.tizen_core_init.restype = None
            core_lib.tizen_core_shutdown.argtypes = []
            core_lib.tizen_core_shutdown.restype = None
            core_lib.tizen_core_task_create.argtypes = [ctypes.c_char_p, ctypes.c_bool, ctypes.POINTER(ctypes.c_void_p)]
            core_lib.tizen_core_task_create.restype = ctypes.c_int
            core_lib.tizen_core_task_destroy.argtypes = [ctypes.c_void_p]
            core_lib.tizen_core_task_destroy.restype = ctypes.c_int
            core_lib.tizen_core_task_run.argtypes = [ctypes.c_void_p]
            core_lib.tizen_core_task_run.restype = ctypes.c_int
            core_lib.tizen_core_task_quit.argtypes = [ctypes.c_void_p]
            core_lib.tizen_core_task_quit.restype = ctypes.c_int
            core_lib.tizen_core_task_get_tizen_core.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_void_p)]
            core_lib.tizen_core_task_get_tizen_core.restype = ctypes.c_int
            core_lib.tizen_core_add_idle_job.argtypes = [ctypes.c_void_p, TIZEN_CORE_TASK_CB, ctypes.c_void_p, ctypes.POINTER(ctypes.c_void_p)]
            core_lib.tizen_core_add_idle_job.restype = ctypes.c_int
            core_lib.tizen_core_add_timer.argtypes = [ctypes.c_void_p, ctypes.c_uint, TIZEN_CORE_TASK_CB, ctypes.c_void_p, ctypes.POINTER(ctypes.c_void_p)]
            core_lib.tizen_core_add_timer.restype = ctypes.c_int

            bt_lib.bt_adapter_start_device_discovery.argtypes = []
            bt_lib.bt_adapter_start_device_discovery.restype = ctypes.c_int
            bt_lib.bt_adapter_stop_device_discovery.argtypes = []
            bt_lib.bt_adapter_stop_device_discovery.restype = ctypes.c_int
            bt_lib.bt_adapter_set_device_discovery_state_changed_cb.argtypes = [BT_DISCOVERY_STATE_CB, ctypes.c_void_p]
            bt_lib.bt_adapter_set_device_discovery_state_changed_cb.restype = ctypes.c_int
            bt_lib.bt_adapter_unset_device_discovery_state_changed_cb.argtypes = []
            bt_lib.bt_adapter_unset_device_discovery_state_changed_cb.restype = ctypes.c_int

            devices = []
            scan_error = [None]
            task_handle = ctypes.c_void_p()

            # bt_adapter_device_discovery_state_e: 0=STARTED, 1=FOUND, 2=FINISHED
            def on_discovery_state(result_code, state, discovery_info, user_data):
                if state == 1 and discovery_info:  # FOUND
                    info = discovery_info.contents
                    addr = info.remote_address.decode("utf-8") if info.remote_address else ""
                    # Deduplicate
                    if not any(d["address"] == addr for d in devices):
                        devices.append({
                            "address": addr,
                            "name": info.remote_name.decode("utf-8") if info.remote_name else "(unknown)",
                            "rssi_dbm": info.rssi,
                            "is_bonded": info.is_bonded,
                        })
                elif state == 2:  # FINISHED
                    core_lib.tizen_core_task_quit(task_handle)

            def on_idle(user_data):
                ret = bt_lib.bt_adapter_start_device_discovery()
                if ret != 0:
                    scan_error[0] = f"bt_adapter_start_device_discovery failed: {ret}"
                    core_lib.tizen_core_task_quit(task_handle)
                return False

            def on_timeout(user_data):
                bt_lib.bt_adapter_stop_device_discovery()
                core_lib.tizen_core_task_quit(task_handle)
                return False

            core_lib.tizen_core_init()
            ret = core_lib.tizen_core_task_create(b"main", False, ctypes.byref(task_handle))
            if ret != 0:
                bt_lib.bt_deinitialize()
                core_lib.tizen_core_shutdown()
                return {"error": f"tizen_core_task_create failed: {ret}"}

            core_handle = ctypes.c_void_p()
            core_lib.tizen_core_task_get_tizen_core(task_handle, ctypes.byref(core_handle))

            discovery_cb = BT_DISCOVERY_STATE_CB(on_discovery_state)
            bt_lib.bt_adapter_set_device_discovery_state_changed_cb(discovery_cb, None)

            idle_cb = TIZEN_CORE_TASK_CB(on_idle)
            idle_source = ctypes.c_void_p()
            core_lib.tizen_core_add_idle_job(core_handle, idle_cb, None, ctypes.byref(idle_source))

            timeout_cb = TIZEN_CORE_TASK_CB(on_timeout)
            timer_source = ctypes.c_void_p()
            core_lib.tizen_core_add_timer(core_handle, 15000, timeout_cb, None, ctypes.byref(timer_source))

            core_lib.tizen_core_task_run(task_handle)

            bt_lib.bt_adapter_unset_device_discovery_state_changed_cb()
            bt_lib.bt_deinitialize()
            core_lib.tizen_core_task_destroy(task_handle)
            core_lib.tizen_core_shutdown()

            if scan_error[0]:
                return {"error": scan_error[0], "devices": devices}

            devices.sort(key=lambda x: x.get("rssi_dbm", -100), reverse=True)
            return {"action": "scan", "devices": devices, "count": len(devices)}

        else:
            bt_lib.bt_deinitialize()
            return {"error": f"Unknown action: {action}. Use 'scan' or 'bonded'."}

    except Exception as e:
        return {"error": str(e)}


if __name__ == "__main__":
    print(json.dumps(scan_bluetooth_devices()))
