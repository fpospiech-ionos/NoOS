#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(abi_x86_interrupt)]

use bootloader_api::{BootInfo, entry_point};

use crate::{gdt::init, interrupts::{PICS, init_idt}};

pub mod gdt;
pub mod serial;
pub mod interrupts;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    use x86_64::instructions::{nop, port::Port};

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }

    loop {
        nop();
    }
}

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    init();
    init_idt();
    println!("Entered kernel with boot info: {:?}", boot_info);
    println!("\n=(^.^)= meow\n");

    unsafe { PICS.lock().initialize(); }
    unsafe { PICS.lock().write_masks(0, 0); }

    x86_64::instructions::interrupts::enable();

    println!("\nMeow Meow?\n");

    loop {
        x86_64::instructions::hlt();
    }
}

/// This function is called on panic.
#[panic_handler]
#[cfg(not(test))]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("PANIC: {:?}", info);
    exit_qemu(QemuExitCode::Failed);
}