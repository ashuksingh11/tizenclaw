#!/usr/bin/env python3
import ctypes
import json
import os
import sys

# Add common directory to path to import tizen_capi_utils
sys.path.append(os.path.join(os.path.dirname(__file__), "..", "common"))
import tizen_capi_utils

def get_installed_apps():
    try:
        lib = tizen_capi_utils.load_library(["libcapi-appfw-app-manager.so.0", "libcapi-appfw-app-manager.so.1"])
        
        # Define callback and function types
        # bool (*app_manager_app_info_cb)(app_info_h app_info, void *user_data)
        CMPFUNC = ctypes.CFUNCTYPE(ctypes.c_bool, ctypes.c_void_p, ctypes.c_void_p)
        
        lib.app_manager_foreach_app_info.argtypes = [CMPFUNC, ctypes.c_void_p]
        lib.app_info_get_app_id.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_char_p)]
        lib.app_info_get_label.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_char_p)]

        discovered_apps = []

        def app_info_cb(app_info, user_data):
            app_id_ptr = tizen_capi_utils.get_char_ptr()
            label_ptr = tizen_capi_utils.get_char_ptr()
            
            lib.app_info_get_app_id(app_info, ctypes.byref(app_id_ptr))
            lib.app_info_get_label(app_info, ctypes.byref(label_ptr))
            
            app_id = tizen_capi_utils.decode_ptr(app_id_ptr)
            label = tizen_capi_utils.decode_ptr(label_ptr)
            
            # Filter out common headless or hidden services
            ignored_keywords = ["service", "daemon", "system", "bootstrap", "setting", "engine"]
            is_hidden = any(kw in app_id.lower() or kw in label.lower() for kw in ignored_keywords)
            
            # Basic filtering to limit to UI apps that a user might want
            if not is_hidden and app_id.startswith("org.tizen") and len(discovered_apps) < 30:
                discovered_apps.append({"id": app_id, "label": label})
                
            return True

        cb = CMPFUNC(app_info_cb)
        tizen_capi_utils.check_return(lib.app_manager_foreach_app_info(cb, None), "app_manager_foreach_app_info failed")
            
        return {"apps": discovered_apps}

    except Exception as e:
        return {"error": str(e)}

if __name__ == "__main__":
    result = get_installed_apps()
    print(json.dumps(result))
