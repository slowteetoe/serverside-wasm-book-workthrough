use core::ptr;

#[unsafe(no_mangle)]
pub fn greet(ptr: i32, len: i32) {
    let input_ptr = ptr as *mut u8;
    let input_len = len as usize;

    let hello = b"Hello, ";
    let output_ptr: *mut u8 = ptr::without_provenance_mut(0);

    unsafe {
        for i in 0..hello.len() {
            output_ptr.add(i).write_volatile(hello[i]);
        }

        for i in 0..input_len {
            output_ptr
                .add(hello.len() + i)
                .write_volatile(input_ptr.add(i).read());
        }
    }
}
