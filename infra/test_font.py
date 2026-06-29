from fontTools.ttLib import TTFont
import sys

def has_char(font_path, char_code):
    font = TTFont(font_path)
    for table in font['cmap'].tables:
        if char_code in table.cmap:
            return True
    return False

print("DejaVu Sans:", has_char("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 0x1F62D))
print("Noto Sans Symbols:", has_char("/usr/share/fonts/truetype/noto/NotoSansSymbols-Regular.ttf", 0x1F62D))
print("Noto Sans Symbols 2:", has_char("/usr/share/fonts/truetype/noto/NotoSansSymbols2-Regular.ttf", 0x1F62D))
