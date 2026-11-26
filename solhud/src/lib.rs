//! Jauge de Tension de Sol Badguy affichée dans le HUD (panes « sol_tension » / « sol_tg_00..14 »
//! du layout info_melee). Le plugin de Sol appelle `sol_hud_tension(entry_id, tension)` à chaque frame.
//! Même méthode que le plugin de Potemkin (13.0.4) : hook inline sur le dessin des layouts
//! (texte + 0x4b638, x0 = nn::ui2d::Layout*), puis recherche des panes par leur nom.
#![allow(static_mut_refs)]
use skyline::hooks::InlineCtx;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

const N_IMG: usize = 15;
const TIMEOUT: u64 = 600; // frames sans nouvelle de Sol avant de cacher la barre

/// niveau (0..30, comme les effets sol_tension_00..30) -> image (0..14)
const LEVEL_TO_IMG: [u8; 31] = [
    0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 11, 12, 12, 13, 13, 14,
];

/// 0 = pas de Sol dans ce HUD, sinon image + 1
static SLOT_VAL: [AtomicU32; 8] = [
    AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0),
    AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0),
];
static SLOT_SEEN: [AtomicU64; 8] = [
    AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
    AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
];
static FRAME: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy)]
struct Panes {
    root: u64,
    imgs: [u64; N_IMG],
}
static mut PANES: [Panes; 8] = [Panes { root: 0, imgs: [0; N_IMG] }; 8];
static mut CACHED_LAYOUT: u64 = 0;
static mut CACHED_ROOT: u64 = 0;

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
        SLOT_SEEN[slot].store(FRAME.load(Ordering::Relaxed), Ordering::Relaxed);
    }
}

// ---- nn::ui2d (mêmes champs que nnsdk-rs / plugin du Potemkin) ----
// Layout : +0x18 root_pane, +0x30 layout_name
// Pane   : +0x20 liste des enfants (prev, next), +0x08 lien dans la liste du parent,
//          +0x58 drapeaux (bit 0 = visible), +0xb0 nom
// Parts  : +0xe8 Layout* de la partie
unsafe fn name_is(pane: u64, name: &[u8]) -> bool {
    let p = (pane + 0xb0) as *const u8;
    for (i, c) in name.iter().enumerate() {
        if *p.add(i) != *c {
            return false;
        }
    }
    *p.add(name.len()) == 0
}

unsafe fn find(pane: u64, name: &[u8], depth: u32) -> u64 {
    if pane == 0 || depth > 32 {
        return 0;
    }
    if name_is(pane, name) {
        return pane;
    }
    let head = pane + 0x20;
    let mut node = *((head + 8) as *const u64);
    let mut guard = 0;
    while node != 0 && node != head && guard < 4096 {
        let r = find(node - 8, name, depth + 1);
        if r != 0 {
            return r;
        }
        node = *((node + 8) as *const u64);
        guard += 1;
    }
    0
}

unsafe fn set_visible(pane: u64, visible: bool) {
    if pane == 0 {
        return;
    }
    let f = (pane + 0x58) as *mut u8;
    *f = (*f & 0xFE) | (visible as u8);
}

unsafe fn refresh_cache(root: u64) {
    const PN: [&[u8]; 8] = [b"p1", b"p2", b"p3", b"p4", b"p5", b"p6", b"p7", b"p8"];
    for s in 0..8 {
        let mut p = Panes { root: 0, imgs: [0; N_IMG] };
        let part = find(root, PN[s], 0);
        if part != 0 {
            let lay = *((part + 0xe8) as *const u64);
            if lay != 0 {
                let proot = *((lay + 0x18) as *const u64);
                p.root = find(proot, b"sol_tension", 0);
                if p.root != 0 {
                    for i in 0..N_IMG {
                        let n = [b's', b'o', b'l', b'_', b't', b'g', b'_', b'0' + (i / 10) as u8, b'0' + (i % 10) as u8];
                        p.imgs[i] = find(p.root, &n, 0);
                    }
                }
            }
        }
        PANES[s] = p;
    }
}

#[skyline::hook(offset = 0x4b638, inline)]
unsafe fn layout_draw(ctx: &InlineCtx) {
    let layout = ctx.registers[0].x();
    if layout == 0 {
        return;
    }
    let name = *((layout + 0x30) as *const u64) as *const u8;
    if name.is_null() {
        return;
    }
    let want = b"info_melee\0";
    for (i, c) in want.iter().enumerate() {
        if *name.add(i) != *c {
            return;
        }
    }
    let root = *((layout + 0x18) as *const u64);
    if root == 0 {
        return;
    }
    let frame = FRAME.fetch_add(1, Ordering::Relaxed) + 1;
    if layout != CACHED_LAYOUT || root != CACHED_ROOT {
        CACHED_LAYOUT = layout;
        CACHED_ROOT = root;
        for s in 0..8 {
            SLOT_VAL[s].store(0, Ordering::Relaxed);
        }
        refresh_cache(root);
    }
    for s in 0..8 {
        let p = PANES[s];
        if p.root == 0 {
            continue;
        }
        let v = SLOT_VAL[s].load(Ordering::Relaxed);
        let seen = SLOT_SEEN[s].load(Ordering::Relaxed);
        if v == 0 || frame.saturating_sub(seen) > TIMEOUT {
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
    skyline::install_hooks!(layout_draw);
}
