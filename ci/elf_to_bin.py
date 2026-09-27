#!/usr/bin/env python3
"""
karatOS module post-processor: ELF -> raw loadable image.

1. Extracts all PT_LOAD segments at their physical addresses (the module is
   linked for its fixed RAM slot, so p_paddr IS the image offset).
2. Prepends/patches the 32-byte karatOS ModuleHeader: image_size at [8..12],
   CRC32(IEEE/zlib) over image[32..] at [16..20].

No external dependencies — pure stdlib, works in slim CI containers.

Usage: elf_to_bin.py <module.elf> <output.bin>
"""
import struct
import sys
import zlib

PT_LOAD = 1


def parse_elf(data: bytes):
    if data[:4] != b"\x7fELF":
        raise ValueError("not an ELF file")
    is64 = data[4] == 2
    little = data[5] == 1
    if not little:
        raise ValueError("only little-endian ELFs supported")
    if is64:
        e_phoff = struct.unpack_from("<Q", data, 0x20)[0]
        e_phentsize = struct.unpack_from("<H", data, 0x36)[0]
        e_phnum = struct.unpack_from("<H", data, 0x38)[0]
    else:
        e_phoff = struct.unpack_from("<I", data, 0x1C)[0]
        e_phentsize = struct.unpack_from("<H", data, 0x2A)[0]
        e_phnum = struct.unpack_from("<H", data, 0x2C)[0]

    loads = []
    for i in range(e_phnum):
        base = e_phoff + i * e_phentsize
        if is64:
            p_type, _p_flags = struct.unpack_from("<II", data, base)
            p_offset, p_vaddr, p_paddr = struct.unpack_from("<QQQ", data, base + 8)
            p_filesz = struct.unpack_from("<Q", data, base + 32)[0]
        else:
            p_type = struct.unpack_from("<I", data, base)[0]
            p_offset, p_vaddr, p_paddr = struct.unpack_from("<III", data, base + 4)
            p_filesz = struct.unpack_from("<I", data, base + 16)[0]
        if p_type == PT_LOAD and p_filesz > 0:
            loads.append((p_paddr, data[p_offset : p_offset + p_filesz]))
    if not loads:
        raise ValueError("no PT_LOAD segments")
    return loads


def main():
    if len(sys.argv) != 3:
        print(f"usage: {sys.argv[0]} <module.elf> <output.bin>", file=sys.stderr)
        return 2
    elf_path, bin_path = sys.argv[1], sys.argv[2]
    with open(elf_path, "rb") as f:
        data = f.read()

    loads = parse_elf(data)
    base = min(p for p, _ in loads)
    end = max(p + len(seg) for p, seg in loads)
    image = bytearray(end - base)
    for paddr, seg in loads:
        image[paddr - base : paddr - base + len(seg)] = seg

    if len(image) < 72:
        raise ValueError(f"image too small ({len(image)} B) — build broken?")

    # Patch ModuleHeader: image_size at [8..12], CRC32 over [32..] at [16..20].
    struct.pack_into("<I", image, 8, len(image))
    crc = zlib.crc32(bytes(image[32:])) & 0xFFFFFFFF
    struct.pack_into("<I", image, 16, crc)

    with open(bin_path, "wb") as f:
        f.write(image)
    print(
        f"karatOS module: {bin_path} ({len(image)} B, base {base:#x}, "
        f"crc32 {crc:#010x})"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
