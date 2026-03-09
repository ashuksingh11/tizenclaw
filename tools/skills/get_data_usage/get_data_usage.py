#!/usr/bin/env python3
import ctypes
import json
import os
import sys

sys.path.append(os.path.join(os.path.dirname(__file__), "..", "common"))
import tizen_capi_utils


def get_data_usage():
    try:
        lib = tizen_capi_utils.load_library(
            ["libcapi-network-connection.so.1", "libcapi-network-connection.so"]
        )

        # connection_h
        conn = ctypes.c_void_p()
        lib.connection_create.argtypes = [ctypes.POINTER(ctypes.c_void_p)]
        lib.connection_create.restype = ctypes.c_int

        ret = lib.connection_create(ctypes.byref(conn))
        if ret != 0:
            return {"error": f"connection_create failed: {ret}"}

        # int connection_get_statistics(connection_h, connection_statistics_type_e, long long *)
        lib.connection_get_statistics.argtypes = [
            ctypes.c_void_p, ctypes.c_int, ctypes.POINTER(ctypes.c_longlong)
        ]
        lib.connection_get_statistics.restype = ctypes.c_int

        # connection_statistics_type_e:
        # WIFI_LAST_RECEIVED_DATA  = 0
        # WIFI_LAST_SENT_DATA      = 1
        # WIFI_TOTAL_RECEIVED_DATA = 2
        # WIFI_TOTAL_SENT_DATA     = 3
        # CELLULAR_LAST_RECEIVED_DATA  = 4
        # CELLULAR_LAST_SENT_DATA      = 5
        # CELLULAR_TOTAL_RECEIVED_DATA = 6
        # CELLULAR_TOTAL_SENT_DATA     = 7

        result = {}

        # WiFi stats
        wifi = {}
        for stat_id, stat_name in [(0, "last_received"), (1, "last_sent"),
                                    (2, "total_received"), (3, "total_sent")]:
            val = ctypes.c_longlong()
            ret = lib.connection_get_statistics(conn, stat_id, ctypes.byref(val))
            if ret == 0:
                wifi[stat_name + "_bytes"] = val.value
        if wifi:
            total = wifi.get("total_received_bytes", 0) + wifi.get("total_sent_bytes", 0)
            wifi["total_bytes"] = total
            if total > 0:
                wifi["total_mb"] = round(total / (1024 * 1024), 2)
            result["wifi"] = wifi

        # Cellular stats
        cellular = {}
        for stat_id, stat_name in [(4, "last_received"), (5, "last_sent"),
                                    (6, "total_received"), (7, "total_sent")]:
            val = ctypes.c_longlong()
            ret = lib.connection_get_statistics(conn, stat_id, ctypes.byref(val))
            if ret == 0:
                cellular[stat_name + "_bytes"] = val.value
        if cellular:
            total = cellular.get("total_received_bytes", 0) + cellular.get("total_sent_bytes", 0)
            cellular["total_bytes"] = total
            if total > 0:
                cellular["total_mb"] = round(total / (1024 * 1024), 2)
            result["cellular"] = cellular

        lib.connection_destroy.argtypes = [ctypes.c_void_p]
        lib.connection_destroy.restype = ctypes.c_int
        lib.connection_destroy(conn)

        return {"data_usage": result}

    except Exception as e:
        return {"error": str(e)}


if __name__ == "__main__":
    print(json.dumps(get_data_usage()))
