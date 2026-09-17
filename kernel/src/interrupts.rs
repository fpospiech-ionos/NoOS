use lazy_static::lazy_static;
use x86_64::structures::{idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode}, paging::page};

use crate::{gdt::DOUBLE_FAULT_IST_INDEX, println};

lazy_static!{
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);

        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

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
}

extern "x86-interrupt" fn general_protection_fault_handler(stack_frame: InterruptStackFrame, code: u64) {
    println!("[GP Code] {:?}", code);
    println!("[GP Stack] {:?}", stack_frame);
}

pub fn init_idt() {
    IDT.load();
}