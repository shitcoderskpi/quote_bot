import unicodedata

ranges = [
    (0x1F000, 0x1FFFF),  # Main emoji block
    (0x2600, 0x27BF),  # Misc symbols, dingbats
    (0x2300, 0x23FF),  # Misc technical
    (0x2B00, 0x2BFF),  # Misc symbols and arrows
    (0xFE00, 0xFE0F),  # Variation selectors
]

emojis = []
for start, end in ranges:
    for cp in range(start, end + 1):
        try:
            ch = chr(cp)
            name = unicodedata.name(ch, "")
            if name:
                emojis.append(ch)
        except Exception:
            pass

result = "".join(emojis)
print(f"Total emoji-range characters: {len(emojis)}")
print()
print(result)

# Also write to file
with open("all_emoji.txt", "w", encoding="utf-8") as f:
    f.write(result)
print(f"\nWritten to all_emoji.txt")
