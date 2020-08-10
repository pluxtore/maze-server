#!/usr/bin/python3
import socket
import time
import random
import threading
import struct

# WHITE RABBIT BOT # 

SERVER =  ("127.0.0.1", 1337)

def encode(r):
    first_random    = random.randint(1,255)
    second_random   = random.randint(1,255)
    encoded = bytearray()

    encoded.append(first_random)
    encoded.append(second_random)

    for i in range(0, len(r)):
        encoded.append(first_random ^ r[i])
        v21 = first_random + second_random
        first_random = (v21 + ((2155905153 * v21) >> 39)) & 0xff

    return encoded


def decode(r):
    first_random    = r[0]
    second_random   = r[1]
    decoded = bytearray()
    for i in range(0, len(r) - 2):
        decoded.append(first_random ^ r[i+2])
        v21 = first_random + second_random
        first_random = (v21 + ((2155905153 * v21) >> 39)) & 0xff

    return decoded

draw_data = open("whiterabbit_get_there_data", "rb").read() + open("whiterabbit_data", "rb").read()
secret = b"\x33" * 8 # changed for obvious reasons
name = b"The White Rabbit"
padding =  ( 42 - (2 + len(name) + len(secret)) )  * b"\x00"

sock = socket.socket(socket.AF_INET,socket.SOCK_DGRAM)
sock.sendto(encode(b"\x4c" + secret + b"\x00" + name + padding),SERVER) # login
time.sleep(0.5)
indx = 0
basetime = int(time.time())  % 2**32
while True:
    sock.sendto(encode(b"\x3c\x33" + secret + struct.pack('d', basetime)),SERVER) # heartbeat
    sock.recvfrom(1024)
    time.sleep(0.2)
    sock.sendto(encode(b"\x50" + secret + struct.pack('i', basetime) + b"\x00" *4 + draw_data[indx:indx+29]),SERVER) # pos updates
    indx+=29
    if (indx+29)>=len(draw_data):
        indx = len(open("whiterabbit_get_there_data", "rb").read())

        

        