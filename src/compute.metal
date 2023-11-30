kernel void calculate_pins( constant uint *offset       [[ buffer(0) ]],
                            device   bool *resultArray  [[ buffer(1) ]],
                                     uint3 index        [[ thread_position_in_grid ]]) {
  // PINS calculation goes in here
}