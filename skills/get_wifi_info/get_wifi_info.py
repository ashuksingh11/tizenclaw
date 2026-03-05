#!/usr/bin/env python3
import ctypes
import json
import os
import sys

# Add common directory to path to import tizen_capi_utils
sys.path.append(os.path.join(os.path.dirname(__file__), "..", "common"))
import tizen_capi_utils

def get_wifi_info():
    try:
        lib = tizen_capi_utils.load_library(["libcapi-network-wifi-manager.so.1", "libcapi-network-wifi-manager.so.0"])
        
        # Define functions
        # int wifi_manager_initialize(wifi_manager_h *wifi)
        wifi_manager_initialize = lib.wifi_manager_initialize
        wifi_manager_initialize.argtypes = [ctypes.POINTER(ctypes.c_void_p)]
        
        # int wifi_manager_is_activated(wifi_manager_h wifi, bool *activated)
        wifi_manager_is_activated = lib.wifi_manager_is_activated
        wifi_manager_is_activated.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_bool)]
        
        # int wifi_manager_get_connection_state(wifi_manager_h wifi, wifi_connection_state_e *state)
        wifi_manager_get_connection_state = lib.wifi_manager_get_connection_state
        wifi_manager_get_connection_state.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_int)]
        
        # int wifi_manager_get_connected_ap(wifi_manager_h wifi, wifi_ap_h *ap)
        wifi_manager_get_connected_ap = lib.wifi_manager_get_connected_ap
        wifi_manager_get_connected_ap.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_void_p)]
        
        # int wifi_manager_ap_get_essid(wifi_ap_h ap, char **essid)
        wifi_manager_ap_get_essid = lib.wifi_manager_ap_get_essid
        wifi_manager_ap_get_essid.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_char_p)]
        
        # Start calls
        handle = ctypes.c_void_p()
        tizen_capi_utils.check_return(wifi_manager_initialize(ctypes.byref(handle)), "Failed to initialize Wi-Fi Manager")
        
        activated = ctypes.c_bool()
        wifi_manager_is_activated(handle, ctypes.byref(activated))
        
        result = {
            "activated": activated.value,
            "connection_state": "unknown",
            "essid": ""
        }
        
        if activated.value:
            conn_state = ctypes.c_int()
            wifi_manager_get_connection_state(handle, ctypes.byref(conn_state))
            
            # wifi_connection_state_e mapping
            states = {0: "disconnected", 1: "association", 2: "configuration", 3: "connected", 4: "failure"}
            result["connection_state"] = states.get(conn_state.value, "unknown")
            
            if conn_state.value == 3: # connected
                ap_handle = ctypes.c_void_p()
                if wifi_manager_get_connected_ap(handle, ctypes.byref(ap_handle)) == 0:
                    essid_ptr = tizen_capi_utils.get_char_ptr()
                    if lib.wifi_manager_ap_get_essid(ap_handle, ctypes.byref(essid_ptr)) == 0:
                        result["essid"] = tizen_capi_utils.decode_ptr(essid_ptr)
        
        # Clean up
        lib.wifi_manager_deinitialize(handle)
        
        return result

    except Exception as e:
        return {"error": str(e)}

if __name__ == "__main__":
    print(json.dumps(get_wifi_info()))
