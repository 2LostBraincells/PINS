#!/bin/zsh

xcrun -sdk macosx metal -c compute.metal -o compute.air  
xcrun -sdk macosx metallib compute.air -o compute.metallib

rm compute.air