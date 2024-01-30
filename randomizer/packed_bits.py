# This file is heavily based on the equivalent file in the Wind Waker Randomizer codebase (SD).
# That file can be found here: https://github.com/LagoLunatic/wwrando/blob/master/wwr_ui/packedbits.py
import base64
import struct


class PackedBitsWriter:
    def __init__(self):
        self.bits_left_in_byte = 8
        self.current_byte = 0
        self.bytes = bytearray()

    def write(self, value: int, length: int):
        while length:
            if length >= self.bits_left_in_byte:
                bits_to_read = self.bits_left_in_byte
            else:
                bits_to_read = length

            mask = (1 << bits_to_read) - 1
            self.current_byte |= (value & mask) << (8 - self.bits_left_in_byte)

            self.bits_left_in_byte -= bits_to_read
            length -= bits_to_read
            value >>= bits_to_read

            if self.bits_left_in_byte:
                continue

            self.flush()

    def flush(self):
        self.bytes.append(self.current_byte)
        self.current_byte = 0
        self.bits_left_in_byte = 8

    def get_packed_bytes(self) -> bytes:
        packed_bytes = b""

        for byte in self.bytes:
            packed_bytes += struct.pack(">B", byte)

        return packed_bytes


class PackedBitsReader:
    def __init__(self, bytes: bytes):
        self.bytes = bytearray(struct.unpack(">" + "B" * len(bytes), bytes))
        self.current_byte_index = 0
        self.current_bit_index = 0

    def read(self, length: int):
        bits_read = 0
        value = 0
        bits_left_to_read = length

        while bits_read != length:
            if bits_left_to_read > 8:
                bits_to_read = 8
            else:
                bits_to_read = bits_left_to_read

            if bits_to_read + self.current_bit_index > 8:
                bits_to_read = 8 - self.current_bit_index

            mask = ((1 << bits_to_read) - 1) << self.current_bit_index

            current_byte = self.bytes[self.current_byte_index]
            value = (
                (current_byte & mask) >> self.current_bit_index
            ) << bits_read | value

            self.current_bit_index += bits_to_read
            self.current_byte_index += self.current_bit_index >> 3
            self.current_bit_index %= 8
            bits_left_to_read -= bits_to_read
            bits_read += bits_to_read

        return value
