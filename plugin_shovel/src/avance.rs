//! Coups qui avancent dans REX (jab 2 / 3, tilt avant, smash avant) : animations sur place (le mouvement de Link
//! enfant ne suit pas l'avancée de l'animation : sinon le modèle part devant puis revient d'un coup), le plugin le
//! fait avancer de la même distance que dans REX, image par image (table AVANCE de generated.rs, avance_rex.py).
//! Même méthode que Master Chief ; au sol seulement (au bord, la correction de sol du statut l'arrête).

use smash::app::lua_bind::*;
use smash::app::*;
use smash::hash40;
use smash::lib::lua_const::*;
use smash::lua2cpp::*;
use smash_script::*;

static mut AV_PREC: [(u64, f32); 8] = [(0, 0.0); 8];

unsafe fn id_of(boma: *mut BattleObjectModuleAccessor) -> usize {
    (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7)
}

fn avance_a(t: &[f32], f: f32) -> f32 {
    if t.is_empty() {
        return 0.0;
    }
    let f = f.max(0.0);
    let i = f.floor() as usize;
    if i + 1 >= t.len() {
        return t[t.len() - 1];
    }
    let k = f - i as f32;
    t[i] * (1.0 - k) + t[i + 1] * k
}

pub unsafe fn suivre_animation(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let id = id_of(boma);
    let m = MotionModule::motion_kind(boma);
    let tab = crate::AVANCE.iter().find(|(n, _)| hash40(n) == m).map(|(_, t)| *t);
    let t = match tab {
        Some(t) if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND => t,
        _ => {
            AV_PREC[id] = (0, 0.0);
            return;
        }
    };
    let f = MotionModule::frame(boma);
    let (pm, pf) = AV_PREC[id];
    AV_PREC[id] = (m, f);
    if pm != m || f < pf {
        return;
    }
    let d = avance_a(t, f) - avance_a(t, pf);
    let lr = PostureModule::lr(boma);
    KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_STOP);
    sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, d * lr, 0.0);
}
