//! Final Smash de Master Chief (REX : sous-action « FinalSmashplaceholder » de FitChief.pac + FitChiefFinal.pac).
//! Dans REX : vitesse 0,6 puis 0,5 (image 11) ; intangible ; lance-roquettes (modèle WpnChiefRocketLauncher, effet
//! 0xE00007 sur RHaveN, tourné de 90°) ; image 5 : souffle autour de lui (14 %, angles 69 / 80, fixe 168) ; image 26 :
//! cible posée 45 devant et 24 sous lui (effet 0x1FB0015) ; image 39 : tir de la roquette (son 58) ; image 43 : énorme
//! explosion sur la cible (46 %, angle 48, base 255, taille 45, son 59) ; fin à 95.
//! REX n'a pas d'animation pour ce Final Smash (« placeholder ») : pose du lance-roquettes de REX (ItemLauncher, visée
//! vers le bas), voir retarget_exo.py (groupe chief_final). Statut FINAL de Snake remplacé (comme le Final Smash de
//! Zero sur Roy) ; fin : statut de fin du jeu (remise à zéro de l'état « Final Smash »).

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
const BLEU: (f32, f32, f32) = (0.45, 0.65, 1.0);
const ROUGE: (f32, f32, f32) = (1.0, 0.1, 0.05);
const ORANGE: (f32, f32, f32) = (1.0, 0.55, 0.15);
const CIBLE_AV: f32 = 45.0;
const CIBLE_BAS: f32 = 24.0;
const T_FIN: f32 = 95.0;

#[derive(Clone, Copy)]
struct St {
    t: f32,
    fait: i32,
    air: bool,
    x0: f32,
    y0: f32,
    lr: f32,
    dep: (f32, f32),
}
static mut ST: [St; 8] = [St { t: 0.0, fait: -1, air: false, x0: 0.0, y0: 0.0, lr: 1.0, dep: (0.0, 0.0) }; 8];

unsafe fn fx_w(boma: *mut BattleObjectModuleAccessor, name: &str, x: f32, y: f32, sc: f32, c: (f32, f32, f32)) {
    let pos = Vector3f { x, y, z: 0.0 };
    let rot = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    let h = EffectModule::req(boma, Hash40::new(name), &pos, &rot, sc, 0, 0, false, 0) as u32;
    if h != 0 {
        EffectModule::set_rgb(boma, h, c.0, c.1, c.2);
    }
}

/// centre visible de l'explosion : la cible de REX (24 sous lui) ; au sol, posée au niveau du sol
unsafe fn centre(s: &St) -> (f32, f32) {
    let x = s.x0 + CIBLE_AV * s.lr;
    let y = if s.air { s.y0 - CIBLE_BAS } else { s.y0 + 3.0 };
    (x, y)
}

unsafe extern "C" fn final_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let air = StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR;
    let p = *PostureModule::pos(boma);
    ST[slot(boma)] = St { t: 0.0, fait: -1, air, x0: p.x, y0: p.y, lr: PostureModule::lr(boma), dep: (p.x, p.y + 12.0) };
    if air {
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_AIR_STOP);
        GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
        KineticModule::unable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
        sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0_f32, 0.0_f32);
    } else {
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_GROUND_STOP);
        GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_GROUND_CLIFF_STOP));
    }
    // REX : intangible (06050100 3)
    HitModule::set_whole(boma, HitStatus(*HIT_STATUS_XLU), 0);
    MotionModule::change_motion(boma, Hash40::new(if air { "final_air_start" } else { "final_start" }), 0.0, 1.0, false, 0.0, false, false);
    fighter.sub_shift_status_main(L2CValue::Ptr(final_loop as *const () as _))
}

unsafe extern "C" fn final_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    // la pose finale (visée vers le bas) reste jusqu'à la fin
    if MotionModule::frame(boma) >= MotionModule::end_frame(boma) - 1.0 {
        MotionModule::set_rate(boma, 0.0);
    }
    if ST[e].air {
        sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0_f32, 0.0_f32);
    }
    // temps de REX : vitesse 0,6 jusqu'à l'image 11, puis 0,5
    let t0 = ST[e].t;
    ST[e].t += if t0 < 11.0 { 0.6 } else { 0.5 };
    let t = ST[e].t;
    let mut k = ST[e].fait + 1;
    while (k as f32) <= t {
        evenement(fighter, k);
        k += 1;
    }
    ST[e].fait = k - 1;
    // roquette en vol (images 39 à 43 de REX) : traînée de feu et de fumée de sa main jusqu'à la cible
    if t > 39.0 && t < 43.0 {
        let s = ST[e];
        let (cx, cy) = centre(&s);
        let a = (t - 39.0) / 4.0;
        let x = s.dep.0 + (cx - s.dep.0) * a;
        let y = s.dep.1 + (cy - s.dep.1) * a;
        fx_w(boma, "sys_hit_fire", x, y, 0.6, (1.0, 1.0, 1.0));
        fx_w(boma, "sys_dash_smoke", x, y, 0.5, (0.8, 0.8, 0.8));
    }
    if t >= T_FIN {
        let st = if StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR { FIGHTER_STATUS_KIND_FALL } else { FIGHTER_STATUS_KIND_WAIT };
        fighter.change_status(st.into(), false.into());
    }
    0.into()
}

