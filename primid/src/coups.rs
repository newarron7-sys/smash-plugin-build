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


// ------------------------------------------------------------------ v0.26 : haut B
/// Trajet de REX (animation SpecialHi, os Trans, 81 images) : l'Auroros l'attrape (images 0-22), l'emporte en avant et
/// vers le haut (53 de haut, 35 en avant), puis l'oiseau repart (image 67). En v0.24 / v0.25 la montée était laissée à
/// l'animation, mais le Super Saut Poing de Mario ne l'applique pas : il restait au sol (vidéo du joueur). Ici
/// le statut est remplacé : déplacement image par image (énergie STOP, respecte sols et murs), puis chute sans action.
const HI_Y: [f32; 81] = [0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 2.42, 5.91, 9.88, 13.79, 17.07, 19.83, 22.57, 25.26, 27.89, 30.46, 32.94, 35.32, 37.58, 39.72, 41.71, 43.54, 45.20, 46.68, 47.95, 49.01, 49.89, 50.66, 51.31, 51.84, 52.28, 52.61, 52.85, 53.00, 53.06, 53.04, 52.95, 52.79, 52.56, 52.27, 51.92, 51.53, 51.08, 50.60, 50.08, 49.53, 48.95, 48.35, 47.74, 47.11, 46.48, 45.85, 45.22, 44.59, 43.98, 43.39, 42.82, 42.28, 41.77, 41.30, 40.87, 40.49, 40.17, 39.90];
const HI_Z: [f32; 81] = [0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.39, 0.94, 1.59, 2.26, 2.91, 3.52, 4.15, 4.78, 5.42, 6.06, 6.71, 7.36, 8.01, 8.66, 9.31, 9.96, 10.60, 11.23, 11.85, 12.47, 13.08, 13.68, 14.28, 14.88, 15.48, 16.07, 16.67, 17.26, 17.85, 18.43, 19.02, 19.60, 20.18, 20.76, 21.34, 21.92, 22.50, 23.07, 23.65, 24.22, 24.79, 25.37, 25.94, 26.51, 27.08, 27.66, 28.23, 28.80, 29.37, 29.95, 30.52, 31.09, 31.67, 32.24, 32.82, 33.40, 33.98, 34.56];
const HI_LACHE: f32 = 67.0;
/// un peu plus haut que l'animation (dans la vidéo de REX il monte bien plus que sa taille x 3)
const HI_ECH: f32 = 1.25;

unsafe extern "C" fn hi_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let air = StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR;
    let m = if air { "special_air_hi" } else { "special_hi" };
    MotionModule::change_motion(boma, Hash40::new(m), 0.0, 1.0, false, 0.0, false, false);
    StatusModule::set_situation_kind(boma, SituationKind(*SITUATION_KIND_AIR), false);
    fighter.global_table[0x17].assign(&L2CValue::I32(*SITUATION_KIND_AIR));
    fighter.global_table[0x16].assign(&L2CValue::I32(*SITUATION_KIND_AIR));
    GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
    KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_AIR_STOP);
    KineticModule::unable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    sv_kinetic_energy!(reset_energy, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, *ENERGY_STOP_RESET_TYPE_FREE, 0.0, 0.0, 0.0, 0.0, 0.0);
    sv_kinetic_energy!(set_accel, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0, 0.0);
    sv_kinetic_energy!(set_stable_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0, 0.0);
    sv_kinetic_energy!(set_limit_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, -1.0, -1.0);
    KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_STOP);
    sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0_f32, 0.0_f32);
    fighter.sub_shift_status_main(L2CValue::Ptr(hi_loop as *const () as _))
}

unsafe extern "C" fn hi_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let f = MotionModule::frame(boma);
    let lr = PostureModule::lr(boma);
    if f >= HI_LACHE || MotionModule::is_end(boma) {
        // l'oiseau le lâche : chute sans action (REX)
        KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_FALL);
        fighter.change_status(FIGHTER_STATUS_KIND_FALL_SPECIAL.into(), false.into());
        return 1.into();
    }
    let i = (f as usize).min(79);
    let (dy, dz) = (HI_Y[i + 1] - HI_Y[i], HI_Z[i + 1] - HI_Z[i]);
    // pendant la prise de l'oiseau (avant l'image 22) : presque immobile en l'air
    sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, dz * HI_ECH * lr, dy * HI_ECH);
    if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND && f > 30.0 {
        fighter.change_status(FIGHTER_STATUS_KIND_LANDING_FALL_SPECIAL.into(), false.into());
        return 1.into();
    }
    0.into()
}

unsafe extern "C" fn hi_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    KineticModule::enable_energy(fighter.module_accessor, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    0.into()
}

unsafe extern "C" fn hi_rien(_fighter: &mut L2CFighterCommon) -> L2CValue {
    0.into()
}

unsafe extern "C" fn rien(_agent: &mut L2CAgentBase) {}

pub fn install(a: &mut Agent) {
    a.status(Main, *FIGHTER_STATUS_KIND_ATTACK_HI4, hi4_main);
    a.status(Main, *FIGHTER_STATUS_KIND_SPECIAL_HI, hi_main);
    a.status(Exec, *FIGHTER_STATUS_KIND_SPECIAL_HI, hi_rien);
    a.status(End, *FIGHTER_STATUS_KIND_SPECIAL_HI, hi_end);
    // v0.24 : haut B = animation de REX (SpecialHi : l'Auroros l'emporte, montée dans l'animation) et ses coups
    // (generated.rs, MAP_PLUS) ; effets, sons et expressions du Super Saut Poing de Mario (pièces) retirés
    for m in ["specialhi", "specialairhi"] {
        for k in ["effect", "sound", "expression"] {
            a.acmd(&format!("{}_{}", k, m), rien, Priority::Low);
        }
    }
}
