//
//  main.swift
//  PINS
//
//  Created by Linus Michelsson on 2023-11-29.
//

import MetalKit

let count: Int = 1_000_000

// Create our random arrays
var results = [Bool].init(repeating: false, count: count)
var offsets = [Int].init(repeating: 0, count: 4)

computeWay(results: results, offsets: offsets)


func computeWay(results: [Bool], offsets: [Int]) -> Int {
    print("setting up")
    
    // The GPU we want to use
    let device = MTLCreateSystemDefaultDevice()

    // A fifo queue for sending commands to the gpu
    let commandQueue = device?.makeCommandQueue()

    // A library for getting our metal functions
    let gpuFunctionLibrary = device?.makeDefaultLibrary()

    // Grab our gpu function
    let additionGPUFunction = gpuFunctionLibrary?.makeFunction(name: "check_pin")

    var additionComputePipelineState: MTLComputePipelineState!
    do {
        additionComputePipelineState = try device?.makeComputePipelineState(function: additionGPUFunction!)
    } catch {
      print(error)
    }

    
    // Create results buffer
    let offsetBuff = device?.makeBuffer(bytes: offsets,
                                        length: MemoryLayout<Int>.size * 4,
                                        options: .storageModeShared)
    
    // Create results buffer
    let resultBuff = device?.makeBuffer(bytes: results,
                                        length: MemoryLayout<Bool>.size * count,
                                        options: .storageModeShared)
    
    // Get the pointer to the beginning of our data
    var offsetBufferPointer = offsetBuff?.contents().bindMemory(to: Int.self,
                                                                capacity: MemoryLayout<Int>.size * count)
    
    
    print("buffers created")




    
    // Call our functions
    let startTime = CFAbsoluteTimeGetCurrent()
    var x = 0
    for i in 0...10000 {
        print("spwaning")
        
        offsetBufferPointer?.advanced(by: 3).pointee = i
        
        // Create a buffer to be sent to the command queue
        let commandBuffer = commandQueue?.makeCommandBuffer()

        // Create an encoder to set vaulues on the compute function
        let commandEncoder = commandBuffer?.makeComputeCommandEncoder()
        commandEncoder?.setComputePipelineState(additionComputePipelineState)

        // Set the parameters of our gpu function
        commandEncoder?.setBuffer(offsetBuff, offset: 0, index: 0)
        commandEncoder?.setBuffer(resultBuff, offset: 0, index: 1)
        
        // Figure out how many threads we need to use for our operation
        let threadsPerGrid = MTLSize(width: 100, height: 100, depth: 100)
        let maxThreadsPerThreadgroup = additionComputePipelineState.maxTotalThreadsPerThreadgroup // 1024
        let threadsPerThreadgroup = MTLSize(width: maxThreadsPerThreadgroup, height: 1, depth: 1)
        commandEncoder?.dispatchThreads(threadsPerGrid,
                                        threadsPerThreadgroup: threadsPerThreadgroup)

        // Tell the encoder that it is done encoding.  Now we can send this off to the gpu.
        commandEncoder?.endEncoding()
        
        // Push this command to the command queue for processing
        commandBuffer?.commit()

        // Wait until the gpu function completes before working with any of the data
        commandBuffer?.waitUntilCompleted()
    }
    let timeElapsed = CFAbsoluteTimeGetCurrent() - startTime
    print("Time elapsed \(String(format: "%.05f", timeElapsed)) seconds")


    
    print("done")


    
    return 1
}

func basicForLoopWay(arr1: [UInt8], arr2: [UInt8]) {
    print("CPU Way")
    
    // Begin the process
    let startTime = CFAbsoluteTimeGetCurrent()

    var result = [UInt8].init(repeating: 0, count: count)

    // Process our additions of the arrays together
    for i in 0..<count {
        result[i] = arr1[i] + arr2[i]
    }

    // Print out the results
    for i in 0..<3 {
        print("\(arr1[i]) + \(arr2[i]) = \(result[i])")
    }

    // Print out the elapsed time
    let timeElapsed = CFAbsoluteTimeGetCurrent() - startTime
    print("Time elapsed \(String(format: "%.05f", timeElapsed)) seconds")

    print()
}

// Helper function
func getRandomArray()->[UInt8] {
    let startTime = CFAbsoluteTimeGetCurrent()
    var result = [UInt8].init(repeating: 0, count: count)
    //for i in 0..<count {
    //    result[i] = UInt8(arc4random_uniform(10))
    //}
    let timeElapsed = CFAbsoluteTimeGetCurrent() - startTime
    print("Time elapsed \(String(format: "%.05f", timeElapsed)) seconds")
    return result
}
