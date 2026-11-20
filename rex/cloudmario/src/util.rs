//! Remplaçants minimaux des outils d'Ultimate S utilisés par src/mario.
use smash::app::lua_bind::*;
use smash::app::BattleObjectModuleAccessor;
use smash::lib::lua_const::*;
use smash::phx::Vector2f;
use std::collections::HashMap;

pub const SLOT: usize = 120;

pub fn get_marked_costumes(_dossier: &str, _marqueur: &str) -> Vec<usize> {
    vec![SLOT]
}

pub(crate) unsafe fn ray_check_pos(boma: &mut BattleObjectModuleAccessor, x_distance: f32, y_distance: f32, ignore_plat: bool) -> u64 {
    GroundModule::ray_check(boma, &Vector2f { x: PostureModule::pos_x(boma), y: PostureModule::pos_y(boma) }, &Vector2f { x: x_distance, y: y_distance }, ignore_plat)
}

// ------------------------------------------------------------------ variables par combattant
static mut INTS: Option<HashMap<(u32, i32), i32>> = None;
pub struct VariableModule;
impl VariableModule {
    pub unsafe fn get_int(boma: *mut BattleObjectModuleAccessor, id: i32) -> i32 {
        *INTS.get_or_insert_with(HashMap::new).get(&((*boma).battle_object_id, id)).unwrap_or(&0)
    }
    pub unsafe fn set_int(boma: *mut BattleObjectModuleAccessor, value: i32, id: i32) {
        INTS.get_or_insert_with(HashMap::new).insert(((*boma).battle_object_id, id), value);
    }
    pub unsafe fn inc_int(boma: *mut BattleObjectModuleAccessor, id: i32) {
        let v = Self::get_int(boma, id) + 1;
        Self::set_int(boma, v, id);
    }
}

// ------------------------------------------------------------------ side B une fois par saut
pub static mut CAN_SIDEB: [bool; 8] = [false; 8];
#[macro_export]
macro_rules! transition_set {
    ($entry:expr, can_sideb) => { unsafe { $crate::util::CAN_SIDEB[($entry as usize).min(7)] = true; } };
}
#[macro_export]
macro_rules! transition_reset {
    ($entry:expr, can_sideb) => { unsafe { $crate::util::CAN_SIDEB[($entry as usize).min(7)] = false; } };
}

/// chaque image (frame.rs) : side B déjà utilisé pendant ce saut -> plus de side B jusqu'au sol
pub unsafe fn bloquer_sideb(boma: *mut BattleObjectModuleAccessor) {
    let e = (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7);
    if CAN_SIDEB[e] && StatusModule::status_kind(boma) != *FIGHTER_STATUS_KIND_SPECIAL_S {
        WorkModule::unable_transition_term(boma, *FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_SPECIAL_S);
    }
}

// ------------------------------------------------------------------ jump_speed_x_mul 1,2 (comme Ultimate S)
// Aucun hook de paramètres ici : libparam_config (installé chez lui) hooke déjà ces fonctions, et son README demande
// qu'aucun autre plugin ne le fasse (v7 : plantage au début de chaque match). Le saut x1,2 passe par le fichier
// config_param.toml à la racine du mod, que libparam_config lit lui-même.
pub fn install() {}
