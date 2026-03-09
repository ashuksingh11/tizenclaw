#!/usr/bin/env python3
import ctypes
import json
import os
import sys

sys.path.append(os.path.join(os.path.dirname(__file__), "..", "common"))
import tizen_capi_utils


def get_sound_devices():
    try:
        lib = tizen_capi_utils.load_library(
            ["libcapi-media-sound-manager.so.0", "libcapi-media-sound-manager.so"]
        )

        # sound_device_mask_e: ALL = 0xFFFF
        SOUND_DEVICE_ALL_MASK = 0xFFFF

        # int sound_manager_get_device_list(int device_mask, sound_device_list_h *list)
        lib.sound_manager_get_device_list.argtypes = [
            ctypes.c_int, ctypes.POINTER(ctypes.c_void_p)
        ]
        lib.sound_manager_get_device_list.restype = ctypes.c_int

        # int sound_manager_get_next_device(sound_device_list_h list, sound_device_h *device)
        lib.sound_manager_get_next_device.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_void_p)
        ]
        lib.sound_manager_get_next_device.restype = ctypes.c_int

        # int sound_manager_get_device_type(sound_device_h device, sound_device_type_e *type)
        lib.sound_manager_get_device_type.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_int)
        ]
        lib.sound_manager_get_device_type.restype = ctypes.c_int

        # int sound_manager_get_device_io_direction(sound_device_h device, sound_device_io_direction_e *direction)
        lib.sound_manager_get_device_io_direction.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_int)
        ]
        lib.sound_manager_get_device_io_direction.restype = ctypes.c_int

        # int sound_manager_get_device_name(sound_device_h device, char **name)
        lib.sound_manager_get_device_name.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_char_p)
        ]
        lib.sound_manager_get_device_name.restype = ctypes.c_int

        # int sound_manager_get_device_id(sound_device_h device, int *id)
        lib.sound_manager_get_device_id.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_int)
        ]
        lib.sound_manager_get_device_id.restype = ctypes.c_int

        device_list = ctypes.c_void_p()
        ret = lib.sound_manager_get_device_list(SOUND_DEVICE_ALL_MASK, ctypes.byref(device_list))
        if ret != 0:
            return {"error": f"sound_manager_get_device_list failed: {ret}"}

        type_map = {
            0: "builtin_speaker", 1: "builtin_receiver", 2: "builtin_mic",
            3: "audio_jack", 4: "bluetooth_media", 5: "hdmi",
            6: "forwarding", 7: "usb_audio", 8: "bluetooth_voice",
        }
        dir_map = {0: "input", 1: "output", 2: "both"}

        devices = []
        while True:
            device = ctypes.c_void_p()
            ret = lib.sound_manager_get_next_device(device_list, ctypes.byref(device))
            if ret != 0:
                break

            dev_type = ctypes.c_int()
            direction = ctypes.c_int()
            name = ctypes.c_char_p()
            dev_id = ctypes.c_int()

            lib.sound_manager_get_device_type(device, ctypes.byref(dev_type))
            lib.sound_manager_get_device_io_direction(device, ctypes.byref(direction))
            lib.sound_manager_get_device_name(device, ctypes.byref(name))
            lib.sound_manager_get_device_id(device, ctypes.byref(dev_id))

            devices.append({
                "id": dev_id.value,
                "type": type_map.get(dev_type.value, f"unknown({dev_type.value})"),
                "name": name.value.decode("utf-8") if name.value else "unknown",
                "direction": dir_map.get(direction.value, f"unknown({direction.value})"),
            })

        lib.sound_manager_free_device_list.argtypes = [ctypes.c_void_p]
        lib.sound_manager_free_device_list.restype = ctypes.c_int
        lib.sound_manager_free_device_list(device_list)

        return {"devices": devices, "count": len(devices)}

    except Exception as e:
        return {"error": str(e)}


if __name__ == "__main__":
    print(json.dumps(get_sound_devices()))
