
#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]
extern "C" {
    static mut level_objects: object;
    static mut level_monsters: object;
    static mut party_room: libc::c_short;
}

use libc::{setuid,perror,geteuid,getuid};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct obj {
    pub m_flags: libc::c_ulong,
    pub damage: *mut libc::c_char,
    pub quantity: libc::c_short,
    pub ichar: libc::c_short,
    pub kill_exp: libc::c_short,
    pub is_protected: libc::c_short,
    pub is_cursed: libc::c_short,
    pub class: libc::c_short,
    pub identified: libc::c_short,
    pub which_kind: libc::c_ushort,
    pub o_row: libc::c_short,
    pub o_col: libc::c_short,
    pub o: libc::c_short,
    pub row: libc::c_short,
    pub col: libc::c_short,
    pub d_enchant: libc::c_short,
    pub quiver: libc::c_short,
    pub trow: libc::c_short,
    pub tcol: libc::c_short,
    pub hit_enchant: libc::c_short,
    pub what_is: libc::c_ushort,
    pub picked_up: libc::c_short,
    pub in_use_flags: libc::c_ushort,
    pub next_object: *mut obj,
}
pub type object = obj;
#[no_mangle]
pub static mut saved_uid: libc::c_int = -(1 as libc::c_int);
#[no_mangle]
pub static mut true_uid: libc::c_int = -(1 as libc::c_int);
#[no_mangle]
pub unsafe extern "C" fn turn_into_games() {
    if setuid(saved_uid) == -(1 as libc::c_int) {
        perror(b"setuid(restore)\0" as *const u8 as *const libc::c_char);
        clean_up(b"\0" as *const u8 as *const libc::c_char);
    }
}
#[no_mangle]
pub unsafe extern "C" fn turn_into_user() {
    if setuid(true_uid) == -(1 as libc::c_int) {
        perror(b"setuid(restore)\0" as *const u8 as *const libc::c_char);
        clean_up(b"\0" as *const u8 as *const libc::c_char);
    }
}
unsafe fn main_0(
    mut argc: libc::c_int,
    mut argv: *mut *mut libc::c_char,
) -> libc::c_int {
    let mut current_block: u64;
    saved_uid = geteuid();
    true_uid = getuid();
    setuid(true_uid);
    if init(argc, argv) != 0 {
        current_block = 12396777863944641605;
    } else {
        current_block = 10680521327981672866;
    }
    loop {
        match current_block {
            12396777863944641605 => {
                play_level();
                free_stuff(&mut level_objects);
                free_stuff(&mut level_monsters);
                current_block = 10680521327981672866;
            }
            _ => {
                clear_level();
                make_level();
                put_objects();
                put_stairs();
                add_traps();
                put_mons();
                put_player(party_room as libc::c_int);
                print_stats(0o377 as libc::c_int);
                current_block = 12396777863944641605;
            }
        }
    };
}
pub fn main() {
    let mut args: Vec::<*mut libc::c_char> = Vec::new();
    for arg in ::std::env::args() {
        args.push(
            (::std::ffi::CString::new(arg))
                .expect("Failed to convert argument into CString.")
                .into_raw(),
        );
    }
    args.push(::core::ptr::null_mut());
    unsafe {
        ::std::process::exit(
            main_0(
                (args.len() - 1) as libc::c_int,
                args.as_mut_ptr() as *mut *mut libc::c_char,
            ) as i32,
        )
    }
}
