#!/usr/bin/env python3
import struct

# Create a minimal 16x16 ICO file
def create_minimal_ico():
    # ICO header (6 bytes)
    ico_header = struct.pack('<HHH', 0, 1, 1)  # Reserved, Type (1=ICO), Count
    
    # ICO directory entry (16 bytes)
    width = 16
    height = 16
    colors = 0  # 0 means 256 or more colors
    reserved = 0
    planes = 1
    bits_per_pixel = 32
    size_in_bytes = 40 + (width * height * 4)  # DIB header + pixel data
    offset = 22  # 6 (ICO header) + 16 (directory entry)
    
    ico_dir_entry = struct.pack('<BBBBHHLL', 
                                width, height, colors, reserved,
                                planes, bits_per_pixel, size_in_bytes, offset)
    
    # DIB header (40 bytes) - BITMAPINFOHEADER
    dib_header = struct.pack('<LLLHHLLLLLL',
                            40,  # header size
                            width,  # width
                            height * 2,  # height (doubled for ICO)
                            1,  # planes
                            bits_per_pixel,  # bits per pixel
                            0,  # compression
                            width * height * 4,  # image size
                            0,  # x pixels per meter
                            0,  # y pixels per meter
                            0,  # colors used
                            0)  # important colors
    
    # Pixel data (16x16 pixels, 4 bytes each - BGRA)
    # Create a simple blue square
    pixel_data = b''
    for y in range(height):
        for x in range(width):
            # BGRA format: Blue, Green, Red, Alpha
            pixel_data += struct.pack('<BBBB', 255, 0, 0, 255)  # Blue pixel
    
    # Combine all parts
    ico_data = ico_header + ico_dir_entry + dib_header + pixel_data
    
    return ico_data

# Write the ICO file
ico_data = create_minimal_ico()
with open('src-tauri/icons/icon.ico', 'wb') as f:
    f.write(ico_data)

print("Created minimal ICO file")
