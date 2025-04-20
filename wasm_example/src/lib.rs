unsafe extern "C" {
    fn get_input(ptr: i32, len: i32) -> i32;
    fn get_trigger(ptr: i32, len: i32) -> i32;
    fn set_output(ptr: i32, len: i32);
}

#[unsafe(no_mangle)]
pub extern "C" fn handle() {
    let mut input_buffer = [0u8; 1024];
    let mut trigger_buffer = [0u8; 256];

    let input_len =
        unsafe { get_input(input_buffer.as_mut_ptr() as i32, input_buffer.len() as i32) };
    let trigger_len = unsafe {
        get_trigger(
            trigger_buffer.as_mut_ptr() as i32,
            trigger_buffer.len() as i32,
        )
    };

    if input_len <= 0 || trigger_len <= 0 {
        return;
    }

    let input = &input_buffer[0..input_len as usize];
    let trigger = &trigger_buffer[0..trigger_len as usize];

    let input_str = String::from_utf8_lossy(input).to_string();
    let trigger_str = String::from_utf8_lossy(trigger).to_string();

    let output = format!("Echo: {} from trigger: {}", input_str, trigger_str);
    let output_bytes = output.as_bytes();

    unsafe {
        set_output(output_bytes.as_ptr() as i32, output_bytes.len() as i32);
    }
}
