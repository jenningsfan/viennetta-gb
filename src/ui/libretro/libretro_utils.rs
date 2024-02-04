use std::ffi::c_void;

pub fn convert_data_to_vec(data: *const c_void, len: usize) -> Vec<u8> {
    // Safety: Ensure that the pointer is valid and doesn't cause UB
    let data_slice = unsafe { std::slice::from_raw_parts(data as *const u8, len) };

    // Convert the slice to a Vec<u8>
    data_slice.to_vec()
}