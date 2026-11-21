#![feature(proc_macro_hygiene)]
#![allow(unused, non_snake_case, static_mut_refs, unused_unsafe, unused_must_use)]

//! Hammer Express (portage du stage Brawl « Lethal League », 2e variante du Petit Champ de bataille) : le mode
//! Lethal League recréé d'après la vidéo du mod Brawl :
//! - la balle (Ballon de foot d'Ultimate) attend immobile en l'air au centre ;
//! - frappée par une attaque, elle part tout droit, sans gravité ni frottement, et rebondit sur le sol, les murs
//!   et le plafond (±75, 0, 81) sans jamais s'arrêter ; chaque coup la fait accélérer (et la fige un instant) ;
//! - elle appartient au dernier qui l'a frappée : si elle touche un autre combattant qui ne la frappe pas,
//!   il est éliminé d'un coup (sauf s'il bouclier : elle rebondit) ; la balle revient alors au centre.
//! Seulement sur Hammer Express : Petit Champ de bataille dont les murs invisibles sont à x ±75.
//! v0.1 : balle d'Ultimate normale au centre ; v0.2 : comportement Lethal League

use smash::app::lua_bind::*;
use smash::app::*;
use smash::lib::lua_const::*;
use smash::lua2cpp::*;
use smash::phx::{Vector2f, Vector3f};
use smashline::*;

const DELAI: i32 = 90; // avant de (re)faire apparaître la balle
const CENTRE: (f32, f32) = (0.0, 45.0);
const R: f32 = 4.5; // rayon de la balle
const MUR: f32 = 75.0;
const SOL: f32 = 0.0;
const PLAFOND: f32 = 81.4; // plafond de la boîte de Brawl (pour la balle seulement)
const V0: f32 = 1.6; // vitesse après le premier coup (unités / image)
const ACCEL: f32 = 1.22; // multiplicateur à chaque coup
const VMAX: f32 = 11.0;
const CORPS: f32 = 7.5; // rayon de contact d'un combattant (autour du milieu du corps)

static mut EST_HE: i32 = -1;
static mut ATTENTE: i32 = DELAI;
static mut CHEF: i32 = -1;
static mut SANS_CHEF: i32 = 0;

static mut BALLE: u32 = 0;
static mut POS: (f32, f32) = CENTRE;
static mut DIR: (f32, f32) = (0.0, 0.0);
static mut VIT: f32 = 0.0;
static mut PROPRIO: i32 = -1;
static mut FIGE: i32 = 0;
static mut GRACE: i32 = 0; // images sans contact mortel après un coup / un rebond sur bouclier

unsafe fn boma_de(id: u32) -> Option<*mut BattleObjectModuleAccessor> {
    if id == 0 || id == *BATTLE_OBJECT_ID_INVALID as u32 || !sv_battle_object::is_active(id) {
        return None;
    }
    let b = sv_battle_object::module_accessor(id);
    if b.is_null() { None } else { Some(b) }
}

unsafe fn hammer_express(boma: *mut BattleObjectModuleAccessor) -> bool {
    if stage::get_stage_id() != *StageID::BattleField_S {
        return false;
    }
    let mut hit = Vector2f { x: 0.0, y: 0.0 };
    let de = Vector2f { x: 0.0, y: 25.0 };
    let droite = GroundModule::ray_check_hit_pos(boma, &de, &Vector2f { x: 100.0, y: 0.0 }, &mut hit, true);
    let x_d = hit.x;
    let gauche = GroundModule::ray_check_hit_pos(boma, &de, &Vector2f { x: -100.0, y: 0.0 }, &mut hit, true);
    droite && gauche && (x_d - MUR).abs() < 8.0 && (hit.x + MUR).abs() < 8.0
}

unsafe fn remettre_au_centre() {
    POS = CENTRE;
    DIR = (0.0, 0.0);
    VIT = 0.0;
    PROPRIO = -1;
    FIGE = 0;
    GRACE = 0;
}

unsafe fn apparaitre(boma: *mut BattleObjectModuleAccessor) {
    if ItemModule::is_have_item(boma, 0) {
        return;
    }
    ItemModule::have_item(boma, ItemKind(*ITEM_KIND_SOCCERBALL), 0, 0, false, false);
    let id = ItemModule::get_have_item_id(boma, 0) as u32;
    ItemModule::drop_item(boma, 0.0, 0.0, 0);
    BALLE = id;
    remettre_au_centre();
}

unsafe fn milieu(o: *mut BattleObjectModuleAccessor) -> (f32, f32) {
    let p = PostureModule::pos(o);
    ((*p).x, (*p).y + 8.0)
}

/// le combattant le plus proche de la balle (celui qui vient de la frapper)
unsafe fn plus_proche(x: f32, y: f32) -> i32 {
    let mut best = (-1, 1.0e9f32);
    for e in 0..8 {
        let Some(o) = boma_de(Fighter::get_id_from_entry_id(e)) else { continue };
        let st = StatusModule::status_kind(o);
        if st == *FIGHTER_STATUS_KIND_DEAD || st == *FIGHTER_STATUS_KIND_STANDBY {
            continue;
        }
        let (ox, oy) = milieu(o);
        let d = (ox - x).powi(2) + (oy - y).powi(2);
        if d < best.1 {
            best = (e, d);
        }
    }
    best.0
}

