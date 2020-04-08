#![no_std]
#![no_main]

use core::panic::PanicInfo;
use os::{QemuExitCode, exit_qemu, serial_println, serial_print};

#[no_mangle]
extern "C" fn _start() -> ! {
    should_fail();

    loop {}
}

fn should_fail() {
    serial_print!("should fail... ");
    assert_eq!(0, 1);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}
