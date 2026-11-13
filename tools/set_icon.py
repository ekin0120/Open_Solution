#!/usr/bin/env python3
"""Inject the KARDOX application icon into a Windows PE executable.

Uses the built-in UpdateResource API (same mechanism as rcedit) via ctypes,
so no external toolchain (rc.exe / windres) is required. Run AFTER
`cargo build --release`:

    python tools/set_icon.py target/release/os-gui.exe
"""
import ctypes
import struct
import sys
import os

RT_ICON = 3
RT_GROUP_ICON = 14
LANG_EN_US = 0x0409

kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
# 64-bit HANDLE: must declare restype/argtypes explicitly or the handle truncates
kernel32.BeginUpdateResourceW.argtypes = [ctypes.c_wchar_p, ctypes.c_bool]
kernel32.BeginUpdateResourceW.restype = ctypes.c_void_p
kernel32.UpdateResourceW.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p,
                                     ctypes.c_ushort, ctypes.c_void_p, ctypes.c_uint]
kernel32.UpdateResourceW.restype = ctypes.c_bool
kernel32.EndUpdateResourceW.argtypes = [ctypes.c_void_p, ctypes.c_bool]
kernel32.EndUpdateResourceW.restype = ctypes.c_bool

def update_resource(h, type_id, name_id, data):
    buf = ctypes.create_string_buffer(data, len(data))
    if not kernel32.UpdateResourceW(h, type_id, name_id, LANG_EN_US, buf, len(data)):
        raise OSError(f"UpdateResourceW({type_id},{name_id}) failed: "
                      f"{ctypes.FormatError(ctypes.get_last_error())}")
    # buf must stay alive until EndUpdateResource; keep reference
    update_resource._keep.append(buf)

update_resource._keep = []

def parse_ico(path):
    blob = open(path, "rb").read()
    _reserved, _type, count = struct.unpack_from("<HHH", blob, 0)  # ICONDIR
    images = []
    for i in range(count):
        w, h, cc, _r, planes, bpp, size, off = struct.unpack_from("<BBBBHHII", blob, 6 + 16 * i)
        images.append((w, h, cc, planes, bpp, blob[off:off + size]))
    return images

def main():
    exe = sys.argv[1] if len(sys.argv) > 1 else "target/release/os-gui.exe"
    ico = os.path.join(os.path.dirname(__file__), "..", "crates", "os-gui", "assets", "icon.ico")
    if not os.path.exists(exe):
        sys.exit(f"exe not found: {exe}")
    if not os.path.exists(ico):
        sys.exit(f"icon not found: {ico}")

    # Delete-all + rewrite: incremental updates on an exe that already has
    # icons fail with ERROR_INTERNAL_ERROR; full rewrite is the reliable path.
    h = kernel32.BeginUpdateResourceW(ctypes.c_wchar_p(os.path.abspath(exe)), True)
    if not h:
        raise OSError(f"BeginUpdateResourceW failed (exe locked?): "
                      f"{ctypes.FormatError(ctypes.get_last_error())}")
    try:
        images = parse_ico(ico)
        for i, (w, hgt, cc, planes, bpp, data) in enumerate(images, 1):
            update_resource(h, RT_ICON, i, data)
        # GRPICONDIR + GRPICONDIRENTRYs (wType: 1 = icon, NOT the RT_GROUP_ICON id)
        grp = struct.pack("<HHH", 0, 1, len(images))
        for i, (w, hgt, cc, planes, bpp, data) in enumerate(images, 1):
            grp += struct.pack("<BBBBHHIH", w & 0xFF, hgt & 0xFF, cc, 0, planes, bpp, len(data), i)
        update_resource(h, RT_GROUP_ICON, 1, grp)
        print(f"injected: {len(images)} icon images ({', '.join(str(w or 256) for w, *_ in images)}) -> {exe}")
    except Exception:
        kernel32.EndUpdateResourceW(h, True)  # discard on failure
        raise
    if not kernel32.EndUpdateResourceW(h, False):
        raise OSError(f"EndUpdateResourceW failed: {ctypes.FormatError(ctypes.get_last_error())}")
    print("icon committed OK")

if __name__ == "__main__":
    main()
