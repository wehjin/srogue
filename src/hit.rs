#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]
#![feature(extern_types)]
extern "C" {
    pub type ldat;
    fn waddch(_: *mut WINDOW, _: chtype) -> libc::c_int;
    fn winch(_: *mut WINDOW) -> chtype;
    fn wmove(_: *mut WINDOW, _: libc::c_int, _: libc::c_int) -> libc::c_int;
    fn sprintf(_: *mut libc::c_char, _: *const libc::c_char, _: ...) -> libc::c_int;
    static mut stdscr: *mut WINDOW;
    static mut rogue: fighter;
    static mut dungeon: [[libc::c_ushort; 80]; 24];
    static mut level_monsters: object;
    fn strcpy(_: *mut libc::c_char, _: *const libc::c_char) -> *mut libc::c_char;
    fn mon_name() -> *mut libc::c_char;
    fn is_direction() -> libc::c_char;
    fn object_at() -> *mut object;
    static mut cur_level: libc::c_short;
    static mut add_strength: libc::c_short;
    static mut ring_exp: libc::c_short;
    static mut r_rings: libc::c_short;
    static mut being_held: libc::c_char;
    static mut interrupted: libc::c_char;
    static mut wizard: libc::c_char;
    fn strlen(_: *const libc::c_char) -> libc::c_ulong;
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
#[no_mangle]
pub static mut fight_monster: *mut object = 0 as *const object as *mut object;
#[no_mangle]
pub static mut detect_monster: libc::c_char = 0;
#[no_mangle]
pub static mut hit_message: [libc::c_char; 80] = unsafe {
    *::core::mem::transmute::<
        &[u8; 80],
        &mut [libc::c_char; 80],
    >(
        b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
    )
};
#[no_mangle]
pub unsafe extern "C" fn mon_hit(
    mut monster: *mut object,
    mut other: *mut libc::c_char,
    mut flame: libc::c_char,
) -> libc::c_int {
    let mut damage: libc::c_short = 0;
    let mut hit_chance: libc::c_short = 0;
    let mut mn: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut minus: libc::c_int = 0;
    if !fight_monster.is_null() && monster != fight_monster {
        fight_monster = 0 as *mut object;
    }
    (*monster).trow = -(1 as libc::c_int) as libc::c_short;
    if cur_level as libc::c_int >= 26 as libc::c_int * 2 as libc::c_int {
        hit_chance = 100 as libc::c_int as libc::c_short;
    } else {
        hit_chance = (*monster).class;
        hit_chance = (hit_chance as libc::c_int
            - (2 as libc::c_int * rogue.exp as libc::c_int
                + 2 as libc::c_int * ring_exp as libc::c_int - r_rings as libc::c_int))
            as libc::c_short;
    }
    if wizard != 0 {
        hit_chance = (hit_chance as libc::c_int / 2 as libc::c_int) as libc::c_short;
    }
    if fight_monster.is_null() {
        interrupted = 1 as libc::c_int as libc::c_char;
    }
    mn = mon_name(monster);
    if !other.is_null() {
        hit_chance = (hit_chance as libc::c_int
            - (rogue.exp as libc::c_int + ring_exp as libc::c_int
                - r_rings as libc::c_int)) as libc::c_short;
    }
    if rand_percent(hit_chance as libc::c_int) == 0 {
        if fight_monster.is_null() {
            sprintf(
                hit_message
                    .as_mut_ptr()
                    .offset(strlen(hit_message.as_mut_ptr()) as isize),
                b"the %s misses\0" as *const u8 as *const libc::c_char,
                if !other.is_null() { other } else { mn },
            );
            message(hit_message.as_mut_ptr(), 1 as libc::c_int);
            hit_message[0 as libc::c_int as usize] = 0 as libc::c_int as libc::c_char;
        }
        return;
    }
    if fight_monster.is_null() {
        sprintf(
            hit_message.as_mut_ptr().offset(strlen(hit_message.as_mut_ptr()) as isize),
            b"the %s hit\0" as *const u8 as *const libc::c_char,
            if !other.is_null() { other } else { mn },
        );
        message(hit_message.as_mut_ptr(), 1 as libc::c_int);
        hit_message[0 as libc::c_int as usize] = 0 as libc::c_int as libc::c_char;
    }
    if (*monster).m_flags & 0o100000000 as libc::c_long as libc::c_ulong == 0 {
        damage = get_damage((*monster).damage, 1 as libc::c_int) as libc::c_short;
        if !other.is_null() {
            if flame != 0 {
                damage = (damage as libc::c_int - get_armor_class(rogue.armor))
                    as libc::c_short;
                if (damage as libc::c_int) < 0 as libc::c_int {
                    damage = 1 as libc::c_int as libc::c_short;
                }
            }
        }
        if cur_level as libc::c_int >= 26 as libc::c_int * 2 as libc::c_int {
            minus = 26 as libc::c_int * 2 as libc::c_int - cur_level as libc::c_int;
        } else {
            minus = (get_armor_class(rogue.armor) as libc::c_double * 3.00f64)
                as libc::c_int;
            minus = minus / 100 as libc::c_int * damage as libc::c_int;
        }
        damage = (damage as libc::c_int - minus as libc::c_short as libc::c_int)
            as libc::c_short;
    } else {
        let fresh0 = (*monster).identified;
        (*monster).identified = (*monster).identified + 1;
        damage = fresh0;
    }
    if wizard != 0 {
        damage = (damage as libc::c_int / 3 as libc::c_int) as libc::c_short;
    }
    if damage as libc::c_int > 0 as libc::c_int {
        rogue_damage(damage as libc::c_int, monster);
    }
    if (*monster).m_flags
        & (0o2000 as libc::c_long | 0o4000 as libc::c_long | 0o10000 as libc::c_long
            | 0o20000 as libc::c_long | 0o40000 as libc::c_long
            | 0o100000 as libc::c_long | 0o200000 as libc::c_long
            | 0o400000 as libc::c_long) as libc::c_ulong != 0
    {
        special_hit(monster);
    }
    panic!("Reached end of non-void function without returning");
}
#[no_mangle]
pub unsafe extern "C" fn rogue_hit(
    mut monster: *mut object,
    mut force_hit: libc::c_char,
) -> libc::c_int {
    let mut damage: libc::c_short = 0;
    let mut hit_chance: libc::c_short = 0;
    if !monster.is_null() {
        if check_imitator(monster) != 0 {
            return;
        }
        hit_chance = (if force_hit as libc::c_int != 0 {
            100 as libc::c_int
        } else {
            get_hit_chance(rogue.weapon)
        }) as libc::c_short;
        if wizard != 0 {
            hit_chance = (hit_chance as libc::c_int * 2 as libc::c_int) as libc::c_short;
        }
        if rand_percent(hit_chance as libc::c_int) == 0 {
            if fight_monster.is_null() {
                strcpy(
                    hit_message.as_mut_ptr(),
                    b"you miss  \0" as *const u8 as *const libc::c_char,
                );
            }
        } else {
            damage = get_weapon_damage(rogue.weapon) as libc::c_short;
            if wizard != 0 {
                damage = (damage as libc::c_int * 3 as libc::c_int) as libc::c_short;
            }
            if mon_damage(monster, damage as libc::c_int) != 0 {
                if fight_monster.is_null() {
                    strcpy(
                        hit_message.as_mut_ptr(),
                        b"you hit  \0" as *const u8 as *const libc::c_char,
                    );
                }
            }
        }
        check_gold_seeker(monster);
        wake_up(monster);
    }
    panic!("Reached end of non-void function without returning");
}
#[no_mangle]
pub unsafe extern "C" fn get_w_damage(mut obj: *mut object) -> libc::c_int {
    let mut new_damage: [libc::c_char; 12] = [0; 12];
    let mut to_hit_0: libc::c_int = 0;
    let mut damage: libc::c_int = 0;
    let mut i: libc::c_int = 0 as libc::c_int;
    if obj.is_null()
        || (*obj).what_is as libc::c_int
            != 0o2 as libc::c_int as libc::c_ushort as libc::c_int
    {
        return -(1 as libc::c_int);
    }
    to_hit_0 = get_number((*obj).damage) + (*obj).hit_enchant as libc::c_int;
    loop {
        let fresh1 = i;
        i = i + 1;
        if !(*((*obj).damage).offset(fresh1 as isize) as libc::c_int != 'd' as i32) {
            break;
        }
    }
    damage = get_number(((*obj).damage).offset(i as isize))
        + (*obj).d_enchant as libc::c_int;
    sprintf(
        new_damage.as_mut_ptr(),
        b"%dd%d\0" as *const u8 as *const libc::c_char,
        to_hit_0,
        damage,
    );
    return get_damage(new_damage.as_mut_ptr(), 1 as libc::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn lget_number(mut s: *mut libc::c_char) -> libc::c_long {
    let mut i: libc::c_long = 0 as libc::c_int as libc::c_long;
    let mut total: libc::c_long = 0 as libc::c_int as libc::c_long;
    while *s.offset(i as isize) as libc::c_int >= '0' as i32
        && *s.offset(i as isize) as libc::c_int <= '9' as i32
    {
        total = 10 as libc::c_int as libc::c_long * total
            + (*s.offset(i as isize) as libc::c_int - '0' as i32) as libc::c_long;
        i += 1;
        i;
    }
    return total;
}
#[no_mangle]
pub unsafe extern "C" fn to_hit(mut obj: *mut object) -> libc::c_int {
    if obj.is_null() {
        return 1 as libc::c_int;
    }
    return get_number((*obj).damage) + (*obj).hit_enchant as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn damage_for_strength() -> libc::c_int {
    let mut strength: libc::c_short = 0;
    strength = (rogue.str_current as libc::c_int + add_strength as libc::c_int)
        as libc::c_short;
    if strength as libc::c_int <= 6 as libc::c_int {
        return strength as libc::c_int - 5 as libc::c_int;
    }
    if strength as libc::c_int <= 14 as libc::c_int {
        return 1 as libc::c_int;
    }
    if strength as libc::c_int <= 17 as libc::c_int {
        return 3 as libc::c_int;
    }
    if strength as libc::c_int <= 18 as libc::c_int {
        return 4 as libc::c_int;
    }
    if strength as libc::c_int <= 20 as libc::c_int {
        return 5 as libc::c_int;
    }
    if strength as libc::c_int <= 21 as libc::c_int {
        return 6 as libc::c_int;
    }
    if strength as libc::c_int <= 30 as libc::c_int {
        return 7 as libc::c_int;
    }
    return 8 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn fight(mut to_the_death: libc::c_char) -> libc::c_int {
    let mut ch: libc::c_short = 0;
    let mut c: libc::c_short = 0;
    let mut row: libc::c_short = 0;
    let mut col: libc::c_short = 0;
    let mut first_miss: libc::c_char = 1 as libc::c_int as libc::c_char;
    let mut possible_damage: libc::c_short = 0;
    let mut monster: *mut object = 0 as *mut object;
    loop {
        ch = rgetchar() as libc::c_short;
        if !(is_direction(ch as libc::c_int) == 0) {
            break;
        }
        sound_bell();
        if first_miss != 0 {
            message(
                b"direction?\0" as *const u8 as *const libc::c_char,
                0 as libc::c_int,
            );
            first_miss = 0 as libc::c_int as libc::c_char;
        }
    }
    check_message();
    if ch as libc::c_int == '\u{1b}' as i32 {
        return;
    }
    row = rogue.row;
    col = rogue.col;
    get_dir_rc(ch as libc::c_int, &mut row, &mut col, 0 as libc::c_int);
    c = (if wmove(stdscr, row as libc::c_int, col as libc::c_int) == -(1 as libc::c_int)
    {
        -(1 as libc::c_int) as chtype
    } else {
        winch(stdscr)
    }) as libc::c_short;
    if (c as libc::c_int) < 'A' as i32 || c as libc::c_int > 'Z' as i32
        || can_move(
            rogue.row as libc::c_int,
            rogue.col as libc::c_int,
            row as libc::c_int,
            col as libc::c_int,
        ) == 0
    {
        message(
            b"I see no monster there\0" as *const u8 as *const libc::c_char,
            0 as libc::c_int,
        );
        return;
    }
    fight_monster = object_at(
        &mut level_monsters,
        row as libc::c_int,
        col as libc::c_int,
    );
    if fight_monster.is_null() {
        return;
    }
    if (*fight_monster).m_flags & 0o100000000 as libc::c_long as libc::c_ulong == 0 {
        possible_damage = (get_damage((*fight_monster).damage, 0 as libc::c_int)
            * 2 as libc::c_int / 3 as libc::c_int) as libc::c_short;
    } else {
        possible_damage = ((*fight_monster).identified as libc::c_int - 1 as libc::c_int)
            as libc::c_short;
    }
    while !fight_monster.is_null() {
        one_move_rogue(ch as libc::c_int, 0 as libc::c_int);
        if to_the_death == 0
            && rogue.hp_current as libc::c_int <= possible_damage as libc::c_int
            || interrupted as libc::c_int != 0
            || dungeon[row as usize][col as usize] as libc::c_int
                & 0o2 as libc::c_int as libc::c_ushort as libc::c_int == 0
        {
            fight_monster = 0 as *mut object;
        } else {
            monster = object_at(
                &mut level_monsters,
                row as libc::c_int,
                col as libc::c_int,
            );
            if monster != fight_monster {
                fight_monster = 0 as *mut object;
            }
        }
    }
    panic!("Reached end of non-void function without returning");
}
