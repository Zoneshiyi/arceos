#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;

const PLASH_START: usize = 0xffff_ffc0_2200_0000;

const MAX_APP_SIZE: usize = 0x100000;

use core::{
    mem::size_of,
    slice::from_raw_parts,
};

#[derive(Clone, Copy)]
struct ImageHeader {
    app_num: u32,
}

#[derive(Clone, Copy)]
struct AppHeader {
    app_size: usize,
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {

    println!("Load payload ...");

    let image_header = unsafe {*(PLASH_START as *const ImageHeader)};
    let app_num = image_header.app_num;
    println!("app_num: {}", app_num);

    let mut offset = PLASH_START + size_of::<ImageHeader>();

    for _i in 0..app_num {
        let app_header = unsafe {*(offset as *const AppHeader)};
        let app_size = app_header.app_size;
        println!("app_size: {}", app_size);

        offset += size_of::<AppHeader>();
        let app = unsafe {from_raw_parts(offset as *const u8, app_size.max(MAX_APP_SIZE))};
        println!("app: {:?}", &app[0..10.min(app_size)]);

        offset += app_size;
    }

    println!("Load payload ok!");
}