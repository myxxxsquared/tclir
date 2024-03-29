
import serial
import struct

VOLUME_UP = 'B111100101111000011010000E'
VOLUME_DOWN = 'B111100101110000011010001E'
POWER = 'B111100101010000011010101E'
UP = 'B111101011001000010100110E'
DOWN = 'B111101011000000010100111E'
OK = 'B111111110100000000001011E'
LEFT = 'B111101010110000010101001E'
RIGHT = 'B111101010111000010101000E'
SOURCE = 'B111110100011000001011100E'

def process(x):
    assert len(x) == 26
    x = x[1:-1]
    x = int(x, 2)
    x = b'Sb' + struct.pack("<I", x)
    return x


s = serial.Serial('COM5', baudrate=115200, bytesize=8, parity='N', stopbits=2)
s.write(process(POWER))
