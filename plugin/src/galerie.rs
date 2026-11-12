//! Galerie d'effets (outil de réglage, provisoire) : je ne vois pas les effets du jeu, deux choix à l'aveugle étaient
//! faux (sys_direction2 : zigzag blanc ; sys_bossgalaga_beam : rien). Une provocation (taunt) lance un défilé : un
//! effet commun du jeu toutes les 45 images, posé devant Chief, dans l'ordre de LISTE ; éclat rouge au-dessus tous les
//! 5 effets (repère pour compter sur la vidéo). Le joueur filme une fois, les bons effets sont choisis à vue.

use smash::app::lua_bind::*;
use smash::app::*;
use smash::lib::lua_const::*;
use smash::lua2cpp::*;
use smash::phx::{Hash40, Vector3f};

pub const LISTE: &[&str] = &[
    "sys_muzzleflash", "sys_rocketbelt_jet", "sys_rocketbelt_burner", "sys_dolphinbomb_jet", "sys_bossgalaga_beam",
    "sys_mball_beam", "sys_genesis_beam", "sys_staff_shot", "sys_staff_light", "sys_specialflag_light",
    "sys_special_all_up", "sys_special_defense_up", "sys_special_speed_up", "sys_special_attack_up", "sys_spirits_up_s",
    "sys_final_aura", "sys_final_aura2", "sys_thunder", "sys_recovery", "sys_teamhealfield",
    "sys_counter_flash", "sys_sp_flash", "sys_flash", "sys_raygun_bullet", "sys_raygun_shot",
    "sys_sscope_bullet", "sys_starrod_light", "sys_killsword_light1", "sys_magicball_light", "sys_magicball_aura",
    "sys_damage_aura", "sys_hit_aura_s", "sys_warpstar_trace", "sys_club_flare", "sys_flame",
    "sys_freezer", "sys_killer_jetsmoke", "sys_windwave", "sys_spin_wind", "sys_whirlwind_l",
];
const PAS: i32 = 45;
static mut T: [i32; 8] = [-1; 8];
static mut H: [u32; 8] = [0; 8];
static mut POS: [(f32, f32); 8] = [(0.0, 0.0); 8];

pub unsafe fn chaque_image(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let e = (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7);
    if T[e] < 0 {
        if StatusModule::status_kind(boma) == *FIGHTER_STATUS_KIND_APPEAL && MotionModule::frame(boma) <= 1.0 {
            let p = PostureModule::pos(boma);
            POS[e] = ((*p).x + 16.0 * PostureModule::lr(boma), (*p).y);
            T[e] = 0;
        } else {
            return;
        }
    }
    let t = T[e];
    T[e] += 1;
    if t % PAS != 0 {
        return;
    }
    if H[e] != 0 {
        EffectModule::kill(boma, H[e], true, true);
        H[e] = 0;
    }
    let i = (t / PAS) as usize;
    if i >= LISTE.len() {
        T[e] = -1;
        return;
    }
    let z = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    let p = Vector3f { x: POS[e].0, y: POS[e].1 + 5.0, z: 0.0 };
    H[e] = EffectModule::req(boma, Hash40::new(LISTE[i]), &p, &z, 1.0, 0, 0, false, 0) as u32;
    if i % 5 == 0 {
        let m = Vector3f { x: POS[e].0, y: POS[e].1 + 34.0, z: 0.0 };
        let h = EffectModule::req(boma, Hash40::new("sys_smash_flash"), &m, &z, 0.9, 0, 0, false, 0) as u32;
        if h != 0 {
            EffectModule::set_rgb(boma, h, 1.0, 0.1, 0.1);
        }
    }
}
