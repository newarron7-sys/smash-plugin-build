//! Dark Bowser : side B = « Koopa Punch » (animation du mod GameBanana 608236 de Userbouy) :
//! Bowser fonce en avant (au sol comme en l'air) et frappe d'un gros coup de poing.
//! Remplace le side B HDR (prise) pour le slot c55.
use super::*;

const DASH_DEBUT: f32 = 8.0;
const DASH_FIN: f32 = 24.0;
const DASH_VITESSE: f32 = 5.6;

unsafe fn changer_cinetique(fighter: &mut L2CFighterCommon) {
    if fighter.is_situation(*SITUATION_KIND_GROUND) {
        KineticModule::change_kinetic(fighter.module_accessor, *FIGHTER_KINETIC_TYPE_GROUND_STOP);
        GroundModule::set_correct(fighter.module_accessor, GroundCorrectKind(*GROUND_CORRECT_KIND_GROUND_CLIFF_STOP_ATTACK));
    } else {
        KineticModule::change_kinetic(fighter.module_accessor, *FIGHTER_KINETIC_TYPE_AIR_STOP);
        GroundModule::set_correct(fighter.module_accessor, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
    }
}

unsafe extern "C" fn punch_init(_fighter: &mut L2CFighterCommon) -> L2CValue { 0.into() }
unsafe extern "C" fn punch_exec(_fighter: &mut L2CFighterCommon) -> L2CValue { 0.into() }
unsafe extern "C" fn punch_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    KineticModule::enable_energy(fighter.module_accessor, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    0.into()
}

unsafe extern "C" fn punch_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    fighter.sub_change_motion_by_situation(Hash40::new("special_s_catch").into(), Hash40::new("special_s_air_catch").into(), false.into());
    changer_cinetique(fighter);
    let vx = KineticModule::get_sum_speed_x(fighter.module_accessor, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_ALL);
    sv_kinetic_energy!(set_speed, fighter, FIGHTER_KINETIC_ENERGY_ID_STOP, vx * 0.3, 0.0);
    // en l'air : une seule fois par saut (remis à zéro à l'atterrissage, comme le side B HDR)
    let deja = VarModule::is_flag(fighter.battle_object, vars::koopa::instance::SPECIAL_S_DISABLE_STALL);
    VarModule::set_flag(fighter.battle_object, vars::koopa::status::SPECIAL_S_ABOVE_BLASTZONE, deja); // variable de statut réutilisée : « déjà utilisé dans ce saut »
    if fighter.is_situation(*SITUATION_KIND_AIR) {
        VarModule::on_flag(fighter.battle_object, vars::koopa::instance::SPECIAL_S_DISABLE_STALL);
    }
    fighter.sub_shift_status_main(L2CValue::Ptr(punch_loop as *const () as _))
}

unsafe extern "C" fn punch_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    if CancelModule::is_enable_cancel(fighter.module_accessor) {
        if fighter.sub_wait_ground_check_common(false.into()).get_bool()
        || fighter.sub_air_check_fall_common().get_bool() {
            return 0.into();
        }
    }
    if StatusModule::is_situation_changed(fighter.module_accessor) {
        fighter.sub_change_motion_by_situation(Hash40::new("special_s_catch").into(), Hash40::new("special_s_air_catch").into(), true.into());
        changer_cinetique(fighter);
    }
    if MotionModule::is_end(fighter.module_accessor) {
        if fighter.is_situation(*SITUATION_KIND_AIR) {
            fighter.change_status(FIGHTER_STATUS_KIND_FALL.into(), false.into());
        } else {
            fighter.change_status(FIGHTER_STATUS_KIND_WAIT.into(), false.into());
        }
        return 0.into();
    }
    let f = MotionModule::frame(fighter.module_accessor);
    let lr = PostureModule::lr(fighter.module_accessor);
    let deja = VarModule::is_flag(fighter.battle_object, vars::koopa::status::SPECIAL_S_ABOVE_BLASTZONE);
    if f >= DASH_DEBUT && f < DASH_FIN {
        // ruée : vitesse fixe vers l'avant, sans gravité en l'air (sauf 2e utilisation dans le même saut)
        let vy = if fighter.is_situation(*SITUATION_KIND_AIR) && !deja { 0.0 } else {
            KineticModule::get_sum_speed_y(fighter.module_accessor, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_ALL)
        };
        if fighter.is_situation(*SITUATION_KIND_AIR) && !deja {
            KineticModule::unable_energy(fighter.module_accessor, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
        }
        sv_kinetic_energy!(set_speed, fighter, FIGHTER_KINETIC_ENERGY_ID_STOP, DASH_VITESSE * lr, vy);
    } else if f >= DASH_FIN && f < DASH_FIN + 1.0 {
        sv_kinetic_energy!(set_speed, fighter, FIGHTER_KINETIC_ENERGY_ID_STOP, 0.9 * lr, 0.0);
        KineticModule::enable_energy(fighter.module_accessor, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    }
    0.into()
}

pub fn install(agent: &mut Agent) {
    agent.status(Init, *FIGHTER_STATUS_KIND_SPECIAL_S, punch_init);
    agent.status(Main, *FIGHTER_STATUS_KIND_SPECIAL_S, punch_main);
    agent.status(Exec, *FIGHTER_STATUS_KIND_SPECIAL_S, punch_exec);
    agent.status(End, *FIGHTER_STATUS_KIND_SPECIAL_S, punch_end);
}
