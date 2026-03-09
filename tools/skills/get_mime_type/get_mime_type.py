#!/usr/bin/env python3
import ctypes
import json
import os
import sys

sys.path.append(os.path.join(os.path.dirname(__file__), "..", "common"))
import tizen_capi_utils


def get_mime_type():
    try:
        args = json.loads(os.environ.get("CLAW_ARGS", "{}"))
        file_ext = args.get("file_extension", "")
        file_path = args.get("file_path", "")
        mime_type_input = args.get("mime_type", "")

        lib = tizen_capi_utils.load_library(
            ["libcapi-content-mime-type.so.0", "libcapi-content-mime-type.so"]
        )

        result = {}

        if file_ext:
            # int mime_type_get_mime_type(const char *file_extension, char **mime_type)
            lib.mime_type_get_mime_type.argtypes = [
                ctypes.c_char_p, ctypes.POINTER(ctypes.c_char_p)
            ]
            lib.mime_type_get_mime_type.restype = ctypes.c_int

            ext = file_ext.lstrip(".")
            mime_out = ctypes.c_char_p()
            ret = lib.mime_type_get_mime_type(
                ext.encode("utf-8"), ctypes.byref(mime_out)
            )
            if ret == 0 and mime_out.value:
                result["extension"] = ext
                result["mime_type"] = mime_out.value.decode("utf-8")
            else:
                result["error_extension"] = f"No MIME type found for .{ext}"

        if file_path:
            # int mime_type_get_mime_type_for_file(const char *file_path, char **mime_type)
            lib.mime_type_get_mime_type_for_file.argtypes = [
                ctypes.c_char_p, ctypes.POINTER(ctypes.c_char_p)
            ]
            lib.mime_type_get_mime_type_for_file.restype = ctypes.c_int

            mime_out = ctypes.c_char_p()
            ret = lib.mime_type_get_mime_type_for_file(
                file_path.encode("utf-8"), ctypes.byref(mime_out)
            )
            if ret == 0 and mime_out.value:
                result["file_path"] = file_path
                result["file_mime_type"] = mime_out.value.decode("utf-8")
            else:
                result["error_file"] = f"Cannot detect MIME type for {file_path}"

        if mime_type_input:
            # int mime_type_get_file_extension(const char *mime_type, char ***file_extension, int *length)
            lib.mime_type_get_file_extension.argtypes = [
                ctypes.c_char_p,
                ctypes.POINTER(ctypes.POINTER(ctypes.c_char_p)),
                ctypes.POINTER(ctypes.c_int),
            ]
            lib.mime_type_get_file_extension.restype = ctypes.c_int

            ext_arr = ctypes.POINTER(ctypes.c_char_p)()
            length = ctypes.c_int()
            ret = lib.mime_type_get_file_extension(
                mime_type_input.encode("utf-8"),
                ctypes.byref(ext_arr),
                ctypes.byref(length),
            )
            if ret == 0 and length.value > 0:
                extensions = []
                for i in range(length.value):
                    if ext_arr[i]:
                        extensions.append(ext_arr[i].decode("utf-8"))
                result["requested_mime"] = mime_type_input
                result["extensions"] = extensions
            else:
                result["error_mime"] = f"No extensions found for {mime_type_input}"

        if not file_ext and not file_path and not mime_type_input:
            return {"error": "Provide at least one of: file_extension, file_path, or mime_type"}

        return result

    except Exception as e:
        return {"error": str(e)}


if __name__ == "__main__":
    print(json.dumps(get_mime_type()))
