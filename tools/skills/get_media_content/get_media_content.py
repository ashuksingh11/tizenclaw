#!/usr/bin/env python3
import ctypes
import json
import os
import sys

sys.path.append(os.path.join(os.path.dirname(__file__), "..", "common"))
import tizen_capi_utils

# Callback type: bool (*media_info_cb)(media_info_h media, void *user_data)
MEDIA_INFO_CB = ctypes.CFUNCTYPE(ctypes.c_bool, ctypes.c_void_p, ctypes.c_void_p)


def get_media_content():
    try:
        args = json.loads(os.environ.get("CLAW_ARGS", "{}"))
        media_type = args.get("media_type", "all")
        max_count = int(args.get("max_count", 20))

        lib = tizen_capi_utils.load_library(
            ["libcapi-content-media-content.so.0", "libcapi-content-media-content.so"]
        )

        # int media_content_connect(void)
        lib.media_content_connect.argtypes = []
        lib.media_content_connect.restype = ctypes.c_int

        # int media_content_disconnect(void)
        lib.media_content_disconnect.argtypes = []
        lib.media_content_disconnect.restype = ctypes.c_int

        # int media_info_foreach_media_from_db(filter_h filter, media_info_cb cb, void *user_data)
        lib.media_info_foreach_media_from_db.argtypes = [
            ctypes.c_void_p, MEDIA_INFO_CB, ctypes.c_void_p
        ]
        lib.media_info_foreach_media_from_db.restype = ctypes.c_int

        # int media_info_get_display_name(media_info_h media, char **name)
        lib.media_info_get_display_name.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_char_p)
        ]
        lib.media_info_get_display_name.restype = ctypes.c_int

        # int media_info_get_file_path(media_info_h media, char **path)
        lib.media_info_get_file_path.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_char_p)
        ]
        lib.media_info_get_file_path.restype = ctypes.c_int

        # int media_info_get_media_type(media_info_h media, media_content_type_e *type)
        lib.media_info_get_media_type.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_int)
        ]
        lib.media_info_get_media_type.restype = ctypes.c_int

        # int media_info_get_size(media_info_h media, unsigned long long *size)
        lib.media_info_get_size.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_ulonglong)
        ]
        lib.media_info_get_size.restype = ctypes.c_int

        # int media_info_get_mime_type(media_info_h media, char **mime_type)
        lib.media_info_get_mime_type.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_char_p)
        ]
        lib.media_info_get_mime_type.restype = ctypes.c_int

        ret = lib.media_content_connect()
        if ret != 0:
            return {"error": f"media_content_connect failed: {ret}"}

        type_map = {0: "image", 1: "video", 2: "sound", 3: "music", 4: "other"}
        type_filter = {"image": 0, "video": 1, "sound": 2, "music": 3}

        media_list = []

        def media_cb(media, user_data):
            if len(media_list) >= max_count:
                return False

            name = ctypes.c_char_p()
            path = ctypes.c_char_p()
            mtype = ctypes.c_int()
            size = ctypes.c_ulonglong()
            mime = ctypes.c_char_p()

            lib.media_info_get_display_name(media, ctypes.byref(name))
            lib.media_info_get_file_path(media, ctypes.byref(path))
            lib.media_info_get_media_type(media, ctypes.byref(mtype))
            lib.media_info_get_size(media, ctypes.byref(size))
            lib.media_info_get_mime_type(media, ctypes.byref(mime))

            mtype_str = type_map.get(mtype.value, "other")

            if media_type != "all" and media_type in type_filter:
                if mtype.value != type_filter[media_type]:
                    return True  # skip, continue

            entry = {
                "name": name.value.decode("utf-8") if name.value else "",
                "path": path.value.decode("utf-8") if path.value else "",
                "type": mtype_str,
                "size_bytes": size.value,
                "mime_type": mime.value.decode("utf-8") if mime.value else "",
            }
            media_list.append(entry)
            return len(media_list) < max_count

        cb = MEDIA_INFO_CB(media_cb)
        lib.media_info_foreach_media_from_db(None, cb, None)

        lib.media_content_disconnect()

        return {
            "media_files": media_list,
            "count": len(media_list),
            "filter": media_type,
        }

    except Exception as e:
        return {"error": str(e)}


if __name__ == "__main__":
    print(json.dumps(get_media_content()))
