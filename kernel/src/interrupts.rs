use spin::Mutex;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use x86_64::structures::{idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode}};

use crate::{gdt::DOUBLE_FAULT_IST_INDEX, print, println};

lazy_static!{
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);

        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }
        
        idt[Interrupt::Timer as u8].set_handler_fn(timer_interrupt_hander);
        idt[Interrupt::Keyboard as u8].set_handler_fn(keyboard_interrupt_handler);

        idt
    };
}

pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe { ChainedPics::new(32, 40)} );

enum Interrupt {
    Timer = 32,
    Keyboard,
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("[BREAKPOINT] {:?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) -> ! {
    panic!("[EXCEPTION] (double fault: {:?}) {:?}", error_code, stack_frame)
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, code: PageFaultErrorCode) {
        println!("[PAGE FAULT Code] {:?}", code);
    println!("[PAGE FAULT Stack] {:?}", stack_frame);

    panic!();
}

extern "x86-interrupt" fn general_protection_fault_handler(stack_frame: InterruptStackFrame, code: u64) {
    println!("[GP Code] {:?}", code);
    println!("[GP Stack] {:?}", stack_frame);

    panic!();

}

extern "x86-interrupt" fn timer_interrupt_hander(_stack_frame: InterruptStackFrame) {
    unsafe {
        PICS.lock().notify_end_of_interrupt(Interrupt::Timer as u8);
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{DecodedKey, HandleControl, PS2Keyboard, ScancodeSet1, layouts::{AnyLayout, Us104Key}};
    use x86_64::instructions::port::Port;
    use spin::Mutex;

    lazy_static! {
        static ref KEYBOARD: Mutex<PS2Keyboard<AnyLayout, ScancodeSet1>> = Mutex::new(PS2Keyboard::new(ScancodeSet1::new(), AnyLayout::Us104Key(Us104Key), HandleControl::Ignore));
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port  = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(char)  => print!("{}", char),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock().notify_end_of_interrupt(Interrupt::Keyboard as u8);
    }
}

pub fn init_idt() {
    IDT.load();
}