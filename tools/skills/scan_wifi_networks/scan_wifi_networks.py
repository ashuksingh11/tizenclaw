#!/usr/bin/env python3
"""
scan_wifi_networks - Async WiFi scan using tizen-core event loop.

This is a proof-of-concept demonstrating how to use tizen-core's event loop
from Python FFI to handle asynchronous Tizen APIs.

Pattern:
1. tizen_core_init() -> tizen_core_task_create("main", false)
2. tizen_core_add_idle_job() -> starts wifi_manager_scan() in idle callback
3. scan finished callback iterates APs, stores results, quits loop
4. tizen_core_task_run() blocks until quit -> return results
"""
import ctypes
import json
import os
import sys
import threading
import time

sys.path.append(os.path.join(os.path.dirname(__file__), "..", "common"))
import tizen_capi_utils

# Callback types
# void (*wifi_manager_scan_finished_cb)(wifi_manager_error_e error_code, void *user_data)
WIFI_SCAN_FINISHED_CB = ctypes.CFUNCTYPE(None, ctypes.c_int, ctypes.c_void_p)
# bool (*wifi_manager_found_ap_cb)(wifi_manager_ap_h ap, void *user_data)
WIFI_FOUND_AP_CB = ctypes.CFUNCTYPE(ctypes.c_bool, ctypes.c_void_p, ctypes.c_void_p)
# bool (*tizen_core_task_cb)(void *user_data)
TIZEN_CORE_TASK_CB = ctypes.CFUNCTYPE(ctypes.c_bool, ctypes.c_void_p)


