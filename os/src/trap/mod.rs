pub mod trap_context;

use core::arch::{asm, global_asm};
use riscv::register::{mie, mtvec::TrapMode, satp, scause::{self, Exception, Interrupt, Trap}, sepc, sie, stval, stvec};
use crate::mm::{TRAMPOLINE, TRAP_CONTEXT};
use crate::println;
use crate::process::scheduler::{Scheduler, SCHEDULER};
use crate::syscall::syscall;

global_asm!(include_str!("trap.S"));

pub fn init() {
    set_kernel_trap_entry();
    unsafe {
        sie::set_sext();
        sie::set_stimer();
        sie::set_ssoft();
    }
}

fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(trap_from_kernel as usize, TrapMode::Direct);
    }
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
    }
}


#[no_mangle]
pub fn trap_handler() -> ! {
    set_kernel_trap_entry();
    let scause = scause::read();
    let stval = stval::read();
    //Different action corresponding to scause
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            let mut cxt = Scheduler::get_cur_trap_cxt();
            cxt.sepc += 4;
            let result = syscall(cxt.x[17], [cxt.x[10], cxt.x[11], cxt.x[12]]);
            cxt = Scheduler::get_cur_trap_cxt();
            cxt.x[10] = result as usize;
        }
        Trap::Exception(Exception::StoreFault)
        | Trap::Exception(Exception::StorePageFault)
        | Trap::Exception(Exception::InstructionFault)
        | Trap::Exception(Exception::InstructionPageFault)
        | Trap::Exception(Exception::LoadFault)
        | Trap::Exception(Exception::LoadPageFault) => {
            println!(
                "[kernel] {:?} in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.",
                scause.cause(),
                stval,
                Scheduler::get_cur_trap_cxt().sepc,
            );
            Scheduler::kernel_exit(-2);
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            // illegal instruction exit code
            Scheduler::kernel_exit(-3);
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            Scheduler::kernel_yield();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    trap_return();
}

#[no_mangle]
pub fn trap_return() -> ! {
    set_user_trap_entry();
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = Scheduler::get_cur_token();
    extern "C" {
        fn __alltraps();
        fn __restore();
    }
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    unsafe {
        asm!(
        "fence.i",
        "jr {restore_va}",
        restore_va = in(reg) restore_va,
        in("a0") trap_cx_ptr,
        in("a1") user_satp,
        options(noreturn)
        );
    }
}

#[no_mangle]
pub fn trap_from_kernel() -> ! {
    println!( "Kernel trap in scause {}, stval {:#x}, sepc {:#x}, and satp {:#x}.",
              scause::read().bits(),
              stval::read(),
              sepc::read(),
              satp::read().bits(),);
    panic!("a trap {:?} from kernel!", scause::read().cause());
}

