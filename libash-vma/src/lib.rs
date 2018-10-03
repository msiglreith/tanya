pub mod ffi;

#[test]
fn test_basic() {
    let allocator = unsafe { std::mem::uninitialized() };
    let pool = unsafe { std::mem::uninitialized() };
    unsafe { ffi::vmaDestroyPool(allocator, pool) };
}
