//! Armes de Master Chief (maillages chief_* du modèle, extraits de REX par armes_rex.py / armes_ult.py) affichées
//! selon le coup comme dans REX (effets 0x1FB00xx posés sur RHaveN / LHaveN / BustN dans ses scripts) ; attaque en
//! course (deux versions de REX) ; bas aérien « Ground Pound » (départ, charge tant que A est tenu, plongée).

use smash::app::lua_bind::*;
use smash::app::*;
use smash::hash40;
use smash::lib::lua_const::*;
use smash::lua2cpp::*;
use smash::phx::Hash40;
use smash_script::*;

const TOUTES: [&str; 13] = [
    "chief_rifle", "chief_rifledos", "chief_sword", "chief_swordb", "chief_smg", "chief_smgl", "chief_sniper",
    "chief_shotgun", "chief_battle", "chief_plasmar", "chief_plasmarl", "chief_magnum", "chief_plasmap",
];

/// armes visibles pendant l'animation (REX : fusil d'assaut en main par défaut, rangé dans le dos quand il prend une
/// autre arme ou frappe à mains nues)
fn armes_de(motion: u64) -> &'static [&'static str] {
    match motion {
        m if m == hash40("attack_11") || m == hash40("attack_12") || m == hash40("attack_13") => &["chief_rifledos"],
        m if m == hash40("attack_s3_s") || m == hash40("attack_s3_s2") => &["chief_battle", "chief_rifledos"],
        m if m == hash40("attack_hi3") => &["chief_smg", "chief_smgl", "chief_rifledos"],
        m if m == hash40("attack_lw3") => &["chief_plasmap", "chief_rifledos"],
        m if m == hash40("attack_s4_s") || m == hash40("attack_s4_hold") => &["chief_shotgun", "chief_rifledos"],
        m if m == hash40("attack_lw4") || m == hash40("attack_lw4_hold") => &["chief_sniper", "chief_rifledos"],
        m if m == hash40("attack_air_f") || m == hash40("landing_air_f") => &["chief_sword", "chief_swordb", "chief_rifledos"],
        m if m == hash40("attack_air_hi") || m == hash40("landing_air_hi") => &["chief_plasmar", "chief_plasmarl", "chief_rifledos"],
        _ => &["chief_rifle"],
    }
}

unsafe fn id_of(boma: *mut BattleObjectModuleAccessor) -> usize {
    (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7)
}

pub unsafe fn visibilite(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let m = MotionModule::motion_kind(boma);
    // à chaque image (le modèle peut être rechargé : entrée, réapparition)
    let on = armes_de(m);
    for n in TOUTES {
        ModelModule::set_mesh_visibility(boma, Hash40::new(n), on.contains(&n));
    }
}

/// attaque en course : dans REX, deux versions après le même début (AttackDash) : « Slide Kick » quand il court,
/// « Spartan Charge » sinon (animation swing_dash = début + glissade, voir retarget_exo.py chief_coups)
pub unsafe fn attaque_course(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    if StatusModule::status_kind(boma) != *FIGHTER_STATUS_KIND_ATTACK_DASH {
        return;
    }
    if MotionModule::motion_kind(boma) != hash40("attack_dash") || MotionModule::frame(boma) > 1.0 {
        return;
    }
    if StatusModule::prev_status_kind(boma, 0) == *FIGHTER_STATUS_KIND_RUN {
        MotionModule::change_motion(boma, Hash40::new("swing_dash"), 0.0, 1.0, false, 0.0, false, false);
    }
}

/// « Ground Pound » (images de son animation : départ 0-12, charge 12-67, plongée 67-261) : immobile en l'air
/// pendant le départ et la charge ; A lâché (ou fin de la charge) -> plongée : petit bond puis piqué (REX : vitesse
/// 1,6 vers l'avant, 3,8 vers le bas) jusqu'au sol ; l'atterrissage (landing_air_lw) fait l'onde de choc
const GP_CHARGE: f32 = 12.0;
const GP_GO: f32 = 67.0;
const GP_PIQUE: f32 = 87.0;
static mut GP_LIBRE: [bool; 8] = [false; 8];

unsafe fn stop_libre(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    sv_kinetic_energy!(reset_energy, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, *ENERGY_STOP_RESET_TYPE_FREE, 0.0, 0.0, 0.0, 0.0, 0.0);
    sv_kinetic_energy!(set_accel, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0, 0.0);
    sv_kinetic_energy!(set_stable_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0, 0.0);
    sv_kinetic_energy!(set_limit_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, -1.0, -1.0);
    KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_STOP);
    KineticModule::unable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    KineticModule::unable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_CONTROL);
}

pub unsafe fn ground_pound(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let id = id_of(boma);
    let dans = StatusModule::status_kind(boma) == *FIGHTER_STATUS_KIND_ATTACK_AIR
        && MotionModule::motion_kind(boma) == hash40("attack_air_lw")
        && StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR;
    if !dans {
        GP_LIBRE[id] = false;
        return;
    }
    if !GP_LIBRE[id] {
        stop_libre(fighter);
        GP_LIBRE[id] = true;
    }
    let f = MotionModule::frame(boma);
    let lr = PostureModule::lr(boma);
    if f < GP_GO {
        // départ et charge : il reste en l'air (jets des propulseurs : effets.rs)
        sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0, 0.0);
        if f >= GP_CHARGE && !ControlModule::check_button_on(boma, *CONTROL_PAD_BUTTON_ATTACK) {
            MotionModule::set_frame_sync_anim_cmd(boma, GP_GO, true, true, false);
        }
    } else if f < GP_PIQUE {
        sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0, 0.6);
    } else {
        sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 1.6 * lr, -3.8);
        // il plonge jusqu'au sol : l'animation (longue) ne doit pas se finir en l'air
        // (avant l'image 197 où REX retire ses coups : ils restent jusqu'au sol)
        if f > 185.0 {
            MotionModule::set_frame(boma, GP_PIQUE + 33.0, false);
        }
    }
}

/// demandé (vidéo : « il se téléporte ») : ses animations de REX avancent (TransN : uppercut du jab +18, smash avant,
/// attaques en course) ; dans les statuts d'attaque au sol de Snake le jeu ne suivait pas ce déplacement : le modèle
/// partait devant puis revenait d'un coup à sa place. Physique « MOTION » : le perso suit le déplacement de
/// l'animation (comme les attaques en course du jeu)
pub unsafe fn suivre_animation(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    if StatusModule::situation_kind(boma) != *SITUATION_KIND_GROUND {
        return;
    }
    let st = StatusModule::status_kind(boma);
    let sol = [*FIGHTER_STATUS_KIND_ATTACK, *FIGHTER_STATUS_KIND_ATTACK_S3, *FIGHTER_STATUS_KIND_ATTACK_HI3,
               *FIGHTER_STATUS_KIND_ATTACK_LW3, *FIGHTER_STATUS_KIND_ATTACK_S4, *FIGHTER_STATUS_KIND_ATTACK_HI4,
               *FIGHTER_STATUS_KIND_ATTACK_LW4, *FIGHTER_STATUS_KIND_ATTACK_DASH];
    if !sol.contains(&st) {
        return;
    }
    if KineticModule::get_kinetic_type(boma) != *FIGHTER_KINETIC_TYPE_MOTION {
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_MOTION);
    }
}
