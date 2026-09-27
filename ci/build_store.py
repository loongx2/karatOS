#!/usr/bin/env python3
"""
karatOS store builder (Phase 4): packs provisioned module images into the
flash store image the kernel loads modules from.

Store layout (mirrors karatos-kapi):
  [0..32]    StoreHeader  { magic 'KSTO', version, board_id, entry_count,
                           total_size, crc32, reserved }
  [32..224]  directory: STORE_MAX_ENTRIES x StoreEntry (24 B each)
  [224..]    module images, each self-verified by the loader
             (ModuleHeader with its own size + CRC32)

The store header CRC32(IEEE/zlib) covers everything after the header.

Usage: build_store.py --board-id N -o out.bin module1.bin [module2.bin ...]
"""
import argparse
import struct
import zlib

STORE_MAGIC = 0x4B53_544F  # 'KSTO'
STORE_VERSION = 1
MAX_ENTRIES = 8
DIR_OFF = 24               # StoreHeader is 24 bytes (4+2+2+4+4+4+4)
ENTRY_SIZE = 24
IMG_BASE = DIR_OFF + MAX_ENTRIES * ENTRY_SIZE  # 216
FLAG_ENABLED = 1 << 0


def crc32(data: bytes) -> int:
    return zlib.crc32(data) & 0xFFFFFFFF


def main():
    ap = argparse.ArgumentParser(description="karatOS store builder")
    ap.add_argument("--board-id", type=lambda x: int(x, 0), default=0,
                    help="board identity (0 = unprovisioned wildcard)")
    ap.add_argument("-o", "--output", required=True)
    ap.add_argument("images", nargs="+", help="raw module images (*.bin)")
    args = ap.parse_args()

    if len(args.images) > MAX_ENTRIES:
        raise SystemExit(f"store holds at most {MAX_ENTRIES} modules")

    # payload[i] == store[DIR_OFF + i]: directory first, then images.
    payload = bytearray(IMG_BASE - DIR_OFF)  # directory placeholder (192 B)
    entries = []
    cur = IMG_BASE  # absolute store offset of the first image
    for path in args.images:
        with open(path, "rb") as f:
            img = f.read()
        if len(img) < 72:
            raise SystemExit(f"{path}: too small to be a module image")
        name = path.rsplit("/", 1)[-1].split("-")[0]  # hello-riscv...bin -> hello
        entries.append((name[:12], cur, len(img)))
        payload.extend(img)
        cur += len(img)
        while (cur - DIR_OFF) != len(payload):  # 4-byte align between images
            payload.append(0)
            cur += 1

    # fill directory entries (offsets are absolute store offsets)
    for i, (name, off, size) in enumerate(entries):
        base = i * ENTRY_SIZE
        payload[base : base + 12] = name.encode().ljust(12, b"\0")[:12]
        struct.pack_into("<I", payload, base + 12, off)
        struct.pack_into("<I", payload, base + 16, size)
        struct.pack_into("<I", payload, base + 20, FLAG_ENABLED)

    total = DIR_OFF + len(payload)
    crc = crc32(payload)  # over directory + images (== store[DIR_OFF:total])
    header = struct.pack(
        "<IHHIIII",
        STORE_MAGIC,
        STORE_VERSION,
        args.board_id & 0xFFFF,
        len(entries),
        total,
        crc,
        0,
    )
    store = bytearray(header)
    store.extend(payload)

    with open(args.output, "wb") as f:
        f.write(store)
    names = ", ".join(f"{n}@{o:#x}" for n, o, _ in entries)
    print(
        f"karatOS store: {args.output} ({len(store)} B, board {args.board_id:#06x}, "
        f"entries: {names}, crc32 {crc:#010x})"
    )


if __name__ == "__main__":
    main()