unsafe fn evenement(fighter: &mut L2CFighterCommon, k: i32) {
    let boma = fighter.module_accessor;
    let s = ST[slot(boma)];
    let (cx, cy) = centre(&s);
    match k {
        0 => {
            // REX : écran teinté (11170700 : 120, 90, 190), lueur sur le buste et les pieds (0x1FB0006)
            FLASH(fighter, 0.47, 0.35, 0.75, 0.35);
            EFFECT_FOLLOW(fighter, Hash40::new("sys_final_aura"), Hash40::new("hip"), 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.1, true);
            LAST_EFFECT_SET_COLOR(fighter, BLEU.0, BLEU.1, BLEU.2);
            LANDING_EFFECT(fighter, Hash40::new("sys_down_smoke"), Hash40::new("top"), 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 0.8, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, false);
        }
        5 => {
            // souffle autour de lui (REX : 2 coups spéciaux sur TopN, 14 %)
            PLAY_SE(fighter, Hash40::new("se_snake_final01"));
            for (id, ang, t) in [(0u64, 69u64, 6.0f32), (1, 80, 9.0)] {
                ATTACK(fighter, id, 0, Hash40::new("top"), 14.0, ang, 100, 168, 0, t * ECHELLE, 0.0, 0.0, 0.0, None, None, None, 0.3, 1.0,
                       *ATTACK_SETOFF_KIND_OFF, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true,
                       *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
                       Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_PUNCH, *ATTACK_REGION_BOMB);
            }
            fx_w(boma, "sys_smash_flash", s.x0, s.y0 + 8.0, 1.6, BLEU);
        }
        9 => {
            AttackModule::clear_all(boma);
            COL_NORMAL(fighter);
        }
        26..=38 => {
            // cible de REX (0x1FB0015) : 45 devant, 24 sous lui
            if (k - 26) % 3 == 0 {
                fx_w(boma, "sys_smash_flash_s", cx, cy, 1.6, ROUGE);
            }
        }
        39 => {
            // tir de la roquette (REX : son 58, effet 0xE00006)
            PLAY_SE(fighter, Hash40::new("se_snake_final02"));
            EFFECT_FOLLOW(fighter, Hash40::new("sys_muzzleflash"), Hash40::new("haver"), 0.0, 0.0, 8.0, 0.0, 0.0, 0.0, 1.6, true);
            EFFECT_FOLLOW(fighter, Hash40::new("sys_hit_fire"), Hash40::new("haver"), 0.0, 0.0, 8.0, 0.0, 0.0, 0.0, 0.8, true);
            // souffle arrière du lance-roquettes
            EFFECT(fighter, Hash40::new("sys_dash_smoke"), Hash40::new("haver"), 0.0, 0.0, -16.0, 0.0, 0.0, 0.0, 1.2, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, true);
            QUAKE(fighter, *CAMERA_QUAKE_KIND_S);
            let mut p = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
            ModelModule::joint_global_position(boma, Hash40::new("haver"), &mut p, true);
            ST[slot(boma)].dep = (p.x, p.y);
        }
        43 => {
            // explosion sur la cible (REX : 46 %, angle 48, base 255, taille 45, centre 45 devant / 24 dessous)
            PLAY_SE(fighter, Hash40::new("se_snake_final03"));
            PLAY_SE(fighter, Hash40::new("se_common_bomb_l"));
            fx_w(boma, "sys_bomb_a", cx, cy + 6.0, 3.6, (1.0, 1.0, 1.0));
            fx_w(boma, "sys_bomb_a", cx - 16.0, cy + 2.0, 2.2, (1.0, 1.0, 1.0));
            fx_w(boma, "sys_bomb_a", cx + 16.0, cy + 2.0, 2.2, (1.0, 1.0, 1.0));
            fx_w(boma, "sys_smash_flash", cx, cy + 10.0, 3.0, ORANGE);
            QUAKE(fighter, *CAMERA_QUAKE_KIND_XL);
            AttackModule::clear_all(boma);
            ATTACK(fighter, 0, 0, Hash40::new("top"), 46.0, 48, 100, 0, 255, 45.0 * ECHELLE, 0.0, -CIBLE_BAS, CIBLE_AV, None, None, None, 2.0, 1.0,
                   *ATTACK_SETOFF_KIND_OFF, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true,
                   *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
                   Hash40::new("collision_attr_fire"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_FIRE, *ATTACK_REGION_BOMB);
        }
        46 => {
            AttackModule::clear_all(boma);
            fx_w(boma, "sys_bomb_a", cx, cy + 22.0, 2.0, (1.0, 1.0, 1.0));
        }
        50 | 56 | 62 => {
            fx_w(boma, "sys_dash_smoke", cx + (k - 56) as f32 * 2.0, cy + 8.0, 2.5, (0.35, 0.35, 0.35));
        }
        80 => {
            EFFECT_OFF_KIND(fighter, Hash40::new("sys_final_aura"), false, false);
        }
        _ => {}
    }
}

unsafe extern "C" fn final_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    HitModule::set_whole(boma, HitStatus(*HIT_STATUS_NORMAL), 0);
    KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    AttackModule::clear_all(boma);
    EFFECT_OFF_KIND(fighter, Hash40::new("sys_final_aura"), false, false);
    COL_NORMAL(fighter);
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
    for m in ["finalstart", "finalairstart", "finalstartl", "finalairstartl"] {
        for p in ["game_", "effect_", "sound_", "expression_"] {
            let n: &'static str = Box::leak(format!("{}{}", p, m).into_boxed_str());
            a.acmd(n, vide, Priority::Low);
        }
    }
}
