//
//  compute.metal
//  PINS
//
//  Created by Linus Michelsson on 2023-11-29.
//

#include <metal_stdlib>
using namespace metal;

kernel void check_individual (
  constant uint *nums     [[ buffer(0) ]],
  device   char *results  [[ buffer(1) ]],
          uint3 index         [[ thread_position_in_grid ]]
) {
    results[index.x] = nums[index.x];
}

kernel void check_pin(constant  uint *offset        [[ buffer(0) ]],
                      device    char *resultArray   [[ buffer(1) ]],
                                uint3 index         [[ thread_position_in_grid ]]
                      ) {
    int pin[10];
    int year = offset[0] + index.x;
    int month = offset[1] + index.y;
    int day = offset[2] + index.z;
    
    int checksum = offset[3];
    
    pin[0] = year / 10;
    pin[1] = year % 10;
    
    pin[2] = month / 10;
    pin[3] = month % 10;
    
    pin[4] = day / 10;
    pin[5] = day % 10;
    
    pin[6] = checksum / 1000;
    pin[7] = checksum / 100;
    pin[8] = checksum / 10;
    pin[9] = checksum % 10;
    
    int sum = 0;
    
    for (int i = 0; i < 10; i++) {
        sum += pin[i] + ((i & 0b1) ^ 0b1) * (pin[i] + (pin[i] >= 5) * (-9));
    }
    
    resultArray[index.x + 100 * index.y + 10000 * index.z] = sum;
}
