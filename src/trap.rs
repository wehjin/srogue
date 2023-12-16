#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]
#![feature(extern_types)]
extern "C" {
    pub type ldat;
    fn waddch(_: *mut WINDOW, _: chtype) -> libc::c_int;
    fn wmove(_: *mut WINDOW, _: libc::c_int, _: libc::c_int) -> libc::c_int;
    static mut stdscr: *mut WINDOW;
    static mut rogue: fighter;
    static mut rooms: [room; 0];
    static mut dungeon: [[libc::c_ushort; 80]; 24];
    fn is_direction() -> libc::c_char;
    fn reg_move() -> libc::c_char;
    static mut cur_level: libc::c_short;
    static mut party_room: libc::c_short;
    static mut new_level_message: *mut libc::c_char;
    static mut interrupted: libc::c_char;
    static mut ring_exp: libc::c_short;
    static mut sustain_strength: libc::c_char;
    static mut blind: libc::c_short;
}
pub type chtype = libc::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _win_st {
    pub _cury: libc::c_short,
    pub _curx: libc::c_short,
    pub _maxy: libc::c_short,
    pub _maxx: libc::c_short,
    pub _begy: libc::c_short,
    pub _begx: libc::c_short,
    pub _flags: libc::c_short,
    pub _attrs: attr_t,
    pub _bkgd: chtype,
    pub _notimeout: libc::c_int,
    pub _clear: libc::c_int,
    pub _leaveok: libc::c_int,
    pub _scroll: libc::c_int,
    pub _idlok: libc::c_int,
    pub _idcok: libc::c_int,
    pub _immed: libc::c_int,
    pub _sync: libc::c_int,
    pub _use_keypad: libc::c_int,
    pub _delay: libc::c_int,
    pub _line: *mut ldat,
    pub _regtop: libc::c_short,
    pub _regbottom: libc::c_short,
    pub _parx: libc::c_int,
    pub _pary: libc::c_int,
    pub _parent: *mut WINDOW,
    pub _pad: pdat,
    pub _yoffset: libc::c_short,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct pdat {
    pub _pad_y: libc::c_short,
    pub _pad_x: libc::c_short,
    pub _pad_top: libc::c_short,
    pub _pad_left: libc::c_short,
    pub _pad_bottom: libc::c_short,
    pub _pad_right: libc::c_short,
}
pub type WINDOW = _win_st;
pub type attr_t = chtype;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dr {
    pub oth_room: libc::c_short,
    pub oth_row: libc::c_short,
    pub oth_col: libc::c_short,
    pub door_row: libc::c_short,
    pub door_col: libc::c_short,
}
pub type door = dr;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct rm {
    pub bottom_row: libc::c_char,
    pub right_col: libc::c_char,
    pub left_col: libc::c_char,
    pub top_row: libc::c_char,
    pub doors: [door; 4],
    pub is_room: libc::c_ushort,
}
pub type room = rm;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tr {
    pub trap_type: libc::c_short,
    pub trap_row: libc::c_short,
    pub trap_col: libc::c_short,
}
pub type trap = tr;
#[no_mangle]
pub static mut traps: [trap; 10] = [tr {
    trap_type: 0,
    trap_row: 0,
    trap_col: 0,
}; 10];
#[no_mangle]
pub static mut trap_door: libc::c_char = 0 as libc::c_int as libc::c_char;
#[no_mangle]
pub static mut bear_trap: libc::c_short = 0 as libc::c_int as libc::c_short;
#[no_mangle]
pub static mut trap_strings: [*mut libc::c_char; 12] = [
    b"trap door\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
    b"you fell down a trap\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
    b"bear trap\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
    b"you are caught in a bear trap\0" as *const u8 as *const libc::c_char
        as *mut libc::c_char,
    b"teleport trap\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
    b"teleport\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
    b"poison dart trap\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
    b"a small dart just hit you in the shoulder\0" as *const u8 as *const libc::c_char
        as *mut libc::c_char,
    b"sleeping gas trap\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
    b"a strange white mist envelops you and you fall asleep\0" as *const u8
        as *const libc::c_char as *mut libc::c_char,
    b"rust trap\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
    b"a gush of water hits you on the head\0" as *const u8 as *const libc::c_char
        as *mut libc::c_char,
];
#[no_mangle]
pub unsafe extern "C" fn trap_at(
    mut row: libc::c_int,
    mut col: libc::c_int,
) -> libc::c_int {
    let mut i: libc::c_short = 0;
    i = 0 as libc::c_int as libc::c_short;
    while (i as libc::c_int) < 10 as libc::c_int
        && traps[i as usize].trap_type as libc::c_int != -(1 as libc::c_int)
    {
        if traps[i as usize].trap_row as libc::c_int == row
            && traps[i as usize].trap_col as libc::c_int == col
        {
            return traps[i as usize].trap_type as libc::c_int;
        }
        i += 1;
        i;
    }
    return -(1 as libc::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn trap_player(
    mut row: libc::c_short,
    mut col: libc::c_short,
) -> libc::c_int {
    let mut t: libc::c_short = 0;
    t = trap_at(row as libc::c_int, col as libc::c_int) as libc::c_short;
    if t as libc::c_int == -(1 as libc::c_int) {
        return;
    }
    dungeon[row
        as usize][col
        as usize] = (dungeon[row as usize][col as usize] as libc::c_int
        & !(0o1000 as libc::c_int as libc::c_ushort as libc::c_int)) as libc::c_ushort;
    if rand_percent(rogue.exp as libc::c_int + ring_exp as libc::c_int) != 0 {
        message(
            b"the trap failed\0" as *const u8 as *const libc::c_char,
            1 as libc::c_int,
        );
        return;
    }
    match t as libc::c_int {
        0 => {
            trap_door = 1 as libc::c_int as libc::c_char;
            new_level_message = trap_strings[(t as libc::c_int * 2 as libc::c_int
                + 1 as libc::c_int) as usize];
        }
        1 => {
            message(
                trap_strings[(t as libc::c_int * 2 as libc::c_int + 1 as libc::c_int)
                    as usize],
                1 as libc::c_int,
            );
            bear_trap = get_rand(4 as libc::c_int, 7 as libc::c_int) as libc::c_short;
        }
        2 => {
            if wmove(stdscr, rogue.row as libc::c_int, rogue.col as libc::c_int)
                == -(1 as libc::c_int)
            {
                -(1 as libc::c_int);
            } else {
                waddch(stdscr, '^' as i32 as chtype);
            };
            tele();
        }
        3 => {
            message(
                trap_strings[(t as libc::c_int * 2 as libc::c_int + 1 as libc::c_int)
                    as usize],
                1 as libc::c_int,
            );
            rogue
                .hp_current = (rogue.hp_current as libc::c_int
                - get_damage(
                    b"1d6\0" as *const u8 as *const libc::c_char,
                    1 as libc::c_int,
                )) as libc::c_short;
            if rogue.hp_current as libc::c_int <= 0 as libc::c_int {
                rogue.hp_current = 0 as libc::c_int as libc::c_short;
            }
            if sustain_strength == 0 && rand_percent(40 as libc::c_int) != 0
                && rogue.str_current as libc::c_int >= 3 as libc::c_int
            {
                rogue.str_current -= 1;
                rogue.str_current;
            }
            print_stats(0o4 as libc::c_int | 0o10 as libc::c_int);
            if rogue.hp_current as libc::c_int <= 0 as libc::c_int {
                killed_by(0 as *mut object, 3 as libc::c_int);
            }
        }
        4 => {
            message(
                trap_strings[(t as libc::c_int * 2 as libc::c_int + 1 as libc::c_int)
                    as usize],
                1 as libc::c_int,
            );
            take_a_nap();
        }
        5 => {
            message(
                trap_strings[(t as libc::c_int * 2 as libc::c_int + 1 as libc::c_int)
                    as usize],
                1 as libc::c_int,
            );
            rust(0 as *mut object);
        }
        _ => {}
    }
    panic!("Reached end of non-void function without returning");
}
#[no_mangle]
pub unsafe extern "C" fn add_traps() -> libc::c_int {
    let mut i: libc::c_short = 0;
    let mut n: libc::c_short = 0;
    let mut tries: libc::c_short = 0 as libc::c_int as libc::c_short;
    let mut row: libc::c_short = 0;
    let mut col: libc::c_short = 0;
    if cur_level as libc::c_int <= 2 as libc::c_int {
        n = 0 as libc::c_int as libc::c_short;
    } else if cur_level as libc::c_int <= 7 as libc::c_int {
        n = get_rand(0 as libc::c_int, 2 as libc::c_int) as libc::c_short;
    } else if cur_level as libc::c_int <= 11 as libc::c_int {
        n = get_rand(1 as libc::c_int, 2 as libc::c_int) as libc::c_short;
    } else if cur_level as libc::c_int <= 16 as libc::c_int {
        n = get_rand(2 as libc::c_int, 3 as libc::c_int) as libc::c_short;
    } else if cur_level as libc::c_int <= 21 as libc::c_int {
        n = get_rand(2 as libc::c_int, 4 as libc::c_int) as libc::c_short;
    } else if cur_level as libc::c_int <= 26 as libc::c_int + 2 as libc::c_int {
        n = get_rand(3 as libc::c_int, 5 as libc::c_int) as libc::c_short;
    } else {
        n = get_rand(5 as libc::c_int, 10 as libc::c_int) as libc::c_short;
    }
    i = 0 as libc::c_int as libc::c_short;
    while (i as libc::c_int) < n as libc::c_int {
        traps[i as usize]
            .trap_type = get_rand(0 as libc::c_int, 6 as libc::c_int - 1 as libc::c_int)
            as libc::c_short;
        if i as libc::c_int == 0 as libc::c_int
            && party_room as libc::c_int != -(1 as libc::c_int)
        {
            loop {
                row = get_rand(
                    (*rooms.as_mut_ptr().offset(party_room as isize)).top_row
                        as libc::c_int + 1 as libc::c_int,
                    (*rooms.as_mut_ptr().offset(party_room as isize)).bottom_row
                        as libc::c_int - 1 as libc::c_int,
                ) as libc::c_short;
                col = get_rand(
                    (*rooms.as_mut_ptr().offset(party_room as isize)).left_col
                        as libc::c_int + 1 as libc::c_int,
                    (*rooms.as_mut_ptr().offset(party_room as isize)).right_col
                        as libc::c_int - 1 as libc::c_int,
                ) as libc::c_short;
                tries += 1;
                tries;
                if !((dungeon[row as usize][col as usize] as libc::c_int
                    & (0o1 as libc::c_int as libc::c_ushort as libc::c_int
                        | 0o4 as libc::c_int as libc::c_ushort as libc::c_int
                        | 0o400 as libc::c_int as libc::c_ushort as libc::c_int
                        | 0o200 as libc::c_int as libc::c_ushort as libc::c_int) != 0
                    || dungeon[row as usize][col as usize] as libc::c_int
                        == 0 as libc::c_int as libc::c_ushort as libc::c_int)
                    && (tries as libc::c_int) < 15 as libc::c_int)
                {
                    break;
                }
            }
            if tries as libc::c_int >= 15 as libc::c_int {
                gr_row_col(
                    &mut row,
                    &mut col,
                    0o100 as libc::c_int as libc::c_ushort as libc::c_int
                        | 0o2 as libc::c_int as libc::c_ushort as libc::c_int,
                );
            }
        } else {
            gr_row_col(
                &mut row,
                &mut col,
                0o100 as libc::c_int as libc::c_ushort as libc::c_int
                    | 0o2 as libc::c_int as libc::c_ushort as libc::c_int,
            );
        }
        traps[i as usize].trap_row = row;
        traps[i as usize].trap_col = col;
        dungeon[row
            as usize][col
            as usize] = (dungeon[row as usize][col as usize] as libc::c_int
            | (0o400 as libc::c_int as libc::c_ushort as libc::c_int
                | 0o1000 as libc::c_int as libc::c_ushort as libc::c_int))
            as libc::c_ushort;
        i += 1;
        i;
    }
    panic!("Reached end of non-void function without returning");
}
#[no_mangle]
pub unsafe extern "C" fn id_trap() -> libc::c_int {
    let mut dir: libc::c_short = 0;
    let mut row: libc::c_short = 0;
    let mut col: libc::c_short = 0;
    let mut t: libc::c_short = 0;
    message(b"direction? \0" as *const u8 as *const libc::c_char, 0 as libc::c_int);
    loop {
        dir = rgetchar() as libc::c_short;
        if !(is_direction(dir as libc::c_int) == 0) {
            break;
        }
        sound_bell();
    }
    check_message();
    if dir as libc::c_int == '\u{1b}' as i32 {
        return;
    }
    row = rogue.row;
    col = rogue.col;
    get_dir_rc(dir as libc::c_int, &mut row, &mut col, 0 as libc::c_int);
    if dungeon[row as usize][col as usize] as libc::c_int
        & 0o400 as libc::c_int as libc::c_ushort as libc::c_int != 0
        && dungeon[row as usize][col as usize] as libc::c_int
            & 0o1000 as libc::c_int as libc::c_ushort as libc::c_int == 0
    {
        t = trap_at(row as libc::c_int, col as libc::c_int) as libc::c_short;
        message(
            trap_strings[(t as libc::c_int * 2 as libc::c_int) as usize],
            0 as libc::c_int,
        );
    } else {
        message(
            b"no trap there\0" as *const u8 as *const libc::c_char,
            0 as libc::c_int,
        );
    }
    panic!("Reached end of non-void function without returning");
}
#[no_mangle]
pub unsafe extern "C" fn show_traps() -> libc::c_int {
    let mut i: libc::c_short = 0;
    let mut j: libc::c_short = 0;
    i = 0 as libc::c_int as libc::c_short;
    while (i as libc::c_int) < 24 as libc::c_int {
        j = 0 as libc::c_int as libc::c_short;
        while (j as libc::c_int) < 80 as libc::c_int {
            if dungeon[i as usize][j as usize] as libc::c_int
                & 0o400 as libc::c_int as libc::c_ushort as libc::c_int != 0
            {
                if wmove(stdscr, i as libc::c_int, j as libc::c_int)
                    == -(1 as libc::c_int)
                {
                    -(1 as libc::c_int);
                } else {
                    waddch(stdscr, '^' as i32 as chtype);
                };
            }
            j += 1;
            j;
        }
        i += 1;
        i;
    }
    panic!("Reached end of non-void function without returning");
}
#[no_mangle]
pub unsafe extern "C" fn search(
    mut n: libc::c_short,
    mut is_auto: libc::c_char,
) -> libc::c_int {
    let mut s: libc::c_short = 0;
    let mut i: libc::c_short = 0;
    let mut j: libc::c_short = 0;
    let mut row: libc::c_short = 0;
    let mut col: libc::c_short = 0;
    let mut t: libc::c_short = 0;
    let mut shown: libc::c_short = 0 as libc::c_int as libc::c_short;
    let mut found: libc::c_short = 0 as libc::c_int as libc::c_short;
    static mut reg_search: libc::c_char = 0;
    i = -(1 as libc::c_int) as libc::c_short;
    while i as libc::c_int <= 1 as libc::c_int {
        j = -(1 as libc::c_int) as libc::c_short;
        while j as libc::c_int <= 1 as libc::c_int {
            row = (rogue.row as libc::c_int + i as libc::c_int) as libc::c_short;
            col = (rogue.col as libc::c_int + j as libc::c_int) as libc::c_short;
            if !((row as libc::c_int) < 1 as libc::c_int
                || row as libc::c_int >= 24 as libc::c_int - 1 as libc::c_int
                || (col as libc::c_int) < 0 as libc::c_int
                || col as libc::c_int >= 80 as libc::c_int)
            {
                if dungeon[row as usize][col as usize] as libc::c_int
                    & 0o1000 as libc::c_int as libc::c_ushort as libc::c_int != 0
                {
                    found += 1;
                    found;
                }
            }
            j += 1;
            j;
        }
        i += 1;
        i;
    }
    s = 0 as libc::c_int as libc::c_short;
    while (s as libc::c_int) < n as libc::c_int {
        i = -(1 as libc::c_int) as libc::c_short;
        while i as libc::c_int <= 1 as libc::c_int {
            j = -(1 as libc::c_int) as libc::c_short;
            while j as libc::c_int <= 1 as libc::c_int {
                row = (rogue.row as libc::c_int + i as libc::c_int) as libc::c_short;
                col = (rogue.col as libc::c_int + j as libc::c_int) as libc::c_short;
                if !((row as libc::c_int) < 1 as libc::c_int
                    || row as libc::c_int >= 24 as libc::c_int - 1 as libc::c_int
                    || (col as libc::c_int) < 0 as libc::c_int
                    || col as libc::c_int >= 80 as libc::c_int)
                {
                    if dungeon[row as usize][col as usize] as libc::c_int
                        & 0o1000 as libc::c_int as libc::c_ushort as libc::c_int != 0
                    {
                        if rand_percent(
                            17 as libc::c_int
                                + (rogue.exp as libc::c_int + ring_exp as libc::c_int),
                        ) != 0
                        {
                            dungeon[row
                                as usize][col
                                as usize] = (dungeon[row as usize][col as usize]
                                as libc::c_int
                                & !(0o1000 as libc::c_int as libc::c_ushort as libc::c_int))
                                as libc::c_ushort;
                            if blind == 0
                                && (row as libc::c_int != rogue.row as libc::c_int
                                    || col as libc::c_int != rogue.col as libc::c_int)
                            {
                                if wmove(stdscr, row as libc::c_int, col as libc::c_int)
                                    == -(1 as libc::c_int)
                                {
                                    -(1 as libc::c_int);
                                } else {
                                    waddch(
                                        stdscr,
                                        get_dungeon_char(row as libc::c_int, col as libc::c_int)
                                            as chtype,
                                    );
                                };
                            }
                            shown += 1;
                            shown;
                            if dungeon[row as usize][col as usize] as libc::c_int
                                & 0o400 as libc::c_int as libc::c_ushort as libc::c_int != 0
                            {
                                t = trap_at(row as libc::c_int, col as libc::c_int)
                                    as libc::c_short;
                                message(
                                    trap_strings[(t as libc::c_int * 2 as libc::c_int)
                                        as usize],
                                    1 as libc::c_int,
                                );
                            }
                        }
                    }
                    if shown as libc::c_int == found as libc::c_int
                        && found as libc::c_int > 0 as libc::c_int
                        || interrupted as libc::c_int != 0
                    {
                        return;
                    }
                }
                j += 1;
                j;
            }
            i += 1;
            i;
        }
        if is_auto == 0
            && {
                reg_search = (reg_search == 0) as libc::c_int as libc::c_char;
                reg_search as libc::c_int != 0
            }
        {
            reg_move();
        }
        s += 1;
        s;
    }
    panic!("Reached end of non-void function without returning");
}
