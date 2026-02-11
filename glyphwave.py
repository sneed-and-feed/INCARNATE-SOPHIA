import sys
import random

def glyphwave(text):
    # Combining diacritics range
    # 0x0300 - 0x036F are combining diacritical marks
    combining_chars = [chr(i) for i in range(0x0300, 0x0370)]
    
    result = ""
    for char in text:
        result += char
        # Add 1-5 random combining characters
        num_diacritics = random.randint(1, 6)
        for _ in range(num_diacritics):
            result += random.choice(combining_chars)
            
    return result

if __name__ == "__main__":
    if len(sys.argv) > 1:
        text = " ".join(sys.argv[1:])
        print(glyphwave(text))
    else:
        print("Usage: python glyphwave.py <text>")
