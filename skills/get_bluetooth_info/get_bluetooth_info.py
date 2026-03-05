#!/usr/bin/env python3
import ctypes
import json
import os
import sys

# Add common directory to path to import tizen_capi_utils
sys.path.append(os.path.join(os.path.dirname(__file__), "..", "common"))
import tizen_capi_utils

def get_bluetooth_info():
    try:
        lib = tizen_capi_utils.load_library(["libcapi-network-bluetooth.so.0", "libcapi-network-bluetooth.so.1"])
        
        # Define functions
        # int bt_initialize(void)
        lib.bt_initialize.argtypes = []
        
        # int bt_adapter_get_state(bt_adapter_state_e *state)
        lib.bt_adapter_get_state.argtypes = [ctypes.POINTER(ctypes.c_int)]
        
        # int bt_adapter_get_name(char **name)
        lib.bt_adapter_get_name.argtypes = [ctypes.POINTER(ctypes.c_char_p)]
        
        # int bt_adapter_get_address(char **address)
        lib.bt_adapter_get_address.argtypes = [ctypes.POINTER(ctypes.c_char_p)]
        
        # Check if bluetooth is supported
        info_lib = tizen_capi_utils.load_library(["libcapi-system-info.so.0", "libcapi-system-info.so.1"])
        system_info_get_platform_bool = info_lib.system_info_get_platform_bool
        system_info_get_platform_bool.argtypes = [ctypes.c_char_p, ctypes.POINTER(ctypes.c_bool)]
        
        supported = ctypes.c_bool()
        system_info_get_platform_bool(b"http://tizen.org/feature/network.bluetooth", ctypes.byref(supported))
        if not supported.value:
            return {"error": "Bluetooth is not supported on this device"}

        # Start calls
        tizen_capi_utils.check_return(lib.bt_initialize(), "Failed to initialize Bluetooth")
        
        state = ctypes.c_int()
        lib.bt_adapter_get_state(ctypes.byref(state))
        
        # bt_adapter_state_e mapping
        states = {0: "disabled", 1: "enabled"}
        
        result = {
            "activated": states.get(state.value, "disabled") == "enabled",
            "name": "",
            "address": ""
        }
        
        if result["activated"]:
            name_ptr = ctypes.c_char_p()
            if lib.bt_adapter_get_name(ctypes.byref(name_ptr)) == 0:
                result["name"] = tizen_capi_utils.decode_ptr(name_ptr)
            
            addr_ptr = ctypes.c_char_p()
            if lib.bt_adapter_get_address(ctypes.byref(addr_ptr)) == 0:
                result["address"] = tizen_capi_utils.decode_ptr(addr_ptr)
        
        # Clean up
        lib.bt_deinitialize()
        
        return result

    except Exception as e:
        return {"error": str(e)}

if __name__ == "__main__":
    print(json.dumps(get_bluetooth_info()))
