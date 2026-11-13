//! Final Smash de Master Chief « Grenade Launcher » (REX : sous-action « FinalSmashplaceholder » de FitChief.pac +
//! lance-roquettes de FitChiefFinal.pac), refait d'après la vidéo de REX envoyée par le joueur :
//! 1. debout, aura du Final Smash qui change de couleur (bleu, cyan, vert, jaune, orange) ;
//! 2. il décolle en arrière et vers le haut (éclat blanc et souffle au sol : REX image 5, 14 % autour de lui), jets
//!    sous les pieds, lance-roquettes en main ;
//! 3. en l'air, il vise devant / en bas (cible rouge), tire une grenade (traînée blanche) ;
//! 4. énorme explosion sur la cible (REX : 46 %, angle 48, base 255, taille 45) ;
//! 5. il retombe avec le lance-roquettes jusqu'au sol.
//! Animation : poses du lance-roquettes de REX (ItemLauncher / ItemLauncherFall / ItemLauncherAirFire), voir
//! retarget_exo.py (groupe chief_final). Statut FINAL de Snake remplacé (comme le Final Smash de Zero sur Roy) ; fin :
//! statut de fin du jeu (remise à zéro de l'état « Final Smash »).

use smash::app::lua_bind::*;
use smash::app::*;
use smash::lib::lua_const::*;
use smash::lib::L2CValue;
use smash::lua2cpp::*;
use smash::phx::{Hash40, Vector3f};
use smash_script::macros::*;
use smash_script::*;
use smashline::*;

use crate::speciaux::slot;

const ECHELLE: f32 = 0.82;
const BLANC: (f32, f32, f32) = (1.0, 1.0, 1.0);
const ROUGE: (f32, f32, f32) = (1.0, 0.1, 0.05);
const ORANGE: (f32, f32, f32) = (1.0, 0.55, 0.15);
/// couleurs successives de l'aura (vidéo de REX)
const AURA: [(f32, f32, f32); 5] = [(0.3, 0.5, 1.0), (0.3, 1.0, 1.0), (0.5, 1.0, 0.3), (1.0, 0.95, 0.3), (1.0, 0.55, 0.15)];

// images du jeu (animation : 20 debout + 54 en vol + 12 tir + 72 retombée)
const G_DECOLLE: i32 = 20;
const G_VOL: i32 = 40;          // durée de la montée
const G_CIBLE: i32 = 50;
const G_TIR: i32 = 76;
const G_BOUM: i32 = 84;
const G_FIN: i32 = 158;
/// vol : recul et montée totaux (vidéo : ~2 tailles en arrière, ~2,5 tailles en haut)
const RECUL: f32 = 40.0;
const MONTEE: f32 = 45.0;
/// cible : devant son point de départ (vidéo : sur l'adversaire, ~2,5 tailles devant lui)
const CIBLE_AV: f32 = 50.0;

#[derive(Clone, Copy)]
struct St {
    g: i32,
    air0: bool,
    x0: f32,
    y0: f32,
    lr: f32,
    cible: (f32, f32),
    dep: (f32, f32),
    aura: u32,
    vy: f32,
}
static mut ST: [St; 8] = [St { g: 0, air0: false, x0: 0.0, y0: 0.0, lr: 1.0, cible: (0.0, 0.0), dep: (0.0, 0.0), aura: 0, vy: 0.0 }; 8];

unsafe fn fx_w(boma: *mut BattleObjectModuleAccessor, name: &str, x: f32, y: f32, sc: f32, c: (f32, f32, f32)) {
    let pos = Vector3f { x, y, z: 0.0 };
    let rot = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    let h = EffectModule::req(boma, Hash40::new(name), &pos, &rot, sc, 0, 0, false, 0) as u32;
    if h != 0 {
        EffectModule::set_rgb(boma, h, c.0, c.1, c.2);
    }
}

