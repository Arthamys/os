use pic8259_simple::ChainedPics;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use lazy_static::lazy_static;
use spin;
use crate::{print, println};
use crate::gdt;
#[cfg(test)]
use crate::{serial_print, serial_println};

pub const PIC1_OFFSET : u8 = 32;
pub const PIC2_OFFSET : u8 = PIC1_OFFSET + 8;

pub static PICS : spin::Mutex<ChainedPics> = spin::Mutex::new(
    unsafe {ChainedPics::new(PIC1_OFFSET, PIC2_OFFSET)}
);

lazy_static!{
    static ref IDT: InterruptDescriptorTable = {
      let mut idt = InterruptDescriptorTable::new();
      idt.breakpoint.set_handler_fn(breakpoint_handler);
      // This is unsafe because we need to make sure the stack index is not
      // already used by another exception and is a valid memory location
      unsafe {
          idt.double_fault.set_handler_fn(double_fault_handler)
              .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
      }
      idt[InterruptIndex::Timer as usize ]
          .set_handler_fn(timer_interrupt_handler);
      idt
    };
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC1_OFFSET,
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: &mut InterruptStackFrame) {
    println!("EXCEPTION BREAKPOINT:\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    // error_code is always 0 when double_fault occurs
    stack_frame: &mut InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION DOUBLE FAULT:\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: &mut InterruptStackFrame) {
    print!(".");
    // This is unsafe because if we mess up the Interrupt Index we send back
    // to the PIC, we may consume an important interrupt, that could cuase
    // havock in another part of the system.
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8())
    };
}


#[test_case]
fn test_breakpoint_exception() {
    serial_print!("test_breakpoint_exception... ");
    x86_64::instructions::interrupts::int3();
    serial_println!("[ok]");
}
