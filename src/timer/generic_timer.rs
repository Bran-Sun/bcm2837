extern crate aarch64;

use super::BasicTimer;
use crate::addr::phys_to_virt;
use aarch64::regs::*;
use volatile::*;

/// The base address for the ARM generic timer, IRQs, mailboxes
const GEN_TIMER_REG_BASE: usize = phys_to_virt(0x4000_0000);

/// Core interrupt sources (ref: QA7 4.10, page 16)
#[repr(u8)]
#[allow(dead_code)]
#[allow(non_snake_case)]
#[derive(Copy, Clone, PartialEq, Debug)]
enum CoreInterrupt {
    CNTPSIRQ = 0,
    CNTPNSIRQ = 1,
    CNTHPIRQ = 2,
    CNTVIRQ = 3,
    Mailbox0 = 4,
    Mailbox1 = 5,
    Mailbox2 = 6,
    Mailbox3 = 7,
    Gpu = 8,
    Pmu = 9,
    AxiOutstanding = 10,
    LocalTimer = 11,
}

/// Timer, IRQs, mailboxes registers (ref: QA7 chapter 4, page 7)
#[allow(non_snake_case)]
#[repr(C)]
struct Registers {
    CONTROL: Volatile<u32>,
    _unused1: [Volatile<u32>; 8],
    LOCAL_IRQ: Volatile<u32>,
    _unused2: [Volatile<u32>; 3],
    LOCAL_TIMER_CTL: Volatile<u32>,
    LOCAL_TIMER_FLAGS: Volatile<u32>,
    _unused3: Volatile<u32>,
    CORE_TIMER_IRQCNTL: [Volatile<u32>; 4],
    CORE_MAILBOX_IRQCNTL: [Volatile<u32>; 4],
    CORE_IRQ_SRC: [Volatile<u32>; 4],
}

/// The ARM generic timer.
pub struct GenericTimer {
    registers: &'static mut Registers,
    cntfrq: u64,
}

impl BasicTimer for GenericTimer {
    fn new() -> Self {
        GenericTimer {
            registers: unsafe { &mut *(GEN_TIMER_REG_BASE as *mut Registers) },
            cntfrq: CNTFRQ_EL0.get() as u64, // 62500000
        }
    }

    fn init(&mut self) {
        self.registers.CORE_TIMER_IRQCNTL[0].write(1 << (CoreInterrupt::CNTPNSIRQ as u8));
        CNTP_CTL_EL0.write(CNTP_CTL_EL0::ENABLE::SET);
    }

    fn read(&self) -> u64 {
        (CNTPCT_EL0.get() * 1000000 / self.cntfrq) as u64
    }

    fn tick_in(&mut self, us: u32) {
        CNTP_TVAL_EL0.set((self.cntfrq * (us as u64) / 1000000) as u32);
    }

    fn is_pending(&self) -> bool {
        self.registers.CORE_IRQ_SRC[0].read() & (1 << (CoreInterrupt::CNTPNSIRQ as u8)) != 0
    }
}
