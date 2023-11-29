//
//  compute.metal
//  PINS
//
//  Created by Linus Michelsson on 2023-11-29.
//

#include <metal_stdlib>
using namespace metal;

kernel void addition_compute_function(constant uint *arr1        [[ buffer(0) ]],
                                      constant uint *arr2        [[ buffer(1) ]],
                                      device   uint *resultArray [[ buffer(2) ]],
                                               uint   index [[ thread_position_in_grid ]]) {
    for (int i = 0; i < 10; i++) {
        resultArray[index] = arr1[index] + (!(i & 0b1)) * (arr1[index] + (arr1[index] >= 5) * (-9));
    }
}
