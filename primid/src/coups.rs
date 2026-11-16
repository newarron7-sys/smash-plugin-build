//! Primid v0.22 : coups de REX qui demandent du code (pas seulement des boîtes de coup).
//! - Smash haut : il saute en frappant avec son épée (REX AttackHi4 : image 1, élan vertical 2 à 5,5 selon la charge
//!   (0E080200 dans le PSA), puis il est en l'air : retombe, atterrit).

use smash::app::lua_bind::*;
use smash::app::*;
use smash::lib::lua_const::*;
use smash::lib::L2CValue;
use smash::lua2cpp::*;
use smash::phx::Hash40;
use smash_script::*;
use smashline::*;

/// images de charge du smash haut (comptées pendant ATTACK_HI4_HOLD)
static mut CHARGE: [f32; 8] = [0.0; 8];

fn e_of(boma: *mut BattleObjectModuleAccessor) -> usize {
    unsafe { (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7) }
}

pub unsafe fn update(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let e = e_of(boma);
    let st = StatusModule::status_kind(boma);
    if st == *FIGHTER_STATUS_KIND_ATTACK_HI4_START {
        CHARGE[e] = 0.0;
    } else if st == *FIGHTER_STATUS_KIND_ATTACK_HI4_HOLD {
        CHARGE[e] += 1.0;
    }
}

unsafe extern "C" fn hi4_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = e_of(boma);
    MotionModule::change_motion(boma, Hash40::new("attack_hi4"), 0.0, 1.0, false, 0.0, false, false);
    // REX : 2,0 sans charge, 5,5 à pleine charge (unités de Brawl) ; un peu réduit (sauts d'Ultimate)
    let r = (CHARGE[e] / 60.0).clamp(0.0, 1.0);
    let vy = 2.0 + 2.2 * r;
    StatusModule::set_situation_kind(boma, SituationKind(*SITUATION_KIND_AIR), false);
    fighter.global_table[0x17].assign(&L2CValue::I32(*SITUATION_KIND_AIR));
    fighter.global_table[0x16].assign(&L2CValue::I32(*SITUATION_KIND_AIR));
    GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
    KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_FALL);
    sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY, vy);
    fighter.sub_shift_status_main(L2CValue::Ptr(hi4_loop as *const () as _))
}

unsafe extern "C" fn hi4_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let f = MotionModule::frame(boma);
    if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND && f > 6.0 {
        fighter.change_status(FIGHTER_STATUS_KIND_LANDING.into(), false.into());
        return 1.into();
    }
    // REX : AllowInterrupt à l'image 31
    if f >= 31.0 && fighter.sub_air_check_fall_common().get_bool() {
        return 1.into();
    }
    if MotionModule::is_end(boma) {
        fighter.change_status(FIGHTER_STATUS_KIND_FALL.into(), false.into());
        return 1.into();
    }
    0.into()
}

unsafe extern "C" fn rien(_agent: &mut L2CAgentBase) {}

pub fn install(a: &mut Agent) {
    a.status(Main, *FIGHTER_STATUS_KIND_ATTACK_HI4, hi4_main);
    // v0.24 : haut B = animation de REX (SpecialHi : l'Auroros l'emporte, montée dans l'animation) et ses coups
    // (generated.rs, MAP_PLUS) ; effets, sons et expressions du Super Saut Poing de Mario (pièces) retirés
    for m in ["specialhi", "specialairhi"] {
        for k in ["effect", "sound", "expression"] {
            a.acmd(&format!("{}_{}", k, m), rien, Priority::Low);
        }
    }
}
