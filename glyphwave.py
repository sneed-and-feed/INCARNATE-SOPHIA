import sys

def glyphwave(text):
    """
    Standard Sophia GlyphWave filter.
    Adds combining diacritics (\u035C, \u0361) for controlled glitching.
    """
    output = []
    for i, char in enumerate(text):
        if i % 3 == 1:
            output.append(char + "\u035C")
        elif i % 3 == 2:
            output.append(char + "\u0361")
        else:
            output.append(char)
    
    return f"🌀 {''.join(output)} 🌀"

if __name__ == "__main__":
    if len(sys.argv) > 1:
        text = " ".join(sys.argv[1:])
        print(glyphwave(text))
    else:
        print("Usage: python glyphwave.py <text>")
