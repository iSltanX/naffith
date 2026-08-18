#!/usr/bin/env python3
"""نَفِّذ · minimal PNG crop/probe helper for the artwork build.

Headless Chromium's --screenshot can clip when the window is sized to the exact
artboard, so build.sh renders into a slightly larger window and crops back down
to the true frame size here. Pure stdlib (zlib only) — no Pillow dependency.

    pngtool.py crop  IN OUT W H      crop IN to W×H from the top-left
    pngtool.py probe IN X Y          print the RGB of one pixel
    pngtool.py size  IN              print WIDTHxHEIGHT
"""
import sys
import zlib
import struct


def _chunks(data):
    i = 8
    while i < len(data):
        (ln,) = struct.unpack('>I', data[i:i + 4])
        typ = data[i + 4:i + 8]
        yield typ, data[i + 8:i + 8 + ln]
        i += 8 + ln + 4


def read(path):
    """Return (width, height, rows) with rows as bytearrays of RGB triples."""
    raw = open(path, 'rb').read()
    if raw[:8] != b'\x89PNG\r\n\x1a\n':
        raise SystemExit('not a PNG: %s' % path)
    idat = b''
    w = h = depth = ctype = None
    for typ, body in _chunks(raw):
        if typ == b'IHDR':
            w, h, depth, ctype = struct.unpack('>IIBB', body[:10])
        elif typ == b'IDAT':
            idat += body
    if depth != 8 or ctype not in (2, 6):
        raise SystemExit('unsupported PNG (depth=%s colour=%s)' % (depth, ctype))
    nch = 3 if ctype == 2 else 4
    data = zlib.decompress(idat)
    stride = w * nch
    rows, prev, pos = [], bytearray(stride), 0
    for _ in range(h):
        f = data[pos]; pos += 1
        line = bytearray(data[pos:pos + stride]); pos += stride
        # undo the per-scanline filter
        if f == 1:
            for i in range(nch, stride):
                line[i] = (line[i] + line[i - nch]) & 255
        elif f == 2:
            for i in range(stride):
                line[i] = (line[i] + prev[i]) & 255
        elif f == 3:
            for i in range(stride):
                a = line[i - nch] if i >= nch else 0
                line[i] = (line[i] + ((a + prev[i]) >> 1)) & 255
        elif f == 4:
            for i in range(stride):
                a = line[i - nch] if i >= nch else 0
                b = prev[i]
                c = prev[i - nch] if i >= nch else 0
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                pr = a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
                line[i] = (line[i] + pr) & 255
        prev = line
        rows.append(bytes(line[i:i + 3] for i in range(0, stride, nch)) if nch == 4
                    else bytes(line))
    return w, h, rows


def write(path, w, h, rows):
    raw = b''.join(b'\x00' + r for r in rows)
    def chunk(typ, body):
        return (struct.pack('>I', len(body)) + typ + body
                + struct.pack('>I', zlib.crc32(typ + body) & 0xffffffff))
    out = (b'\x89PNG\r\n\x1a\n'
           + chunk(b'IHDR', struct.pack('>IIBBBBB', w, h, 8, 2, 0, 0, 0))
           + chunk(b'IDAT', zlib.compress(raw, 9))
           + chunk(b'IEND', b''))
    open(path, 'wb').write(out)


def main():
    cmd = sys.argv[1]
    if cmd == 'crop':
        src, dst, w, h = sys.argv[2], sys.argv[3], int(sys.argv[4]), int(sys.argv[5])
        sw, sh, rows = read(src)
        if sw < w or sh < h:
            raise SystemExit('source %dx%d smaller than crop %dx%d' % (sw, sh, w, h))
        write(dst, w, h, [r[:w * 3] for r in rows[:h]])
    elif cmd == 'probe':
        src, x, y = sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
        _, _, rows = read(src)
        px = rows[y][x * 3:x * 3 + 3]
        print('#%02X%02X%02X' % (px[0], px[1], px[2]))
    elif cmd == 'size':
        w, h, _ = read(sys.argv[2])
        print('%dx%d' % (w, h))
    else:
        raise SystemExit(__doc__)


if __name__ == '__main__':
    main()
