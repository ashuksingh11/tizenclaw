#!/usr/bin/env python3
"""
download_file - Async file download using tizen-core event loop.

Pattern:
1. tizen_core_init() -> task_create("main", false)
2. download_create() -> download_set_url/destination/callback
3. download_start() in idle job
4. state_changed_cb waits for COMPLETED/FAILED -> task_quit()
5. Return download result
"""
import ctypes
import json
import os
import sys

sys.path.append(os.path.join(os.path.dirname(__file__), "..", "common"))
import tizen_capi_utils

# Callback types
# void (*download_state_changed_cb)(int download_id, download_state_e state, void *user_data)
DOWNLOAD_STATE_CB = ctypes.CFUNCTYPE(None, ctypes.c_int, ctypes.c_int, ctypes.c_void_p)
# void (*download_progress_cb)(int download_id, unsigned long long received, void *user_data)
DOWNLOAD_PROGRESS_CB = ctypes.CFUNCTYPE(None, ctypes.c_int, ctypes.c_ulonglong, ctypes.c_void_p)
# bool (*tizen_core_task_cb)(void *user_data)
TIZEN_CORE_TASK_CB = ctypes.CFUNCTYPE(ctypes.c_bool, ctypes.c_void_p)


def download_file():
    try:
        args = json.loads(os.environ.get("CLAW_ARGS", "{}"))
        url = args.get("url", "")
        destination = args.get("destination", "/tmp")
        file_name = args.get("file_name", "")

        if not url:
            return {"error": "url is required"}

        dl_lib = tizen_capi_utils.load_library(
            ["libcapi-web-url-download.so.0", "libcapi-web-url-download.so"]
        )
        core_lib = tizen_capi_utils.load_library(
            ["libtizen-core.so.0", "libtizen-core.so"]
        )

        # --- Download API signatures ---
        dl_lib.download_create.argtypes = [ctypes.POINTER(ctypes.c_int)]
        dl_lib.download_create.restype = ctypes.c_int

        dl_lib.download_destroy.argtypes = [ctypes.c_int]
        dl_lib.download_destroy.restype = ctypes.c_int

        dl_lib.download_set_url.argtypes = [ctypes.c_int, ctypes.c_char_p]
        dl_lib.download_set_url.restype = ctypes.c_int

        dl_lib.download_set_destination.argtypes = [ctypes.c_int, ctypes.c_char_p]
        dl_lib.download_set_destination.restype = ctypes.c_int

        dl_lib.download_set_file_name.argtypes = [ctypes.c_int, ctypes.c_char_p]
        dl_lib.download_set_file_name.restype = ctypes.c_int

        dl_lib.download_set_state_changed_cb.argtypes = [ctypes.c_int, DOWNLOAD_STATE_CB, ctypes.c_void_p]
        dl_lib.download_set_state_changed_cb.restype = ctypes.c_int

        dl_lib.download_set_progress_cb.argtypes = [ctypes.c_int, DOWNLOAD_PROGRESS_CB, ctypes.c_void_p]
        dl_lib.download_set_progress_cb.restype = ctypes.c_int

        dl_lib.download_start.argtypes = [ctypes.c_int]
        dl_lib.download_start.restype = ctypes.c_int

        dl_lib.download_cancel.argtypes = [ctypes.c_int]
        dl_lib.download_cancel.restype = ctypes.c_int

        dl_lib.download_get_downloaded_file_path.argtypes = [ctypes.c_int, ctypes.POINTER(ctypes.c_char_p)]
        dl_lib.download_get_downloaded_file_path.restype = ctypes.c_int

        dl_lib.download_get_content_name.argtypes = [ctypes.c_int, ctypes.POINTER(ctypes.c_char_p)]
        dl_lib.download_get_content_name.restype = ctypes.c_int

        dl_lib.download_get_content_size.argtypes = [ctypes.c_int, ctypes.POINTER(ctypes.c_ulonglong)]
        dl_lib.download_get_content_size.restype = ctypes.c_int

        dl_lib.download_get_mime_type.argtypes = [ctypes.c_int, ctypes.POINTER(ctypes.c_char_p)]
        dl_lib.download_get_mime_type.restype = ctypes.c_int

        # --- tizen-core API signatures ---
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

        # --- State ---
        task_handle = ctypes.c_void_p()
        dl_id = ctypes.c_int()
        result = {"url": url, "destination": destination}
        last_received = [0]

        # download_state_e: 0=NONE, 1=READY, 2=QUEUED, 3=DOWNLOADING, 4=PAUSED, 5=COMPLETED, 6=FAILED, 7=CANCELED
        state_names = {0: "none", 1: "ready", 2: "queued", 3: "downloading", 4: "paused", 5: "completed", 6: "failed", 7: "canceled"}

        def on_state_changed(download_id, state, user_data):
            """Called when download state changes."""
            if state == 5:  # COMPLETED
                # Get downloaded file path
                path = ctypes.c_char_p()
                dl_lib.download_get_downloaded_file_path(download_id, ctypes.byref(path))
                if path.value:
                    result["file_path"] = path.value.decode("utf-8")

                # Get content size
                size = ctypes.c_ulonglong()
                dl_lib.download_get_content_size(download_id, ctypes.byref(size))
                result["content_size"] = size.value

                # Get MIME type
                mime = ctypes.c_char_p()
                dl_lib.download_get_mime_type(download_id, ctypes.byref(mime))
                if mime.value:
                    result["mime_type"] = mime.value.decode("utf-8")

                result["status"] = "completed"
                core_lib.tizen_core_task_quit(task_handle)

            elif state == 6:  # FAILED
                result["status"] = "failed"
                result["error"] = "Download failed"
                core_lib.tizen_core_task_quit(task_handle)

            elif state == 7:  # CANCELED
                result["status"] = "canceled"
                core_lib.tizen_core_task_quit(task_handle)

        def on_progress(download_id, received, user_data):
            last_received[0] = received

        def on_idle(user_data):
            """Start download in idle callback."""
            ret = dl_lib.download_start(dl_id.value)
            if ret != 0:
                result["error"] = f"download_start failed: {ret}"
                result["status"] = "failed"
                core_lib.tizen_core_task_quit(task_handle)
            return False

        def on_timeout(user_data):
            """60-second safety timeout."""
            if "status" not in result:
                result["status"] = "timeout"
                result["error"] = "Download timed out (60s)"
                result["bytes_received"] = last_received[0]
                dl_lib.download_cancel(dl_id.value)
            core_lib.tizen_core_task_quit(task_handle)
            return False

        # --- Main execution ---
        core_lib.tizen_core_init()

        ret = core_lib.tizen_core_task_create(b"main", False, ctypes.byref(task_handle))
        if ret != 0:
            core_lib.tizen_core_shutdown()
            return {"error": f"tizen_core_task_create failed: {ret}"}

        core_handle = ctypes.c_void_p()
        core_lib.tizen_core_task_get_tizen_core(task_handle, ctypes.byref(core_handle))

        # Create download
        ret = dl_lib.download_create(ctypes.byref(dl_id))
        if ret != 0:
            core_lib.tizen_core_task_destroy(task_handle)
            core_lib.tizen_core_shutdown()
            return {"error": f"download_create failed: {ret}"}

        dl_lib.download_set_url(dl_id, url.encode("utf-8"))
        dl_lib.download_set_destination(dl_id, destination.encode("utf-8"))
        if file_name:
            dl_lib.download_set_file_name(dl_id, file_name.encode("utf-8"))

        # Set callbacks (prevent GC)
        state_cb = DOWNLOAD_STATE_CB(on_state_changed)
        progress_cb = DOWNLOAD_PROGRESS_CB(on_progress)
        dl_lib.download_set_state_changed_cb(dl_id, state_cb, None)
        dl_lib.download_set_progress_cb(dl_id, progress_cb, None)

        # Add idle job and timeout
        idle_cb = TIZEN_CORE_TASK_CB(on_idle)
        idle_source = ctypes.c_void_p()
        core_lib.tizen_core_add_idle_job(core_handle, idle_cb, None, ctypes.byref(idle_source))

        timeout_cb = TIZEN_CORE_TASK_CB(on_timeout)
        timer_source = ctypes.c_void_p()
        core_lib.tizen_core_add_timer(core_handle, 60000, timeout_cb, None, ctypes.byref(timer_source))

        # Run event loop
        core_lib.tizen_core_task_run(task_handle)

        # Cleanup
        dl_lib.download_destroy(dl_id)
        core_lib.tizen_core_task_destroy(task_handle)
        core_lib.tizen_core_shutdown()

        return result

    except Exception as e:
        return {"error": str(e)}


if __name__ == "__main__":
    print(json.dumps(download_file()))