def scan_wifi_networks():
    try:
        wifi_lib = tizen_capi_utils.load_library(
            ["libcapi-network-wifi-manager.so.1", "libcapi-network-wifi-manager.so"]
        )
        core_lib = tizen_capi_utils.load_library(
            ["libtizen-core.so.0", "libtizen-core.so"]
        )

        # --- Set up function signatures ---

        # wifi_manager_initialize(wifi_manager_h *wifi)
        wifi_lib.wifi_manager_initialize.argtypes = [ctypes.POINTER(ctypes.c_void_p)]
        wifi_lib.wifi_manager_initialize.restype = ctypes.c_int

        # wifi_manager_deinitialize(wifi_manager_h wifi)
        wifi_lib.wifi_manager_deinitialize.argtypes = [ctypes.c_void_p]
        wifi_lib.wifi_manager_deinitialize.restype = ctypes.c_int

        # wifi_manager_scan(wifi_manager_h wifi, wifi_manager_scan_finished_cb cb, void *user_data)
        wifi_lib.wifi_manager_scan.argtypes = [
            ctypes.c_void_p, WIFI_SCAN_FINISHED_CB, ctypes.c_void_p
        ]
        wifi_lib.wifi_manager_scan.restype = ctypes.c_int

        # wifi_manager_foreach_found_ap(wifi_manager_h wifi, wifi_manager_found_ap_cb cb, void *user_data)
        wifi_lib.wifi_manager_foreach_found_ap.argtypes = [
            ctypes.c_void_p, WIFI_FOUND_AP_CB, ctypes.c_void_p
        ]
        wifi_lib.wifi_manager_foreach_found_ap.restype = ctypes.c_int

        # wifi_manager_ap_get_essid(wifi_manager_ap_h ap, char **essid)
        wifi_lib.wifi_manager_ap_get_essid.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_char_p)
        ]
        wifi_lib.wifi_manager_ap_get_essid.restype = ctypes.c_int

        # wifi_manager_ap_get_rssi(wifi_manager_ap_h ap, int *rssi)
        wifi_lib.wifi_manager_ap_get_rssi.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_int)
        ]
        wifi_lib.wifi_manager_ap_get_rssi.restype = ctypes.c_int

        # wifi_manager_ap_get_frequency(wifi_manager_ap_h ap, int *frequency)
        wifi_lib.wifi_manager_ap_get_frequency.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_int)
        ]
        wifi_lib.wifi_manager_ap_get_frequency.restype = ctypes.c_int

        # wifi_manager_ap_get_security_type(wifi_manager_ap_h ap, wifi_manager_security_type_e *type)
        wifi_lib.wifi_manager_ap_get_security_type.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_int)
        ]
        wifi_lib.wifi_manager_ap_get_security_type.restype = ctypes.c_int

        # wifi_manager_ap_get_max_speed(wifi_manager_ap_h ap, int *max_speed)
        wifi_lib.wifi_manager_ap_get_max_speed.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_int)
        ]
        wifi_lib.wifi_manager_ap_get_max_speed.restype = ctypes.c_int

        # tizen_core APIs
        core_lib.tizen_core_init.argtypes = []
        core_lib.tizen_core_init.restype = None

        core_lib.tizen_core_shutdown.argtypes = []
        core_lib.tizen_core_shutdown.restype = None

        core_lib.tizen_core_task_create.argtypes = [
            ctypes.c_char_p, ctypes.c_bool, ctypes.POINTER(ctypes.c_void_p)
        ]
        core_lib.tizen_core_task_create.restype = ctypes.c_int

        core_lib.tizen_core_task_destroy.argtypes = [ctypes.c_void_p]
        core_lib.tizen_core_task_destroy.restype = ctypes.c_int

        core_lib.tizen_core_task_run.argtypes = [ctypes.c_void_p]
        core_lib.tizen_core_task_run.restype = ctypes.c_int

        core_lib.tizen_core_task_quit.argtypes = [ctypes.c_void_p]
        core_lib.tizen_core_task_quit.restype = ctypes.c_int

        core_lib.tizen_core_task_get_tizen_core.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_void_p)
        ]
        core_lib.tizen_core_task_get_tizen_core.restype = ctypes.c_int

        core_lib.tizen_core_add_idle_job.argtypes = [
            ctypes.c_void_p, TIZEN_CORE_TASK_CB, ctypes.c_void_p,
            ctypes.POINTER(ctypes.c_void_p)
        ]
        core_lib.tizen_core_add_idle_job.restype = ctypes.c_int

        core_lib.tizen_core_add_timer.argtypes = [
            ctypes.c_void_p, ctypes.c_uint, TIZEN_CORE_TASK_CB, ctypes.c_void_p,
            ctypes.POINTER(ctypes.c_void_p)
        ]
        core_lib.tizen_core_add_timer.restype = ctypes.c_int

        # --- State ---
        ap_list = []
        scan_error = [None]
        task_handle = ctypes.c_void_p()
        wifi_handle = ctypes.c_void_p()

        security_map = {
            0: "none", 1: "wep", 2: "wpa_psk", 3: "wpa2_psk",
            4: "eap", 5: "wpa_ftp", 6: "sae", 7: "owe",
            8: "dpp",
        }

        # --- Callbacks ---
        def on_ap_found(ap, user_data):
            """Callback for each found AP."""
            essid = ctypes.c_char_p()
            rssi = ctypes.c_int()
            freq = ctypes.c_int()
            sec_type = ctypes.c_int()
            max_speed = ctypes.c_int()

            wifi_lib.wifi_manager_ap_get_essid(ap, ctypes.byref(essid))
            wifi_lib.wifi_manager_ap_get_rssi(ap, ctypes.byref(rssi))
            wifi_lib.wifi_manager_ap_get_frequency(ap, ctypes.byref(freq))
            wifi_lib.wifi_manager_ap_get_security_type(ap, ctypes.byref(sec_type))
            wifi_lib.wifi_manager_ap_get_max_speed(ap, ctypes.byref(max_speed))

            ap_entry = {
                "ssid": essid.value.decode("utf-8") if essid.value else "(hidden)",
                "rssi_dbm": rssi.value,
                "frequency_mhz": freq.value,
                "security": security_map.get(sec_type.value, f"unknown({sec_type.value})"),
                "max_speed_mbps": max_speed.value,
            }
            # Determine band
            if freq.value > 5000:
                ap_entry["band"] = "5GHz"
            elif freq.value > 0:
                ap_entry["band"] = "2.4GHz"

            ap_list.append(ap_entry)
            return True  # continue

        def on_scan_finished(error_code, user_data):
            """Called when WiFi scan completes."""
            if error_code != 0:
                scan_error[0] = f"Scan failed with error: {error_code}"
            else:
                # Now iterate found APs
                ap_cb = WIFI_FOUND_AP_CB(on_ap_found)
                # Store ref to prevent GC
                on_scan_finished._ap_cb = ap_cb
                wifi_lib.wifi_manager_foreach_found_ap(wifi_handle, ap_cb, None)

            # Quit the event loop
            core_lib.tizen_core_task_quit(task_handle)

        def on_idle(user_data):
            """Idle callback - starts the WiFi scan."""
            scan_cb = WIFI_SCAN_FINISHED_CB(on_scan_finished)
            # Store ref to prevent GC
            on_idle._scan_cb = scan_cb

            ret = wifi_lib.wifi_manager_scan(wifi_handle, scan_cb, None)
            if ret != 0:
                scan_error[0] = f"wifi_manager_scan failed: {ret}"
                core_lib.tizen_core_task_quit(task_handle)
            return False  # one-shot idle

        def on_timeout(user_data):
            """Safety timeout - quit loop if scan takes too long."""
            if not ap_list and not scan_error[0]:
                scan_error[0] = "Scan timed out"
            core_lib.tizen_core_task_quit(task_handle)
            return False

        # --- Main execution ---

        # 1. Initialize tizen-core
        core_lib.tizen_core_init()

        # 2. Create task (main loop, NOT threaded)
        ret = core_lib.tizen_core_task_create(b"main", False, ctypes.byref(task_handle))
        if ret != 0:
            core_lib.tizen_core_shutdown()
            return {"error": f"tizen_core_task_create failed: {ret}"}

        # 3. Get core handle
        core_handle = ctypes.c_void_p()
        core_lib.tizen_core_task_get_tizen_core(task_handle, ctypes.byref(core_handle))

        # 4. Initialize WiFi manager
        ret = wifi_lib.wifi_manager_initialize(ctypes.byref(wifi_handle))
        if ret != 0:
            core_lib.tizen_core_task_destroy(task_handle)
            core_lib.tizen_core_shutdown()
            return {"error": f"wifi_manager_initialize failed: {ret}"}

        # 5. Add idle job to start scan
        idle_cb = TIZEN_CORE_TASK_CB(on_idle)
        idle_source = ctypes.c_void_p()
        core_lib.tizen_core_add_idle_job(core_handle, idle_cb, None, ctypes.byref(idle_source))

        # 6. Add 10-second safety timeout
        timeout_cb = TIZEN_CORE_TASK_CB(on_timeout)
        timer_source = ctypes.c_void_p()
        core_lib.tizen_core_add_timer(core_handle, 10000, timeout_cb, None, ctypes.byref(timer_source))

        # 7. Run the event loop (blocks until quit)
        core_lib.tizen_core_task_run(task_handle)

        # 8. Cleanup
        wifi_lib.wifi_manager_deinitialize(wifi_handle)
        core_lib.tizen_core_task_destroy(task_handle)
        core_lib.tizen_core_shutdown()

        # 9. Return results
        if scan_error[0]:
            return {"error": scan_error[0], "networks": ap_list}

        # Sort by signal strength
        ap_list.sort(key=lambda x: x.get("rssi_dbm", -100), reverse=True)

        return {
            "networks": ap_list,
            "count": len(ap_list),
            "async_engine": "tizen-core",
        }

    except Exception as e:
        return {"error": str(e)}


if __name__ == "__main__":
    print(json.dumps(scan_wifi_networks()))
