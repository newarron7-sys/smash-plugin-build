//! Armes de Master Chief (maillages chief_* du modèle, extraits de REX par armes_rex.py / armes_ult.py) affichées
//! selon le coup comme dans REX (effets 0x1FB00xx posés sur RHaveN / LHaveN / BustN dans ses scripts) ; attaque en
//! course (deux versions de REX) ; bas aérien « Ground Pound » (départ, charge tant que A est tenu, plongée).

use smash::app::lua_bind::*;
use smash::app::*;
use smash::hash40;
use smash::lib::lua_const::*;
use smash::lua2cpp::*;
use smash::phx::{Hash40, Vector3f};
use smash_script::*;

const TOUTES: [&str; 14] = [
    "chief_lanceur", "chief_rifle", "chief_rifledos", "chief_sword", "chief_swordb", "chief_smg", "chief_smgl", "chief_sniper",
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
        // B neutre tapé : magnum (REX : effet 0x1FB0012 sur LHaveN, fusil rangé dans le dos) et son rechargement
        m if m == hash40("special_n_throw_m") || m == hash40("special_air_n_throw_m")
            || m == hash40("special_lw_squat_start") || m == hash40("special_air_lw_wall") => &["chief_magnum", "chief_rifledos"],
        // Final Smash : lance-roquettes de REX (FitChiefFinal.pac, effet 0xE00007 sur RHaveN)
        m if m == hash40("final_start_r") || m == hash40("final_start_l") || m == hash40("final_air_start_r")
            || m == hash40("final_air_start_l") => &["chief_lanceur", "chief_rifledos"],
        _ => &["chief_rifle"],
    }
}

/// lancer haut (REX : épée 0x1FB0001 à l'image 0, lames 0x1FB0009 à 9, lames retirées à 104, épée à 124)
fn epee_lancer_haut(f: f32) -> &'static [&'static str] {
    if f < 9.0 {
        &["chief_sword", "chief_rifledos"]
    } else if f < 104.0 {
        &["chief_sword", "chief_swordb", "chief_rifledos"]
    } else if f < 124.0 {
        &["chief_sword", "chief_rifledos"]
    } else {
        &["chief_rifle"]
    }
}

unsafe fn id_of(boma: *mut BattleObjectModuleAccessor) -> usize {
    (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7)
}

pub unsafe fn visibilite(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let m = MotionModule::motion_kind(boma);
    // à chaque image (le modèle peut être rechargé : entrée, réapparition)
    let on = if m == hash40("throw_hi") { epee_lancer_haut(MotionModule::frame(boma)) } else { armes_de(m) };
    for n in TOUTES {
        ModelModule::set_mesh_visibility(boma, Hash40::new(n), on.contains(&n));
    }
}

/// attaque en course : « Spartan Charge » (AttackDash + AttackDashSpartan de REX). La « Slide Kick » (2e version de
/// REX quand il court) passait par le mouvement swing_dash de Snake : pose en T en jeu (vidéo), retirée
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

/// jet bleu choisi par le joueur dans la galerie d'effets (photo : dôme cyan à cœur clair posé au sol = le haut d'un
/// jet qui part de son origine vers le bas, le reste caché par le sol) : celui de la ceinture-fusée du jeu (dans le
/// jeu : EFFECT_FOLLOW sur l'objet, taille 1, un par tuyère, en boucle jusqu'à EFFECT_OFF_KIND)
pub const JET: &str = "sys_rocketbelt_jet";
const GP_JET_TAILLE: f32 = 0.55;
static mut GP_JET: [[u32; 2]; 8] = [[0; 2]; 8];
static mut GP_JET_T: [[i32; 2]; 8] = [[0; 2]; 8];

/// demandé : « mets cet effet-là sur ses deux pieds pendant le down air ». Un jet par pied, créé une fois puis déplacé
/// à chaque image sur le pied (position monde : il pointe toujours vers le bas, quelle que soit la pose), recréé s'il
/// s'éteint (4 images au moins entre deux), retiré à la fin du coup
unsafe fn jets_pieds(boma: *mut BattleObjectModuleAccessor, id: usize, actif: bool) {
    let z = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    for (k, os) in ["footl", "footr"].iter().enumerate() {
        let h = GP_JET[id][k];
        if !actif {
            if h != 0 {
                EffectModule::kill(boma, h, false, true);
                GP_JET[id][k] = 0;
            }
            GP_JET_T[id][k] = 4;
            continue;
        }
        let mut p = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
        ModelModule::joint_global_position(boma, Hash40::new(os), &mut p, true);
        let pos = Vector3f { x: p.x, y: p.y - 0.5, z: p.z };
        GP_JET_T[id][k] += 1;
        if h != 0 && EffectModule::is_exist_effect(boma, h) {
            EffectModule::set_pos(boma, h, &pos);
        } else if GP_JET_T[id][k] >= 4 {
            GP_JET_T[id][k] = 0;
            GP_JET[id][k] = EffectModule::req(boma, Hash40::new(JET), &pos, &z, GP_JET_TAILLE, 0, 0, false, 0) as u32;
        }
    }
}

pub unsafe fn ground_pound(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let id = id_of(boma);
    let dans = StatusModule::status_kind(boma) == *FIGHTER_STATUS_KIND_ATTACK_AIR
        && MotionModule::motion_kind(boma) == hash40("attack_air_lw")
        && StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR;
    jets_pieds(boma, id, dans && MotionModule::frame(boma) >= 3.0);
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

/// demandé (vidéos : « il se téléporte », 2 fois) : ses animations de REX avancent (jab 3 : +18, smash avant :
/// +15,8) mais le mouvement de Snake ne déplace pas le perso pour ces coups (« move » faux dans son motion_list : le
/// passage en physique MOTION n'y changeait rien). Leurs animations sont donc sur place (retarget_exo.py) et le
/// plugin le fait avancer de la même distance que dans REX, image par image (table AVANCE de generated.rs,
/// rex/tools/avance_rex.py) ; au bord il s'arrête (correction de sol du statut)
static mut AV_PREC: [(u64, f32); 8] = [(0, 0.0); 8];

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
