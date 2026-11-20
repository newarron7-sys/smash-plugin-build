//! Final Smash de Primid « Dark Cannon » (REX : sous-action DarkCannon 476, 138 images à la vitesse 0,8 ;
//! méthode du Final Smash de Master Chief : statut FINAL de Mario remplacé, fin = statut de fin du jeu).
//! REX : caméra rapprochée au début, écran teinté violet à l'image 40, tir à l'image 83 (40 %, angle 80, base 100,
//! croissance 100), effets du Light Arrow de Zelda (rayon en forme de flèche). Animation : Primid accroupi, le canon
//! tenu à deux mains devant lui (images 27-82), tir, recul (110).
//! v0.45 (vidéo de REX envoyée par le joueur) : le canon apparaît dans un éclair (étincelles jaunes), Primid le soulève
//! puis le pose devant lui ; charge : étincelles jaune-orange au bout du canon, écran violet ; tir : gros rayon blanc-
//! rose en forme de flèche (40 %) ; Primid recule en glissant ; le canon disparaît. Canon = modèle du Dark Cannon de
//! Brawl (tools/canon_primid.py : objet pr_canon sur l'os Throw, placé entre ses mains à chaque image).

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
const PORTEE: f32 = 200.0;
const VIOLET: (f32, f32, f32) = (0.55, 0.1, 0.9);
const VIOLET_NOIR: (f32, f32, f32) = (0.25, 0.0, 0.42);
/// sons du Dark Cannon de REX (0xA323 / 0xA324 / 0xA325), mis dans des pistes de cris rarement jouées
const SON_CHARGE: &str = "vc_mario_heavyget";
const SON_TIR: &str = "vc_mario_swimup";
const SON_RAYON: &str = "vc_mario_furasleep";
const JAUNE: (f32, f32, f32) = (1.0, 0.8, 0.25);
const ROSE: (f32, f32, f32) = (1.0, 0.65, 0.9);
const BLANC: (f32, f32, f32) = (1.0, 1.0, 1.0);
/// recul après le tir (vidéo : il glisse loin en arrière)
const RECUL: f32 = 40.0;
const G_RECUL: i32 = 28;

#[derive(Clone, Copy)]
struct St {
    g: i32,
    /// v0.46 : images depuis le coup qui touche (ralenti + zoom) ; -1 : pas encore touché
    coup: i32,
    air: bool,
    aura: u32,
}
static mut ST: [St; 8] = [St { g: 0, coup: -1, air: false, aura: 0 }; 8];
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

/// mains de Primid (milieu des deux os de prise)
unsafe fn mains(boma: *mut BattleObjectModuleAccessor) -> (f32, f32) {
    let mut a = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    let mut b = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    ModelModule::joint_global_position(boma, Hash40::new("havel"), &mut a, true);
    ModelModule::joint_global_position(boma, Hash40::new("haver"), &mut b, true);
    ((a.x + b.x) / 2.0, (a.y + b.y) / 2.0)
}

/// bout du canon dans le monde (modèle : bouche à 15 devant la poignée)
unsafe fn canon(boma: *mut BattleObjectModuleAccessor) -> (f32, f32) {
    let lr = PostureModule::lr(boma);
    let (x, y) = mains(boma);
    (x + 17.0 * lr, y + 1.5)
}

/// canon (pr_canon, os Throw) entre ses mains, ou rangé
unsafe fn canon_montrer(boma: *mut BattleObjectModuleAccessor, on: bool) {
    if on {
        let (x, y) = mains(boma);
        ModelModule::set_joint_translate(boma, Hash40::new("throw"), &Vector3f { x, y, z: 0.0 }, true, false);
        ModelModule::set_joint_scale(boma, Hash40::new("throw"), &Vector3f { x: 1.0, y: 1.0, z: 1.0 });
    } else {
        ModelModule::clear_joint_srt(boma, Hash40::new("throw"));
    }
    ModelModule::set_mesh_visibility(boma, Hash40::new("pr_canon"), on);
}

