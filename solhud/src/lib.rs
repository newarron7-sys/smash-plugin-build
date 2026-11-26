//! Jauge de Tension de Sol Badguy affichée dans le HUD (panes « sol_tension » / « sol_tg_00..14 »
//! du layout info_melee). Le plugin de Sol appelle `sol_hud_tension(entry_id, tension)` à chaque frame.
//! Offsets du jeu 13.0.4 (mêmes que HDR 0.49.11 / mode HDR 4 persos).
#![allow(static_mut_refs)]
use skyline::hooks::{getRegionAddress, InlineCtx, Region};
use std::sync::atomic::{AtomicU32, Ordering};

const GET_PANE_BY_NAME: usize = 0x3776360;
const MENU_MODE: usize = 0x53040f0;
const N_IMG: usize = 15;

/// niveau (0..30, comme les effets sol_tension_00..30) -> image (0..14)
const LEVEL_TO_IMG: [u8; 31] = [
    0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 11, 12, 12, 13, 13, 14,
];

/// 0 = pas de Sol dans ce HUD, sinon image + 1
static SLOT_VAL: [AtomicU32; 8] = [
    AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0),
    AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0),
];

#[derive(Clone, Copy)]
struct Panes {
    root: u64,
    imgs: [u64; N_IMG],
}
static mut PANES: [Panes; 8] = [Panes { root: 0, imgs: [0; N_IMG] }; 8];

extern "C" {
    #[link_name = "\u{1}_ZN3app8lua_bind38FighterManager__get_fighter_entry_implEPNS_14FighterManagerENS_14FighterEntryIDE"]
    fn get_fighter_entry(manager: *mut u64, entry_id: u32) -> *mut u8;
    #[link_name = "\u{1}_ZN3lib9SingletonIN3app14FighterManagerEE9instance_E"]
    static FIGHTER_MANAGER: *mut u64;
}

fn level_of(t: u32) -> usize {
    if t > 0x5db {
        30
    } else if t > 0x257 {
        ((t - 600) / 60 + 15) as usize
    } else {
        (t / 40) as usize
    }
}

/// Appelée par le plugin de Sol (via nn::ro::LookupSymbol), sur le thread du combattant.
#[no_mangle]
pub extern "C" fn sol_hud_tension(entry_id: u32, tension: u32) {
    if entry_id >= 8 {
        return;
    }
    unsafe {
        let mgr = FIGHTER_MANAGER;
        if mgr.is_null() {
            return;
        }
        // HUD p1..p8 attribués dans l'ordre des combattants présents
        let mut slot = 0usize;
        for n in 0..entry_id {
            if !get_fighter_entry(mgr, n).is_null() {
                slot += 1;
            }
        }
        if slot >= 8 {
            return;
        }
        let lvl = level_of(tension).min(30);
        SLOT_VAL[slot].store(LEVEL_TO_IMG[lvl] as u32 + 1, Ordering::Relaxed);
    }
}

fn pane_valid(pane: u64) -> bool {
    unsafe { pane != 0 && *(pane as *const u64) != 0 }
}

fn set_visible(pane: u64, visible: bool) {
    if !pane_valid(pane) {
        return;
    }
    unsafe {
        let internal = *(pane as *const u64) as *mut u8;
        *internal.add(0x58) = (*internal.add(0x58) & 0xFE) | (visible as u8);
    }
}

unsafe fn get_pane(layout: u64, name: &[u8]) -> u64 {
    let func: extern "C" fn(u64, *const u8, ...) -> [u64; 4] =
        std::mem::transmute((getRegionAddress(Region::Text) as *mut u8).add(GET_PANE_BY_NAME));
    func(layout, name.as_ptr())[1]
}

#[skyline::hook(offset = 0x1b6cc08, inline)]
unsafe fn set_info_alpha(ctx: &InlineCtx) {
    let layout_udata = ctx.registers[0].x();
    let layout_view = *(layout_udata as *const u64).add(1);
    let layout_pane = *(layout_view as *const u64).add(3);
    let ui2d_pane = *(layout_pane as *const u64);
    let name_ptr = (ui2d_pane as *const u8).add(0xb0);
    let len = skyline::libc::strlen(name_ptr as _);
    let name = std::slice::from_raw_parts(name_ptr, len);
    let index = match name {
        b"p1" => 0,
        b"p2" => 1,
        b"p3" => 2,
        b"p4" => 3,
        b"p5" => 4,
        b"p6" => 5,
        b"p7" => 6,
        b"p8" => 7,
        _ => return,
    };
    let mut p = Panes { root: 0, imgs: [0; N_IMG] };
    p.root = get_pane(layout_udata, b"sol_tension\0");
    if p.root != 0 {
        for i in 0..N_IMG {
            let n = [b's', b'o', b'l', b'_', b't', b'g', b'_', b'0' + (i / 10) as u8, b'0' + (i % 10) as u8, 0];
            p.imgs[i] = get_pane(layout_udata, &n);
        }
    }
    PANES[index] = p;
    SLOT_VAL[index].store(0, Ordering::Relaxed);
}

#[skyline::hook(offset = 0x138a710, inline)]
unsafe fn hud_update(_ctx: &InlineCtx) {
    let mode = (getRegionAddress(Region::Text) as u64 + MENU_MODE as u64) as *const u64;
    if [0x6020000u64, 0x4050000].contains(&*mode) {
        return;
    }
    for s in 0..8 {
        let p = PANES[s];
        if !pane_valid(p.root) {
            continue;
        }
        let v = SLOT_VAL[s].load(Ordering::Relaxed);
        if v == 0 {
            set_visible(p.root, false);
            continue;
        }
        let img = (v - 1) as usize;
        set_visible(p.root, true);
        for i in 0..N_IMG {
            set_visible(p.imgs[i], i == img);
        }
    }
}

#[skyline::main(name = "sol_hud")]
pub fn main() {
    skyline::install_hooks!(set_info_alpha, hud_update);
}
