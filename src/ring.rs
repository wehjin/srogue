#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]
extern "C" {
    static mut rogue: fighter;
    fn strcpy(_: *mut libc::c_char, _: *const libc::c_char) -> *mut libc::c_char;
    fn reg_move() -> libc::c_char;
    fn get_letter_object() -> *mut object;
    static mut curse_message: *mut libc::c_char;
    static mut wizard: libc::c_char;
    fn sprintf(_: *mut libc::c_char, _: *const libc::c_char, _: ...) -> libc::c_int;
}
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct fight {
    pub armor: *mut object,
    pub weapon: *mut object,
    pub left_ring: *mut object,
    pub right_ring: *mut object,
    pub hp_current: libc::c_short,
    pub hp_max: libc::c_short,
    pub str_current: libc::c_short,
    pub str_max: libc::c_short,
    pub pack: object,
    pub gold: libc::c_long,
    pub exp: libc::c_short,
    pub exp_points: libc::c_long,
    pub row: libc::c_short,
    pub col: libc::c_short,
    pub fchar: libc::c_short,
    pub moves_left: libc::c_short,
}
pub type fighter = fight;
#[no_mangle]
pub static mut left_or_right: *mut libc::c_char = b"left or right hand?\0" as *const u8
    as *const libc::c_char as *mut libc::c_char;
#[no_mangle]
pub static mut no_ring: *mut libc::c_char = b"there's no ring on that hand\0"
    as *const u8 as *const libc::c_char as *mut libc::c_char;
