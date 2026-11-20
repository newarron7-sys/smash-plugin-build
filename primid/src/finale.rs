//! Final Smash de Primid « Dark Cannon » (REX : sous-action DarkCannon 476, 138 images à la vitesse 0,8 ;
//! méthode du Final Smash de Master Chief : statut FINAL de Mario remplacé, fin = statut de fin du jeu).
//! REX : caméra rapprochée au début, écran teinté violet à l'image 40, tir à l'image 83 (40 %, angle 80, base 100,
//! croissance 100), effets du Light Arrow de Zelda (rayon en forme de flèche). Animation : Primid accroupi, le canon
//! tenu à deux mains devant lui (images 27-82), tir, recul (110).
//! Ici : le canon = son Super Scope agrandi ; charge violette au bout du canon, puis un rayon violet foncé qui traverse
//! l'écran devant lui (un long coup en capsule pendant 12 images).

use smash::app::lua_bind::*;
use smash::app::*;
use smash::lib::lua_const::*;
use smash::lib::L2CValue;
use smash::lua2cpp::*;
use smash::phx::{Hash40, Vector3f};
use smash_script::macros::*;
use smash_script::*;
use smashline::*;

fn e_of(boma: *mut BattleObjectModuleAccessor) -> usize {
    unsafe { (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7) }
}

const VITESSE: f32 = 0.8;
/// images du jeu (images de REX / 0,8)
const G_TEINTE: i32 = 50;
const G_TIR: i32 = 104;
const G_TIR_FIN: i32 = 116;
const G_FIN: i32 = 173;
/// rayon : du bout du canon jusqu'à PORTEE devant Primid
const CANON_Y: f32 = 6.0;
const CANON_Z: f32 = 12.0;
const PORTEE: f32 = 200.0;
const VIOLET: (f32, f32, f32) = (0.55, 0.1, 0.9);
const VIOLET_NOIR: (f32, f32, f32) = (0.25, 0.0, 0.42);

#[derive(Clone, Copy)]
struct St {
    g: i32,
    air: bool,
    aura: u32,
}
static mut ST: [St; 8] = [St { g: 0, air: false, aura: 0 }; 8];
/// lu par speciaux.rs : Final Smash en cours (canon en main)
pub static mut EN_COURS: [bool; 8] = [false; 8];

unsafe fn fx_w(boma: *mut BattleObjectModuleAccessor, name: &str, x: f32, y: f32, sc: f32, rot_z: f32, c: (f32, f32, f32)) {
    let pos = Vector3f { x, y, z: 0.0 };
    // rotations des effets en RADIANS (correction de Master Chief)
    let rot = Vector3f { x: 0.0, y: 0.0, z: rot_z };
    let h = EffectModule::req(boma, Hash40::new(name), &pos, &rot, sc, 0, 0, false, 0) as u32;
    if h != 0 {
        EffectModule::set_rgb(boma, h, c.0, c.1, c.2);
    }
}

unsafe fn fx_os(boma: *mut BattleObjectModuleAccessor, name: &str, os: &str, sc: f32, c: (f32, f32, f32)) -> u32 {
    let p = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    let r = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    let h = EffectModule::req_follow(boma, Hash40::new(name), Hash40::new(os), &p, &r, sc, true, 0, 0, 0, 0, 0, false, false) as u32;
    if h != 0 {
        EffectModule::set_rgb(boma, h, c.0, c.1, c.2);
    }
    h
}

/// bout du canon dans le monde
unsafe fn canon(boma: *mut BattleObjectModuleAccessor) -> (f32, f32) {
    let p = *PostureModule::pos(boma);
    let lr = PostureModule::lr(boma);
    (p.x + CANON_Z * lr, p.y + CANON_Y)
}

unsafe fn canon_taille(boma: *mut BattleObjectModuleAccessor, s: f32) {
    for os in ["havel", "haver"] {
        if s == 1.0 {
            ModelModule::clear_joint_srt(boma, Hash40::new(os));
        } else {
            ModelModule::set_joint_scale(boma, Hash40::new(os), &Vector3f { x: s, y: s, z: s });
        }
    }
}

unsafe extern "C" fn final_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = e_of(boma);
    // début du Final Smash annoncé au jeu (Master Chief v29 : sinon la Smash Ball reste)
    notify_event_msc_cmd!(fighter, Hash40::new_raw(0x201bc9217c));
    AreaModule::set_whole(boma, false);
    let air = StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR;
    ST[e] = St { g: 0, air, aura: 0 };
    EN_COURS[e] = true;
    if air {
        fighter.set_situation(SITUATION_KIND_AIR.into());
        GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_AIR_STOP);
        KineticModule::unable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    } else {
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_GROUND_STOP);
        GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_GROUND_CLIFF_STOP));
    }
    sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0_f32, 0.0_f32);
    // REX : intangible pendant tout le Final Smash
    HitModule::set_whole(boma, HitStatus(*HIT_STATUS_XLU), 0);
    let mot = if air { "final_air" } else { "final" };
    MotionModule::change_motion(boma, Hash40::new(mot), 0.0, VITESSE, false, 0.0, false, false);
    fighter.sub_shift_status_main(L2CValue::Ptr(final_loop as *const () as _))
}

unsafe extern "C" fn final_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = e_of(boma);
    ST[e].g += 1;
    let g = ST[e].g;
    if MotionModule::frame(boma) >= MotionModule::end_frame(boma) - 1.0 {
        MotionModule::set_rate(boma, 0.0);
    }
    sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0_f32, 0.0_f32);
    evenement(fighter, g);
    if g >= G_FIN {
        let st = if ST[e].air { FIGHTER_STATUS_KIND_FALL } else { FIGHTER_STATUS_KIND_WAIT };
        fighter.change_status(st.into(), false.into());
    }
    0.into()
}

