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

func computeWay(results: [Bool], offsets: [Int]) {
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
    let offsetBufferPointer = offsetBuff?.contents().bindMemory(to: Int.self,
                                                                capacity: MemoryLayout<Int>.size * count)
    
    var resultBufferPointer = resultBuff?.contents().bindMemory(to: Bool.self,
                                                                capacity: MemoryLayout<Bool>.size * count)
    
    print("buffers created")

    
    // Call our functions
    let startTime = CFAbsoluteTimeGetCurrent()
    for i in 0...10000 {
        print(String(Double(i)/100.0) + "%")
        
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
        
        // Dont tell me what the ! does i have no idea
        parseBlock(results: resultBufferPointer!, index: i)
    }
    let timeElapsed = CFAbsoluteTimeGetCurrent() - startTime
    print("Time elapsed \(String(format: "%.05f", timeElapsed)) seconds")


    
    print("done")
}

func parseBlock(results: UnsafeMutablePointer<Bool>, index: Int) {
    // Open the file
    let filePath = #file
    let dir = URL(fileURLWithPath: filePath).deletingLastPathComponent()
    let url = dir.appendingPathComponent("output.txt")
    print(url)
    let fileHandle = FileHandle(forWritingAtPath: url.path)
    
    var str: Data;
    
    // Parse the valid numbers as text
    
    for day in 0...100 {
        for month in 0...100 {
            for year in 0...100 {
                // Check that the string is valid
                if (!results[year + 100 * month + 10000 * day]) { continue }
                
                // format and write the string to a file
                str = "\(year)\(month)\(day)\(String(format: "%04d", index)) ".data(using: .utf8)!
                fileHandle?.seekToEndOfFile()
                fileHandle?.write(str)
            }
        }
    }
    
    fileHandle?.closeFile()
}