unsafe fn fx_os(boma: *mut BattleObjectModuleAccessor, name: &str, os: &str, z: f32, sc: f32, c: (f32, f32, f32)) -> u32 {
    let p = Vector3f { x: 0.0, y: 0.0, z };
    let r = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    let h = EffectModule::req_follow(boma, Hash40::new(name), Hash40::new(os), &p, &r, sc, true, 0, 0, 0, 0, 0, false, false) as u32;
    if h != 0 {
        EffectModule::set_rgb(boma, h, c.0, c.1, c.2);
    }
    h
}

unsafe fn en_air(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    fighter.set_situation(SITUATION_KIND_AIR.into());
    GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
    KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_AIR_STOP);
    KineticModule::unable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
}

unsafe extern "C" fn final_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    // v29 (vidéo : après le Final Smash il le gardait) : début du Final Smash annoncé au jeu, comme le statut Final de
    // Snake (dump Coolsonickirby : événement 0x201bc9217c) ; sans lui la Smash Ball n'est pas consommée
    notify_event_msc_cmd!(fighter, Hash40::new_raw(0x201bc9217c));
    AreaModule::set_whole(boma, false);
    let air = StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR;
    let p = *PostureModule::pos(boma);
    let lr = PostureModule::lr(boma);
    let cible = (p.x + CIBLE_AV * lr, if air { p.y - 30.0 } else { p.y });
    ST[slot(boma)] = St { g: 0, air0: air, x0: p.x, y0: p.y, lr, cible, dep: (p.x, p.y + 12.0), aura: 0, vy: 0.0 };
    if air {
        en_air(fighter);
    } else {
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_GROUND_STOP);
        GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_GROUND_CLIFF_STOP));
    }
    sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0_f32, 0.0_f32);
    // REX : intangible (06050100 3)
    HitModule::set_whole(boma, HitStatus(*HIT_STATUS_XLU), 0);
    // v29 (vidéo : T-pose) : les mouvements du Final de Snake s'appellent final_start_r / _l et final_air_start_r / _l
    // (son statut Final choisit selon lr) ; « final_start » n'existe pas
    let mot = match (air, lr < 0.0) {
        (false, false) => "final_start_r",
        (false, true) => "final_start_l",
        (true, false) => "final_air_start_r",
        (true, true) => "final_air_start_l",
    };
    MotionModule::change_motion(boma, Hash40::new(mot), 0.0, 1.0, false, 0.0, false, false);
    fighter.sub_shift_status_main(L2CValue::Ptr(final_loop as *const () as _))
}

unsafe extern "C" fn final_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    ST[e].g += 1;
    let g = ST[e].g;
    let s = ST[e];
    if MotionModule::frame(boma) >= MotionModule::end_frame(boma) - 1.0 {
        MotionModule::set_rate(boma, 0.0);
    }
    // v29 (vidéo : il ne décollait pas) : montée posée image par image (trajet en arrière qui ralentit), immobile en
    // l'air pendant la visée et le tir, puis chute normale du jeu (gravité ; atterrissage détecté par le jeu)
    if g >= G_DECOLLE && g <= G_DECOLLE + G_VOL {
        let k = (g - G_DECOLLE) as f32 / G_VOL as f32;
        let e2 = 1.0 - (1.0 - k) * (1.0 - k);
        let z = (*PostureModule::pos(boma)).z;
        let p = Vector3f { x: s.x0 - RECUL * e2 * s.lr, y: s.y0 + MONTEE * e2, z };
        PostureModule::set_pos(boma, &p);
    }
    if g <= G_BOUM + 4 {
        sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0_f32, 0.0_f32);
    } else if g == G_BOUM + 5 {
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_FALL);
        KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    }
    evenement(fighter, g);
    // retombé au sol : fin
    if g > G_BOUM + 6 && (StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND
        || GroundModule::is_touch(boma, *GROUND_TOUCH_FLAG_DOWN as u32)) {
        fighter.set_situation(SITUATION_KIND_GROUND.into());
        GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_GROUND_CLIFF_STOP));
        fighter.change_status(FIGHTER_STATUS_KIND_LANDING.into(), false.into());
        return 0.into();
    }
    if g >= G_FIN + 60 {
        let st = if StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR { FIGHTER_STATUS_KIND_FALL } else { FIGHTER_STATUS_KIND_WAIT };
        fighter.change_status(st.into(), false.into());
    }
    0.into()
}