unsafe fn evenement(fighter: &mut L2CFighterCommon, g: i32) {
    let boma = fighter.module_accessor;
    let e = e_of(boma);
    let lr = PostureModule::lr(boma);
    if g == 1 {
        ST[e].aura = fx_os(boma, "sys_final_aura", "hip", 1.1, VIOLET);
        FLASH(fighter, 0.45, 0.1, 0.75, 0.35);
        PLAY_SE(fighter, Hash40::new("se_common_final_cutin"));
    }
    if g == 8 {
        COL_NORMAL(fighter);
    }
    // le canon sort (REX : il le prend en main en s'accroupissant)
    if g == 20 {
        canon_taille(boma, 1.7);
        fx_os(boma, "sys_erace_smoke", "havel", 0.8, VIOLET_NOIR);
    }
    // REX image 40 : écran teinté violet ; ici : fumée violet foncé autour de lui et éclat
    if g == G_TEINTE {
        let p = *PostureModule::pos(boma);
        fx_w(boma, "sys_smash_flash", p.x, p.y + 8.0, 3.0, 0.0, VIOLET);
        for (dx, dy) in [(-10.0, 4.0), (10.0, 4.0), (0.0, 14.0)] {
            fx_w(boma, "sys_erace_smoke", p.x + dx, p.y + dy, 2.2, 0.0, VIOLET_NOIR);
        }
    }
    // charge au bout du canon (de plus en plus grosse)
    if g >= G_TEINTE && g < G_TIR && g % 4 == 0 {
        let (cx, cy) = canon(boma);
        let k = (g - G_TEINTE) as f32 / (G_TIR - G_TEINTE) as f32;
        fx_w(boma, "sys_smash_flash_s", cx, cy, 0.6 + 1.6 * k, 0.0, VIOLET);
        if g % 8 == 0 {
            fx_w(boma, "sys_erace_smoke", cx, cy, 0.3 + 0.6 * k, 0.0, VIOLET_NOIR);
        }
    }
    if g == G_TIR {
        // tir (REX : 40 %, angle 80, base 100, croissance 100) : rayon qui traverse l'écran
        let (cx, cy) = canon(boma);
        PLAY_SE(fighter, Hash40::new("se_common_bomb_l"));
        PLAY_SE(fighter, Hash40::new("se_common_final_shot"));
        QUAKE(fighter, *CAMERA_QUAKE_KIND_L);
        fx_w(boma, "sys_bomb_a", cx, cy, 1.8, 0.0, VIOLET);
        fx_w(boma, "sys_smash_flash", cx, cy, 3.2, 0.0, (0.8, 0.5, 1.0));
        ATTACK(fighter, 0, 0, Hash40::new("top"), 40.0, 80, 100, 0, 100, 14.0, 0.0, CANON_Y, CANON_Z,
               Some(0.0), Some(CANON_Y), Some(CANON_Z + PORTEE), 2.0, 1.0,
               *ATTACK_SETOFF_KIND_OFF, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true,
               *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
               Hash40::new("collision_attr_purple"), *ATTACK_SOUND_LEVEL_LL, *COLLISION_SOUND_ATTR_BOMB, *ATTACK_REGION_ENERGY);
    }
    // rayon : éclats violets et traînées le long de la ligne de tir
    if g >= G_TIR && g < G_TIR_FIN {
        let (cx, cy) = canon(boma);
        let n = 10;
        for k in 0..n {
            if (k + g) % 2 != 0 {
                continue;
            }
            let x = cx + lr * (k as f32) * PORTEE / n as f32;
            let c = if k % 4 == 0 { VIOLET } else { VIOLET_NOIR };
            fx_w(boma, "sys_smash_flash", x, cy, 1.7, 0.0, c);
        }
        if g % 3 == 0 {
            for k in 0..5 {
                let x = cx + lr * (10.0 + k as f32 * 40.0);
                // traînée couchée le long du rayon (rotation en radians)
                fx_w(boma, "sys_attack_speedline", x, cy, 2.0, if lr > 0.0 { -1.5708 } else { 1.5708 }, VIOLET);
            }
        }
    }
    if g == G_TIR_FIN {
        AttackModule::clear_all(boma);
    }
    // recul de l'animation (REX image 110) : fumée au canon
    if g == G_TIR_FIN + 8 {
        fx_os(boma, "sys_erace_smoke", "havel", 1.0, VIOLET_NOIR);
    }
    if g == G_FIN - 25 {
        canon_taille(boma, 1.0);
        EN_COURS[e] = false;
        EFFECT_OFF_KIND(fighter, Hash40::new("sys_final_aura"), false, false);
        ST[e].aura = 0;
    }
}

unsafe extern "C" fn final_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = e_of(boma);
    HitModule::set_whole(boma, HitStatus(*HIT_STATUS_NORMAL), 0);
    KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    AttackModule::clear_all(boma);
    EFFECT_OFF_KIND(fighter, Hash40::new("sys_final_aura"), false, false);
    COL_NORMAL(fighter);
    canon_taille(boma, 1.0);
    EN_COURS[e] = false;
    ST[e].aura = 0;
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
    // scripts du Final Smash de Mario (Mario Finale : boules de feu, caméra) : vidés
    for m in ["final", "finalair"] {
        for p in ["game_", "effect_", "sound_", "expression_"] {
            let n: &'static str = Box::leak(format!("{}{}", p, m).into_boxed_str());
            a.acmd(n, vide, Priority::Low);
        }
    }
}