unsafe fn physique() {
    let Some(ib) = boma_de(BALLE) else { return };
    // la balle a-t-elle été déplacée par le jeu (coup reçu) depuis l'image précédente ?
    let p = PostureModule::pos(ib);
    let (dx, dy) = ((*p).x - POS.0, (*p).y - POS.1);
    let d = (dx * dx + dy * dy).sqrt();
    if d > 0.6 && FIGE == 0 {
        DIR = (dx / d, dy / d);
        VIT = if VIT <= 0.0 { V0 } else { (VIT * ACCEL).min(VMAX) };
        PROPRIO = plus_proche(POS.0, POS.1);
        FIGE = 3 + (VIT * 1.5) as i32; // arrêt sur image, plus long quand elle va vite
        GRACE = 6;
    }
    if FIGE > 0 {
        FIGE -= 1;
    } else if VIT > 0.0 {
        // avance en ligne droite, rebonds parfaits
        let mut x = POS.0 + DIR.0 * VIT;
        let mut y = POS.1 + DIR.1 * VIT;
        if x > MUR - R { x = 2.0 * (MUR - R) - x; DIR.0 = -DIR.0.abs(); }
        if x < -MUR + R { x = 2.0 * (-MUR + R) - x; DIR.0 = DIR.0.abs(); }
        if y < SOL + R { y = 2.0 * (SOL + R) - y; DIR.1 = DIR.1.abs(); }
        if y > PLAFOND - R { y = 2.0 * (PLAFOND - R) - y; DIR.1 = -DIR.1.abs(); }
        POS = (x, y);
    }
    PostureModule::set_pos(ib, &Vector3f { x: POS.0, y: POS.1, z: 0.0 });
    KineticModule::clear_speed_all(ib);
    AttackModule::clear_all(ib); // la balle ne blesse pas d'elle-même : c'est nous qui gérons le contact
    if GRACE > 0 {
        GRACE -= 1;
        return;
    }
    if VIT <= 0.0 || FIGE > 0 {
        return;
    }
    // contact avec un adversaire
    for e in 0..8 {
        if e == PROPRIO {
            continue;
        }
        let Some(o) = boma_de(Fighter::get_id_from_entry_id(e)) else { continue };
        let st = StatusModule::status_kind(o);
        if st == *FIGHTER_STATUS_KIND_DEAD || st == *FIGHTER_STATUS_KIND_STANDBY || st == *FIGHTER_STATUS_KIND_REBIRTH {
            continue;
        }
        let (ox, oy) = milieu(o);
        let dd = ((ox - POS.0).powi(2) + (oy - POS.1).powi(2)).sqrt();
        if dd > R + CORPS {
            continue;
        }
        if AttackModule::is_attack(o, 0, false) || AttackModule::is_attack(o, 1, false) {
            continue; // il est en train de frapper : le coup sera détecté à l'image suivante
        }
        if st == *FIGHTER_STATUS_KIND_GUARD || st == *FIGHTER_STATUS_KIND_GUARD_ON || st == *FIGHTER_STATUS_KIND_GUARD_DAMAGE {
            DIR = (-DIR.0, DIR.1); // bouclier : elle rebondit
            GRACE = 10;
            continue;
        }
        // éliminé d'un coup
        StatusModule::change_status_force(o, *FIGHTER_STATUS_KIND_DEAD, false);
        remettre_au_centre();
        return;
    }
}

unsafe extern "C" fn ballon_frame(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let entree = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID);
    if !sv_information::is_ready_go() {
        EST_HE = -1;
        ATTENTE = DELAI;
        CHEF = -1;
        BALLE = 0;
        return;
    }
    if CHEF < 0 {
        CHEF = entree;
    }
    if entree != CHEF {
        SANS_CHEF += 1;
        if SANS_CHEF > 240 { CHEF = entree; } else { return; }
    }
    SANS_CHEF = 0;
    if EST_HE < 0 {
        EST_HE = hammer_express(boma) as i32;
    }
    if EST_HE == 0 {
        return;
    }
    if boma_de(BALLE).is_some() {
        ATTENTE = DELAI;
        physique();
        return;
    }
    if ATTENTE > 0 {
        ATTENTE -= 1;
        return;
    }
    let st = StatusModule::status_kind(boma);
    if st == *FIGHTER_STATUS_KIND_DEAD || st == *FIGHTER_STATUS_KIND_REBIRTH || st == *FIGHTER_STATUS_KIND_STANDBY {
        return;
    }
    apparaitre(boma);
}

#[skyline::main(name = "ballon")]
pub fn main() {
    Agent::new("fighter").on_line(Main, ballon_frame).install();
}