unsafe extern "C" fn final_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = e_of(boma);
    // début du Final Smash annoncé au jeu (Master Chief v29 : sinon la Smash Ball reste)
    notify_event_msc_cmd!(fighter, Hash40::new_raw(0x201bc9217c));
    AreaModule::set_whole(boma, false);
    let air = StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR;
    ST[e] = St { g: 0, coup: -1, air, aura: 0 };
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
    // v0.47 (le joueur : « t'aurais juste pu faire le même zoom que Rock ») : zoom et ralenti du Final Smash de Rock
    // (gros plan de l'image 5 au tir, adversaires ralentis x4 jusqu'au tir)
    SLOW_OPPONENT(fighter, 4.0, G_TIR as f32);
    slow_others(boma, 4, G_TIR + 60);
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
    if EN_COURS[e] {
        canon_montrer(boma, true);
    }
    evenement(fighter, g);
    critique(fighter);
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
        ST[e].aura = fx_os(boma, "sys_final_aura", "hip", 1.1, (0.3, 0.9, 1.0));
        FLASH(fighter, 0.45, 0.1, 0.75, 0.35);
        PLAY_SE(fighter, Hash40::new("se_common_final_cutin"));
    }
    if g == 8 {
        COL_NORMAL(fighter);
    }
    // le canon apparaît dans un éclair (vidéo : étincelles jaunes)
    if g == 2 {
        let (x, y) = mains(boma);
        fx_w(boma, "sys_smash_flash", x + 8.0 * lr, y, 2.4, 0.0, JAUNE);
        fx_w(boma, "sys_item_arrival", x + 8.0 * lr, y, 1.6, 0.0, BLANC);
        // v0.47 : vrais sons du canon de REX (dans les pistes de cris, cf. build_vc_primid.py) : la charge (image 0)
        PLAY_SE(fighter, Hash40::new(SON_CHARGE));
    }
    // REX image 40 : écran teinté violet (fond du Final Smash de Mario recoloré)
    if g == G_TEINTE {
        let h = EffectModule::req_screen(boma, Hash40::new("bg_mario_final"), false, false, false) as u32;
        if h != 0 {
            EffectModule::set_rgb(boma, h, 0.6, 0.15, 0.9);
        }
        let p = *PostureModule::pos(boma);
        fx_w(boma, "sys_smash_flash", p.x, p.y + 8.0, 2.6, 0.0, VIOLET);
        for (dx, dy) in [(-12.0, 4.0), (12.0, 4.0), (0.0, 16.0)] {
            fx_w(boma, "sys_erace_smoke", p.x + dx, p.y + dy, 2.2, 0.0, VIOLET_NOIR);
        }
    }
    // charge : étincelles jaune-orange au bout du canon, de plus en plus grosses
    if g == 5 {
        let sc = PostureModule::scale(boma);
        CAM_ZOOM_IN_arg5(fighter, 5.0, 0.0, sc * 2.6, 0.0, 0.0);
    }
    if g == G_TIR {
        CAM_ZOOM_OUT(fighter);
    }
    if g >= G_TEINTE - 20 && g < G_TIR && g % 3 == 0 {
        let (cx, cy) = canon(boma);
        let k = ((g - G_TEINTE + 20) as f32 / (G_TIR - G_TEINTE + 20) as f32).clamp(0.0, 1.0);
        fx_w(boma, "sys_smash_flash_s", cx, cy, 0.6 + 1.8 * k, 0.0, JAUNE);
        if g % 9 == 0 {
            fx_w(boma, "sys_hit_elec_s", cx, cy, 0.5 + 0.8 * k, 0.0, JAUNE);
        }
    }
    if g == G_TIR {
        // tir (REX : 40 %, angle 80, base 100, croissance 100) : rayon blanc-rose en forme de flèche
        let (cx, cy) = canon(boma);
        // REX image 83 : la charge s'arrête, le tir et le rayon
        STOP_SE(fighter, Hash40::new(SON_CHARGE));
        PLAY_SE(fighter, Hash40::new(SON_TIR));
        PLAY_SE(fighter, Hash40::new(SON_RAYON));
        QUAKE(fighter, *CAMERA_QUAKE_KIND_L);
        fx_w(boma, "sys_bomb_a", cx, cy, 1.6, 0.0, ROSE);
        fx_w(boma, "sys_smash_flash", cx, cy, 3.4, 0.0, BLANC);
        let p = *PostureModule::pos(boma);
        let dy = cy - p.y;
        let dz = (cx - p.x) * lr;
        ATTACK(fighter, 0, 0, Hash40::new("top"), 40.0, 80, 100, 0, 100, 14.0, 0.0, dy, dz,
               Some(0.0), Some(dy), Some(dz + PORTEE), 2.0, 1.0,
               *ATTACK_SETOFF_KIND_OFF, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true,
               *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
               Hash40::new("collision_attr_purple"), *ATTACK_SOUND_LEVEL_LL, *COLLISION_SOUND_ATTR_BOMB, *ATTACK_REGION_ENERGY);
    }
    // rayon : pointe de flèche qui file, éclats blanc-rose le long de la ligne de tir
    if g >= G_TIR && g < G_TIR_FIN {
        let (cx, cy) = canon(boma);
        let t = (g - G_TIR) as f32 / (G_TIR_FIN - G_TIR) as f32;
        let n = 10;
        for k in 0..n {
            let d = (k as f32) * PORTEE / n as f32;
            if d > PORTEE * (t * 2.0).min(1.0) || (k + g) % 2 != 0 {
                continue;
            }
            let c = if k % 3 == 0 { BLANC } else { ROSE };
            fx_w(boma, "sys_smash_flash", cx + lr * d, cy, 1.8, 0.0, c);
        }
        // pointe
        let tete = cx + lr * PORTEE * (t * 2.0).min(1.0);
        fx_w(boma, "sys_smash_flash", tete, cy, 3.0, 0.0, BLANC);
        if g % 3 == 0 {
            for k in 0..5 {
                let x = cx + lr * (10.0 + k as f32 * 40.0);
                fx_w(boma, "sys_attack_speedline", x, cy, 2.2, if lr > 0.0 { -1.5708 } else { 1.5708 }, ROSE);
            }
        }
    }
    if g == G_TIR_FIN {
        AttackModule::clear_all(boma);
    }
    // recul : il glisse en arrière (vidéo de REX), sans tomber du bord
    if g > G_TIR && g <= G_TIR + G_RECUL && !ST[e].air {
        let k = (g - G_TIR) as f32 / G_RECUL as f32;
        let v = RECUL * 2.0 / G_RECUL as f32 * (1.0 - k);
        let p = *PostureModule::pos(boma);
        let nx = p.x - v * lr;
        let sol = GroundModule::ray_check(boma, &smash::phx::Vector2f { x: nx, y: p.y + 3.0 }, &smash::phx::Vector2f { x: 0.0, y: -8.0 }, true) == 1;
        if sol {
            PostureModule::set_pos(boma, &Vector3f { x: nx, y: p.y, z: p.z });
            if g % 4 == 0 {
                fx_w(boma, "sys_dash_smoke", p.x, p.y, 0.8, 0.0, BLANC);
            }
        }
    }
    if g == G_FIN - 25 {
        let (x, y) = mains(boma);
        fx_w(boma, "sys_erace_smoke", x + 6.0 * lr, y, 1.4, 0.0, VIOLET_NOIR);
        canon_montrer(boma, false);
        EN_COURS[e] = false;
        EFFECT_OFF_KIND(fighter, Hash40::new("sys_final_aura"), false, false);
        EffectModule::remove_screen(boma, Hash40::new("bg_mario_final"), 10);
        ST[e].aura = 0;
    }
}

