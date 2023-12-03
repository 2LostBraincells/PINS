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

kernel void check_pin(
      constant  unsigned short *constants [[ buffer(0) ]],
      device    char *resultArray         [[ buffer(1) ]],
                uint3 index               [[ thread_position_in_grid ]]
  ) {
    

    int pin[10];
    int year = constants[0] + index.x;
    int month = constants[1] + index.y;
    int day = constants[2] + index.z;
  
    int greatest_day = 0;


    if (month > 12 || month == 0) {
      resultArray[index.z + constants[4] * index.y + constants[4] * constants[5] * index.x] = false;
      return;
    }

    switch (month) {
      case 2:
        greatest_day = 28 + (year % 4 == 0);
        break;
      case 4:
      case 6:
      case 9:
      case 11:
        greatest_day = 30;
        break;
      default:
        greatest_day = 31;
        break;
    }

    if (day == 0 || (day > greatest_day && day < 61) || day > greatest_day + 60) {
      resultArray[index.z + constants[4] * index.y + constants[4] * constants[5] * index.x] = false;
      return;
    }

    int checksum = constants[3];
    
    pin[0] = year / 10;
    pin[1] = year % 10;
    
    pin[2] = month / 10;
    pin[3] = month % 10;
    
    pin[4] = day / 10;
    pin[5] = day % 10;
    
    pin[6] = checksum / 1000;
    pin[7] = (checksum / 100) % 10;
    pin[8] = (checksum / 10) % 10;
    pin[9] = checksum % 10;
    
    int sum = 0;
    
    for (int i = 0; i < 10; i += 2) {
        /*
        if (i % 2 == 0) {
          if (pin[i] * 2 > 9) {
            sum += pin[i] * 2 % 10 + 1;
          } 
          else
          {
            sum += pin[i] * 2;
          }
        }
        else 
        {
          sum += pin[i];
        }
        */
        sum += pin[i] + (pin[i] + ((pin[i] >= 5) * (-9)));
    }

    for (int i = 1; i < 10; i += 2) {
      sum += pin[i];
    }


    /*
    sum += (pin[0] > 5) ? pin[0] * 2 % 10 + 1 : pin[0] * 2;
    sum += pin[1];
    sum += (pin[2] > 5) ? pin[2] * 2 % 10 + 1 : pin[2] * 2;
    sum += pin[3];
    sum += (pin[4] > 5) ? pin[4] * 2 % 10 + 1 : pin[4] * 2;
    sum += pin[5];
    sum += (pin[6] > 5) ? pin[6] * 2 % 10 + 1 : pin[6] * 2;
    sum += pin[7];
    sum += (pin[8] > 5) ? pin[8] * 2 % 10 + 1 : pin[8] * 2;
    sum += pin[9];
    */

    resultArray[index.z + constants[4] * index.y + constants[4] * constants[5] * index.x] = (sum % 10) == 0;
}

[[kernel]]
void dot_product(
  constant unsigned short *offsets [[buffer(0)]],
  device unsigned short *result [[buffer(1)]],
  uint index [[thread_position_in_grid]])
{
  result[index] = index + offsets[0];
}
