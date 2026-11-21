#![feature(proc_macro_hygiene)]
#![allow(unused, non_snake_case, static_mut_refs, unused_unsafe, unused_must_use)]

//! Hammer Express (portage du stage Brawl « Lethal League », 2e variante du Petit Champ de bataille) :
//! version simple du mode Lethal League : un Ballon de foot d'Ultimate apparaît au centre au début du match et
//! réapparaît peu après avoir disparu. Seulement sur Hammer Express : Petit Champ de bataille dont les murs
//! invisibles sont à x ±75 (COT / Royal Court : ±117, Petit Champ normal : pas de murs).
//! v0.1

use smash::app::lua_bind::*;
use smash::app::*;
use smash::lib::lua_const::*;
use smash::lua2cpp::*;
use smash::phx::{Vector2f, Vector3f};
use smashline::*;

/// délai avant de (re)faire apparaître la balle (images)
const DELAI: i32 = 90;
/// hauteur d'apparition au centre
const HAUTEUR: f32 = 45.0;

static mut EST_HE: i32 = -1; // -1 : pas encore testé dans ce match ; 0 / 1
static mut ATTENTE: i32 = DELAI;
static mut CHEF: i32 = -1; // numéro du combattant qui gère la balle (un seul par image)
static mut SANS_CHEF: i32 = 0; // appels des autres depuis le dernier passage du chef

/// Hammer Express ? Petit Champ + mur invisible juste avant x = 75 à hauteur de perso
unsafe fn hammer_express(boma: *mut BattleObjectModuleAccessor) -> bool {
    if stage::get_stage_id() != *StageID::BattleField_S {
        return false;
    }
    let mut hit = Vector2f { x: 0.0, y: 0.0 };
    let de = Vector2f { x: 0.0, y: 25.0 };
    let droite = GroundModule::ray_check_hit_pos(boma, &de, &Vector2f { x: 100.0, y: 0.0 }, &mut hit, true);
    let x_d = hit.x;
    let gauche = GroundModule::ray_check_hit_pos(boma, &de, &Vector2f { x: -100.0, y: 0.0 }, &mut hit, true);
    droite && gauche && (x_d - 75.0).abs() < 8.0 && (hit.x + 75.0).abs() < 8.0
}

unsafe extern "C" fn ballon_frame(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let entree = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID);
    if !sv_information::is_ready_go() {
        EST_HE = -1;                                       // décompte / avant le match : tout remettre à zéro
        ATTENTE = DELAI;
        CHEF = -1;
        return;
    }
    // un seul combattant fait le travail (le chef) ; si le chef ne passe plus (éliminé), un autre le remplace
    if CHEF < 0 {
        CHEF = entree;
    }
    if entree != CHEF {
        SANS_CHEF += 1;
        if SANS_CHEF > 240 {
            CHEF = entree;
        } else {
            return;
        }
    }
    SANS_CHEF = 0;
    let st = StatusModule::status_kind(boma);
    if st == *FIGHTER_STATUS_KIND_DEAD || st == *FIGHTER_STATUS_KIND_REBIRTH || st == *FIGHTER_STATUS_KIND_STANDBY {
        return;
    }
    if EST_HE < 0 {
        EST_HE = hammer_express(boma) as i32;
    }
    if EST_HE == 0 {
        return;
    }
    if item_manager::get_num_of_active_item(*ITEM_KIND_SOCCERBALL) > 0 {
        ATTENTE = DELAI;
        return;
    }
    if ATTENTE > 0 {
        ATTENTE -= 1;
        return;
    }
    if ItemModule::is_have_item(boma, 0) {
        return;                                            // ce combattant tient déjà un objet : on réessaie
    }
    // le combattant « reçoit » la balle puis la lâche ; elle est replacée au centre
    ItemModule::have_item(boma, ItemKind(*ITEM_KIND_SOCCERBALL), 0, 0, false, false);
    let id = ItemModule::get_have_item_id(boma, 0) as u32;
    ItemModule::drop_item(boma, 0.0, 0.0, 0);
    if id != 0 && sv_battle_object::is_active(id) {
        let ib = sv_battle_object::module_accessor(id);
        PostureModule::set_pos(ib, &Vector3f { x: 0.0, y: HAUTEUR, z: 0.0 });
        KineticModule::clear_speed_all(ib);
    }
    ATTENTE = DELAI;
}

#[skyline::main(name = "ballon")]
pub fn main() {
    Agent::new("fighter").on_line(Main, ballon_frame).install();
}