#[no_mangle]
pub static mut stealthy: libc::c_short = 0;
#[no_mangle]
pub static mut r_rings: libc::c_short = 0;
#[no_mangle]
pub static mut add_strength: libc::c_short = 0;
#[no_mangle]
pub static mut e_rings: libc::c_short = 0;
#[no_mangle]
pub static mut regeneration: libc::c_short = 0;
#[no_mangle]
pub static mut ring_exp: libc::c_short = 0;
#[no_mangle]
pub static mut auto_search: libc::c_short = 0;
#[no_mangle]
pub static mut r_teleport: libc::c_char = 0;
#[no_mangle]
pub static mut r_see_invisible: libc::c_char = 0;
#[no_mangle]
pub static mut sustain_strength: libc::c_char = 0;
#[no_mangle]
pub static mut maintain_armor: libc::c_char = 0;
#[no_mangle]
pub unsafe extern "C" fn put_on_ring() -> libc::c_int {
    let mut ch: libc::c_short = 0;
    let mut desc: [libc::c_char; 80] = [0; 80];
    let mut ring: *mut object = 0 as *mut object;
    if r_rings as libc::c_int == 2 as libc::c_int {
        message(
            b"wearing two rings already\0" as *const u8 as *const libc::c_char,
            0 as libc::c_int,
        );
        return;
    }
    ch = pack_letter(
        b"put on what?\0" as *const u8 as *const libc::c_char,
        0o200 as libc::c_int as libc::c_ushort as libc::c_int,
    ) as libc::c_short;
    if ch as libc::c_int == '\u{1b}' as i32 {
        return;
    }
    ring = get_letter_object(ch as libc::c_int);
    if ring.is_null() {
        message(
            b"no such item.\0" as *const u8 as *const libc::c_char,
            0 as libc::c_int,
        );
        return;
    }
    if (*ring).what_is as libc::c_int
        & 0o200 as libc::c_int as libc::c_ushort as libc::c_int == 0
    {
        message(
            b"that's not a ring\0" as *const u8 as *const libc::c_char,
            0 as libc::c_int,
        );
        return;
    }
    if (*ring).in_use_flags as libc::c_int
        & (0o4 as libc::c_int as libc::c_ushort as libc::c_int
            | 0o10 as libc::c_int as libc::c_ushort as libc::c_int) != 0
    {
        message(
            b"that ring is already being worn\0" as *const u8 as *const libc::c_char,
            0 as libc::c_int,
        );
        return;
    }
    if r_rings as libc::c_int == 1 as libc::c_int {
        ch = (if !(rogue.left_ring).is_null() { 'r' as i32 } else { 'l' as i32 })
            as libc::c_short;
    } else {
        message(left_or_right, 0 as libc::c_int);
        loop {
            ch = rgetchar() as libc::c_short;
            if !(ch as libc::c_int != '\u{1b}' as i32 && ch as libc::c_int != 'l' as i32
                && ch as libc::c_int != 'r' as i32 && ch as libc::c_int != '\n' as i32
                && ch as libc::c_int != '\r' as i32)
            {
                break;
            }
        }
    }
    if ch as libc::c_int != 'l' as i32 && ch as libc::c_int != 'r' as i32 {
        check_message();
        return;
    }
    if ch as libc::c_int == 'l' as i32 && !(rogue.left_ring).is_null()
        || ch as libc::c_int == 'r' as i32 && !(rogue.right_ring).is_null()
    {
        check_message();
        message(
            b"there's already a ring on that hand\0" as *const u8 as *const libc::c_char,
            0 as libc::c_int,
        );
        return;
    }
    if ch as libc::c_int == 'l' as i32 {
        do_put_on(ring, 1 as libc::c_int);
    } else {
        do_put_on(ring, 0 as libc::c_int);
    }
    ring_stats(1 as libc::c_int);
    check_message();
    get_desc(ring, desc.as_mut_ptr());
    message(desc.as_mut_ptr(), 0 as libc::c_int);
    reg_move();
    panic!("Reached end of non-void function without returning");
}
#[no_mangle]
pub unsafe extern "C" fn remove_ring() -> libc::c_int {
    let mut left: libc::c_char = 0 as libc::c_int as libc::c_char;
    let mut right: libc::c_char = 0 as libc::c_int as libc::c_char;
    let mut ch: libc::c_short = 0;
    let mut buf: [libc::c_char; 80] = [0; 80];
    let mut ring: *mut object = 0 as *mut object;
    if r_rings as libc::c_int == 0 as libc::c_int {
        inv_rings();
    } else if !(rogue.left_ring).is_null() && (rogue.right_ring).is_null() {
        left = 1 as libc::c_int as libc::c_char;
    } else if (rogue.left_ring).is_null() && !(rogue.right_ring).is_null() {
        right = 1 as libc::c_int as libc::c_char;
    } else {
        message(left_or_right, 0 as libc::c_int);
        loop {
            ch = rgetchar() as libc::c_short;
            if !(ch as libc::c_int != '\u{1b}' as i32 && ch as libc::c_int != 'l' as i32
                && ch as libc::c_int != 'r' as i32 && ch as libc::c_int != '\n' as i32
                && ch as libc::c_int != '\r' as i32)
            {
                break;
            }
        }
        left = (ch as libc::c_int == 'l' as i32) as libc::c_int as libc::c_char;
        right = (ch as libc::c_int == 'r' as i32) as libc::c_int as libc::c_char;
        check_message();
    }
    if left as libc::c_int != 0 || right as libc::c_int != 0 {
        if left != 0 {
            if !(rogue.left_ring).is_null() {
                ring = rogue.left_ring;
            } else {
                message(no_ring, 0 as libc::c_int);
            }
        } else if !(rogue.right_ring).is_null() {
            ring = rogue.right_ring;
        } else {
            message(no_ring, 0 as libc::c_int);
        }
        if (*ring).is_cursed != 0 {
            message(curse_message, 0 as libc::c_int);
        } else {
            un_put_on(ring);
            strcpy(buf.as_mut_ptr(), b"removed \0" as *const u8 as *const libc::c_char);
            get_desc(ring, buf.as_mut_ptr().offset(8 as libc::c_int as isize));
            message(buf.as_mut_ptr(), 0 as libc::c_int);
            reg_move();
        }
    }
    panic!("Reached end of non-void function without returning");
}
#[no_mangle]
pub unsafe extern "C" fn gr_ring(
    mut ring: *mut object,
    mut assign_wk: libc::c_char,
) -> libc::c_int {
    (*ring).what_is = 0o200 as libc::c_int as libc::c_ushort;
    if assign_wk != 0 {
        (*ring)
            .which_kind = get_rand(
            0 as libc::c_int,
            11 as libc::c_int - 1 as libc::c_int,
        ) as libc::c_ushort;
    }
    (*ring).class = 0 as libc::c_int as libc::c_short;
    match (*ring).which_kind as libc::c_int {
        1 => {
            (*ring).is_cursed = 1 as libc::c_int as libc::c_short;
        }
        4 | 6 => {
            loop {
                (*ring)
                    .class = (get_rand(0 as libc::c_int, 4 as libc::c_int)
                    - 2 as libc::c_int) as libc::c_short;
                if !((*ring).class as libc::c_int == 0 as libc::c_int) {
                    break;
                }
            }
            (*ring)
                .is_cursed = (((*ring).class as libc::c_int) < 0 as libc::c_int)
                as libc::c_int as libc::c_short;
        }
        7 => {
            (*ring).is_cursed = coin_toss() as libc::c_short;
        }
        _ => {}
    }
    panic!("Reached end of non-void function without returning");
}
