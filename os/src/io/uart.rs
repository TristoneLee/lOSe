#![no_std]
#![no_main]

use core::sync::atomic::AtomicU64;
use crate::io::STDOUT;

pub const UART0: u64 = 0x10000000;
const UART0_IRQ: u32 = 10;
const RHR: u64 = 0;
const THR: u64 = 0;
const IER: u64 = 1;
const IER_RX_ENABLE: u8 = 1 << 0;
const IER_TX_ENABLE: u8 = 1 << 1;
const FCR: u64 = 2;
const FCR_FIFO_ENABLE: u8 = 1 << 0;
const FCR_FIFO_CLEAR: u8 = 3 << 1;
const ISR: u64 = 2;
const LCR: u64 = 3;
const LCR_EIGHT_BITS: u8 = 3 << 0;
const LCR_BAUD_LATCH: u8 = 1 << 7;
const LSR: u64 = 5;
const LSR_TX_READY: u8 = 1 << 0;
const LSR_TX_IDLE: u8 = 1 << 5;

unsafe fn read_reg(reg: u64) -> u8 {
    let ptr = (UART0 + reg) as * mut u8;
    return *ptr;
}

unsafe fn write_reg(reg: u64,ch: u8) {
    let ptr = (UART0 + reg) as *mut u8;
    *ptr = ch;
}

const UART_TX_BUF_SIZE: u64 = 32;

static mut uart_tx_buf: [u64; UART_TX_BUF_SIZE as usize] = [0; UART_TX_BUF_SIZE as usize];
static mut uart_tx_w: u64=0 ;
static mut uart_tx_r: u64=0 ;


pub unsafe fn uart_init() {
    write_reg(IER, 0x00);
    write_reg(LCR, LCR_BAUD_LATCH);
    write_reg(0, 0x03);
    write_reg(1, 0x00);
    write_reg(LCR, LCR_EIGHT_BITS);
    write_reg(FCR, FCR_FIFO_ENABLE | FCR_FIFO_CLEAR);
    write_reg(IER, IER_TX_ENABLE | IER_RX_ENABLE);
}

pub unsafe  fn uart_putchar(c: u8) {
    while uart_tx_w == uart_tx_r + UART_TX_BUF_SIZE {}
    uart_tx_buf[(uart_tx_w % UART_TX_BUF_SIZE) as usize] = c as u64;
    uart_tx_w += 1;
    uart_work();
}

pub unsafe fn uart_getchar() -> u8 {
    return if (read_reg(LSR) & 0x01)==1 {
        read_reg(RHR)
    } else {0}
}

pub unsafe fn uart_work() {
    while true{
        if uart_tx_w== uart_tx_r {
            return
        }
        if read_reg(LSR)&LSR_TX_IDLE==0 {
            return
        }
        let ch = uart_tx_buf[(uart_tx_r%UART_TX_BUF_SIZE)as usize] as u8;
        uart_tx_r+=1;
        write_reg(THR,ch);
    }
}


