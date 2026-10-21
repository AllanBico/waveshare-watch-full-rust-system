#!/usr/bin/env python3
"""Rebuild a WAV from the firmware's `[pcm]` serial dump."""

import argparse
import re
import struct
import sys
import wave

BEGIN = re.compile(r"\[pcm\] begin frames=(\d+) channels=(\d+) rate=(\d+)")
END = "[pcm] end"


def parse(text):
    start = text.find("[pcm] begin")
    if start < 0:
        raise SystemExit("no '[pcm] begin' marker found")
    header = BEGIN.search(text, start)
    if header is None:
        raise SystemExit("malformed '[pcm] begin' marker")
    frames, channels, rate = (int(value) for value in header.groups())
    body = text[header.end():]
    end = body.find(END)
    if end >= 0:
        body = body[:end]

    digits = []
    for line in body.splitlines():
        line = line.strip()
        if not line or len(line) % 4 != 0:
            continue
        if all(c in "0123456789abcdefABCDEF" for c in line):
            digits.append(line)

    hex_data = "".join(digits)
    samples = [int(hex_data[i:i + 4], 16) for i in range(0, len(hex_data), 4)]
    samples = samples[: frames * channels]
    return frames, channels, rate, samples


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", nargs="?", help="serial log (default: stdin)")
    parser.add_argument("output", help="WAV file to write")
    args = parser.parse_args()

    if args.input:
        with open(args.input, "r", encoding="utf-8", errors="replace") as handle:
            text = handle.read()
    else:
        text = sys.stdin.read()

    frames, channels, rate, samples = parse(text)
    raw = b"".join(struct.pack("<H", sample) for sample in samples)

    with wave.open(args.output, "wb") as wav:
        wav.setnchannels(channels)
        wav.setsampwidth(2)
        wav.setframerate(rate)
        wav.writeframes(raw)

    print(
        f"wrote {args.output}: {min(frames, len(samples) // channels)} frames, "
        f"{channels} channels, {rate} Hz"
    )


if __name__ == "__main__":
    main()