/// ralenti des autres combattants (copie de slow_others de Rock)
unsafe fn slow_others(boma: *mut BattleObjectModuleAccessor, mag: u8, frames: i32) {
    let me = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID);
    for e in 0..8 {
        if e == me {
            continue;
        }
        let id = smash::app::Fighter::get_id_from_entry_id(e);
        if id == 0 || id == *BATTLE_OBJECT_ID_INVALID as u32 {
            continue;
        }
        let ob = smash::app::sv_battle_object::module_accessor(id);
        if ob.is_null() {
            continue;
        }
        SlowModule::set_whole(ob, mag, frames);
    }
}

/// coup qui touche : ralenti + zoom + fond « coup critique » (méthode du mode critique d'Ultimate S)
unsafe fn critique(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let e = e_of(boma);
    let g = ST[e].g;
    if ST[e].coup < 0 {
        if g >= G_TIR && g <= G_TIR_FIN && AttackModule::is_infliction_status(boma, *COLLISION_KIND_MASK_HIT) {
            ST[e].coup = 0;
            SlowModule::set_whole(boma, 8, 80);
            CAM_ZOOM_IN_arg5(fighter, 2.0, 0.0, 1.8, 0.0, 0.0);
            EffectModule::req_follow(boma, Hash40::new("sys_bg_criticalhit"), Hash40::new("top"), &Vector3f { x: 0.0, y: 0.0, z: 0.0 },
                                     &Vector3f { x: 0.0, y: 0.0, z: 0.0 }, 1.0, false, 0, 0, 0, 0, 0, false, false);
            PLAY_SE(fighter, Hash40::new("se_common_criticalhit"));
            QUAKE(fighter, *CAMERA_QUAKE_KIND_XL);
        }
        return;
    }
    ST[e].coup += 1;
    if ST[e].coup == 8 {
        critique_fin(fighter);
    }
}

unsafe fn critique_fin(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    SlowModule::clear_whole(boma);
    CameraModule::reset_all(boma);
    EffectModule::kill_kind(boma, Hash40::new("sys_bg_criticalhit"), false, false);
    CAM_ZOOM_OUT(fighter);
}

unsafe extern "C" fn final_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = e_of(boma);
    HitModule::set_whole(boma, HitStatus(*HIT_STATUS_NORMAL), 0);
    KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    AttackModule::clear_all(boma);
    EFFECT_OFF_KIND(fighter, Hash40::new("sys_final_aura"), false, false);
    COL_NORMAL(fighter);
    canon_montrer(boma, false);
    STOP_SE(fighter, Hash40::new(SON_CHARGE));
    EffectModule::remove_screen(boma, Hash40::new("bg_mario_final"), 10);
    if ST[e].coup >= 0 && ST[e].coup < 8 {
        critique_fin(fighter);
    }
    ST[e].coup = -1;
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