unsafe fn evenement(fighter: &mut L2CFighterCommon, g: i32) {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    let s = ST[e];
    // aura du Final Smash, couleur qui change toutes les 8 images
    if g == 1 {
        ST[e].aura = fx_os(boma, "sys_final_aura", "hip", 0.0, 1.1, AURA[0]);
        FLASH(fighter, 0.47, 0.35, 0.75, 0.3);
    }
    if g % 8 == 0 && ST[e].aura != 0 && g < G_BOUM + 20 {
        let c = AURA[((g / 8) as usize) % AURA.len()];
        EffectModule::set_rgb(boma, ST[e].aura, c.0, c.1, c.2);
    }
    if g == 8 {
        COL_NORMAL(fighter);
    }
    if g == G_DECOLLE {
        // décollage : éclat blanc au sol, souffle autour de lui (REX image 5 : 2 coups de 14 %)
        en_air(fighter);
        PLAY_SE(fighter, Hash40::new("se_snake_final01"));
        fx_w(boma, "sys_smash_flash", s.x0, s.y0 + 4.0, 2.2, BLANC);
        LANDING_EFFECT(fighter, Hash40::new("sys_landing_smoke"), Hash40::new("top"), 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.2, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, false);
        EFFECT_FOLLOW(fighter, Hash40::new("sys_attack_speedline"), Hash40::new("top"), 0.0, 8.0, 0.0, -45.0, 0.0, 0.0, 1.2, true);
        for (id, ang, t) in [(0u64, 69u64, 6.0f32), (1, 80, 9.0)] {
            ATTACK(fighter, id, 0, Hash40::new("top"), 14.0, ang, 100, 168, 0, t * ECHELLE, 0.0, 0.0, 0.0, None, None, None, 0.3, 1.0,
                   *ATTACK_SETOFF_KIND_OFF, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true,
                   *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
                   Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_PUNCH, *ATTACK_REGION_BOMB);
        }
    }
    if g == G_DECOLLE + 4 {
        AttackModule::clear_all(boma);
    }
    // jets sous les pieds pendant la montée (flammes orange puis bleues, vidéo)
    if g >= G_DECOLLE && g < G_DECOLLE + G_VOL + 10 && g % 4 == 0 {
        let c = if g < G_DECOLLE + 20 { ORANGE } else { (0.4, 0.9, 1.0) };
        for os in ["footl", "footr"] {
            fx_os(boma, "sys_hit_fire", os, 0.0, 0.45, c);
        }
    }
    // cible rouge sur le point visé
    if g >= G_CIBLE && g < G_BOUM && (g - G_CIBLE) % 4 == 0 {
        fx_w(boma, "sys_smash_flash_s", s.cible.0, s.cible.1 + 3.0, 1.6, ROUGE);
    }
    if g == G_TIR {
        // tir de la grenade (REX : son 58)
        PLAY_SE(fighter, Hash40::new("se_snake_final02"));
        fx_os(boma, "sys_muzzleflash", "haver", 8.0, 1.6, BLANC);
        fx_os(boma, "sys_hit_fire", "haver", 8.0, 0.8, BLANC);
        fx_os(boma, "sys_dash_smoke", "haver", -16.0, 1.0, (0.85, 0.85, 0.85));
        QUAKE(fighter, *CAMERA_QUAKE_KIND_S);
        let mut p = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
        ModelModule::joint_global_position(boma, Hash40::new("haver"), &mut p, true);
        ST[e].dep = (p.x, p.y);
    }
    // grenade en vol : traînée blanche jusqu'à la cible
    if g > G_TIR && g < G_BOUM {
        let a = (g - G_TIR) as f32 / (G_BOUM - G_TIR) as f32;
        let x = s.dep.0 + (s.cible.0 - s.dep.0) * a;
        let y = s.dep.1 + (s.cible.1 + 4.0 - s.dep.1) * a;
        fx_w(boma, "sys_smash_flash_s", x, y, 0.7, BLANC);
        fx_w(boma, "sys_dash_smoke", x, y, 0.35, (0.9, 0.9, 0.9));
    }
    if g == G_BOUM {
        // explosion sur la cible (REX : 46 %, angle 48, base 255, taille 45 ; son 59)
        let (cx, cy) = s.cible;
        PLAY_SE(fighter, Hash40::new("se_snake_final03"));
        PLAY_SE(fighter, Hash40::new("se_common_bomb_l"));
        fx_w(boma, "sys_smash_flash", cx, cy + 12.0, 3.4, (1.0, 0.95, 0.7));
        fx_w(boma, "sys_bomb_a", cx, cy + 10.0, 3.8, BLANC);
        fx_w(boma, "sys_bomb_a", cx - 14.0, cy + 6.0, 2.2, BLANC);
        fx_w(boma, "sys_bomb_a", cx + 14.0, cy + 6.0, 2.2, BLANC);
        QUAKE(fighter, *CAMERA_QUAKE_KIND_XL);
        AttackModule::clear_all(boma);
        let p = *PostureModule::pos(boma);
        let dz = (cx - p.x) * s.lr;
        let dy = cy + 10.0 - p.y;
        ATTACK(fighter, 0, 0, Hash40::new("top"), 46.0, 48, 100, 0, 255, 45.0 * ECHELLE, 0.0, dy, dz, None, None, None, 2.0, 1.0,
               *ATTACK_SETOFF_KIND_OFF, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true,
               *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
               Hash40::new("collision_attr_fire"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_FIRE, *ATTACK_REGION_BOMB);
    }
    if g == G_BOUM + 3 {
        AttackModule::clear_all(boma);
    }
    // boule de feu orange puis fumée noire (vidéo)
    if g == G_BOUM + 6 || g == G_BOUM + 12 {
        fx_w(boma, "sys_bomb_a", s.cible.0 + (g - G_BOUM - 9) as f32, s.cible.1 + 14.0, 2.6, ORANGE);
    }
    if g >= G_BOUM + 16 && g <= G_BOUM + 40 && (g - G_BOUM) % 8 == 0 {
        fx_w(boma, "sys_dash_smoke", s.cible.0, s.cible.1 + 10.0, 2.4, (0.25, 0.25, 0.25));
    }
    if g == G_BOUM + 20 {
        EFFECT_OFF_KIND(fighter, Hash40::new("sys_final_aura"), false, false);
        ST[e].aura = 0;
    }
}

unsafe extern "C" fn final_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    HitModule::set_whole(boma, HitStatus(*HIT_STATUS_NORMAL), 0);
    KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    AttackModule::clear_all(boma);
    EFFECT_OFF_KIND(fighter, Hash40::new("sys_final_aura"), false, false);
    COL_NORMAL(fighter);
    ST[slot(boma)].aura = 0;
    smashline::original_status(End, fighter, *FIGHTER_STATUS_KIND_FINAL)(fighter)
}

unsafe extern "C" fn final_exec(_fighter: &mut L2CFighterCommon) -> L2CValue {
    0.into()
}

unsafe extern "C" fn vide(_agent: &mut L2CAgentBase) {}

pub fn install(a: &mut Agent) {
    a.status(Main, *FIGHTER_STATUS_KIND_FINAL, final_main);
    a.status(Exec, *FIGHTER_STATUS_KIND_FINAL, final_exec);
    a.status(End, *FIGHTER_STATUS_KIND_FINAL, final_end);
    // scripts de Snake (Cypher, missiles, caméra) : vidés
    for m in ["finalstart", "finalairstart", "finalstartl", "finalairstartl", "finalstartr", "finalairstartr"] {
        for p in ["game_", "effect_", "sound_", "expression_"] {
            let n: &'static str = Box::leak(format!("{}{}", p, m).into_boxed_str());
            a.acmd(n, vide, Priority::Low);
        }
    }
}
