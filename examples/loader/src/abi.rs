use axlog::info;
use axstd::{println, process::exit};

const SYS_HELLO: usize = 1;
const SYS_PUTCHAR: usize = 2;
const SYS_EXIT: usize = 3;

pub static mut ABI_TABLE: [usize; 16] = [0; 16];

// map func name to func addr
pub static STR_TO_FUNC: [(&str, AbiFunction); 3] = [
    ("hello", AbiFunction::Hello(abi_hello)),
    ("putchar", AbiFunction::Putchar(abi_putchar)),
    ("exit", AbiFunction::Exit(abi_exit)),
];

pub fn init_abis() {
    info!("abi_hello: 0x{:x}", abi_hello as usize);
    register_abi(SYS_HELLO, abi_hello as usize);
    info!("abi_putchar: 0x{:x}", abi_putchar as usize);
    register_abi(SYS_PUTCHAR, abi_putchar as usize);
    info!("abi_exit: 0x{:x}", abi_exit as usize);
    register_abi(SYS_EXIT, abi_exit as usize);
}

#[derive(Clone, Copy, Debug)]
pub enum AbiFunction {
    Hello(fn() -> ()),
    Putchar(fn(char) -> ()),
    Exit(fn() -> !),
}

impl AbiFunction {

    pub fn from_name(name: &str) -> Option<Self> {
        for (n, f) in STR_TO_FUNC.iter() {
            if n == &name {
                return Some(*f);
            }
        }
        None
    }

    pub fn addr(&self) -> usize {
        match self {
            AbiFunction::Hello(f) => *f as usize,
            AbiFunction::Putchar(f) => *f as usize,
            AbiFunction::Exit(f) => *f as usize,
        }
    }
}

fn register_abi(num: usize, handle: usize) {
    unsafe {
        ABI_TABLE[num] = handle;
    }
}

fn abi_hello() {
    println!("[ABI:Hello] Hello, Apps!");
}

fn abi_putchar(c: char) {
    const LEGACY_CONSOLE_PUTCHAR: usize = 1;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") LEGACY_CONSOLE_PUTCHAR,
            in("a0") c as usize,
        );
    }
}

fn abi_exit() -> ! {
    println!("[ABI:Exit] Exit!");
    exit(0);
}
