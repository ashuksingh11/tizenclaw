#!/usr/bin/env python3
import ctypes
import json
import os
import sys

sys.path.append(os.path.join(os.path.dirname(__file__), "..", "common"))
import tizen_capi_utils


# metadata_extractor_attr_e enum values
METADATA_ATTRS = {
    0: "duration",
    1: "video_bitrate",
    2: "video_fps",
    3: "video_width",
    4: "video_height",
    5: "has_video",
    6: "audio_bitrate",
    7: "audio_channels",
    8: "audio_samplerate",
    9: "has_audio",
    10: "artist",
    11: "title",
    12: "album",
    13: "album_artist",
    14: "genre",
    15: "author",  # composer
    16: "copyright",
    17: "date",  # release date
    18: "description",
    19: "comment",
    20: "track_num",
    21: "classification",
    22: "rating",
    23: "longitude",
    24: "latitude",
    25: "altitude",
    26: "conductor",
    27: "unsynclyrics",
    28: "synclyrics_num",
    29: "recdate",
    30: "rotate",
    31: "content_id",  # video codec
    32: "audio_codec",
    33: "video_codec",
    34: "360",
}


def get_metadata():
    try:
        args = json.loads(os.environ.get("CLAW_ARGS", "{}"))
        file_path = args.get("file_path", "")

        if not file_path:
            return {"error": "file_path is required"}

        if not os.path.exists(file_path):
            return {"error": f"File not found: {file_path}"}

        lib = tizen_capi_utils.load_library(
            ["libcapi-media-metadata-extractor.so", "libcapi-media-metadata-extractor.so.0"]
        )

        # int metadata_extractor_create(metadata_extractor_h *metadata)
        lib.metadata_extractor_create.argtypes = [ctypes.POINTER(ctypes.c_void_p)]
        lib.metadata_extractor_create.restype = ctypes.c_int

        # int metadata_extractor_set_path(metadata_extractor_h metadata, const char *path)
        lib.metadata_extractor_set_path.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
        lib.metadata_extractor_set_path.restype = ctypes.c_int

        # int metadata_extractor_get_metadata(metadata_extractor_h metadata, metadata_extractor_attr_e attr, char **value)
        lib.metadata_extractor_get_metadata.argtypes = [
            ctypes.c_void_p, ctypes.c_int, ctypes.POINTER(ctypes.c_char_p)
        ]
        lib.metadata_extractor_get_metadata.restype = ctypes.c_int

        # int metadata_extractor_destroy(metadata_extractor_h metadata)
        lib.metadata_extractor_destroy.argtypes = [ctypes.c_void_p]
        lib.metadata_extractor_destroy.restype = ctypes.c_int

        handle = ctypes.c_void_p()
        ret = lib.metadata_extractor_create(ctypes.byref(handle))
        if ret != 0:
            return {"error": f"metadata_extractor_create failed: {ret}"}

        ret = lib.metadata_extractor_set_path(handle, file_path.encode("utf-8"))
        if ret != 0:
            lib.metadata_extractor_destroy(handle)
            return {"error": f"metadata_extractor_set_path failed: {ret}"}

        metadata = {"file_path": file_path}

        for attr_id, attr_name in METADATA_ATTRS.items():
            value = ctypes.c_char_p()
            ret = lib.metadata_extractor_get_metadata(handle, attr_id, ctypes.byref(value))
            if ret == 0 and value.value:
                val_str = value.value.decode("utf-8", errors="replace")
                if val_str and val_str.strip():
                    # Try to convert numeric
                    if attr_name in ("duration", "video_bitrate", "audio_bitrate",
                                     "video_fps", "video_width", "video_height",
                                     "audio_channels", "audio_samplerate",
                                     "track_num", "synclyrics_num", "rotate"):
                        try:
                            metadata[attr_name] = int(val_str)
                        except ValueError:
                            metadata[attr_name] = val_str
                    elif attr_name in ("has_video", "has_audio"):
                        metadata[attr_name] = val_str == "1"
                    elif attr_name in ("longitude", "latitude", "altitude"):
                        try:
                            metadata[attr_name] = float(val_str)
                        except ValueError:
                            metadata[attr_name] = val_str
                    else:
                        metadata[attr_name] = val_str

        # Convert duration from ms to readable
        if "duration" in metadata and isinstance(metadata["duration"], int):
            ms = metadata["duration"]
            s = ms // 1000
            metadata["duration_formatted"] = f"{s // 60}:{s % 60:02d}"

        lib.metadata_extractor_destroy(handle)

        return metadata

    except Exception as e:
        return {"error": str(e)}


if __name__ == "__main__":
    print(json.dumps(get_metadata()))
