use metal::*;

const LIB_DATA: &[u8] = include_bytes!("shaders/compute.metallib");

pub fn max_group() -> MTLSize {

    return metal::MTLSize::new(
        128,//width
        1, // height
        1); //depth
}

pub fn get_device() -> Device {
    //! Get gpu device
    return Device::system_default().expect("No device found");
}


fn get_func(device: Device, function: &str) -> Function {
    // Library reference and function reference
    let lib = device.new_library_with_data(LIB_DATA).unwrap();
    return lib.get_function(function, None).unwrap();
}



pub fn use_function(device: &Device, function: &str, encoder: &ComputeCommandEncoderRef) {

    let func = get_func(device.clone(), function);

    let pipeline = device
        .new_compute_pipeline_state_with_function(&func)
        .unwrap();

    encoder.set_compute_pipeline_state(&pipeline);
}
