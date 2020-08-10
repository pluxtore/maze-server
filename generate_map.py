#!/usr/bin/python3
import cv2 as cv
import numpy as np
from collections import deque as deque

# this is basically because I wasn't able to get opencv imported into the rust project

img = cv.imread("map-bw.png",0)
(rows,cols) = img.shape

stack = [] 

temp_array = bytearray()

for i in range(rows):
    for j in range(cols):
        temp_array.append(img[i,j])
    stack.append(temp_array)
    temp_array = bytearray()

stack.reverse()

final = bytearray()

for row in stack:
    for px in row:
        final.append(px)

open("maze_gameserver/maze.map", "wb+").write(final)
