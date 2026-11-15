//! Final Smash de Metal Sonic (REX) : il devient Neo Metal Sonic.
//! Dans REX (FitMetalSonic.pac, sous-actions FinalStart / FinalEnd + FitMetalSonicFinal.pac) c'est le Landmaster de
//! Wolf avec le modèle « Neo_Metal_Sonic » posé dessus (figé, une seule pose) :
//! 1. FinalStart : pose de Metal (images 0-50), aura et éclats à partir de 40, sa voix à 52 ;
//! 2. image 50 : Metal disparaît (0B020100 faux), le véhicule / Neo apparaît (article 3) ;
//! 3. image 70 : énorme souffle autour de lui (REX : 3 % puis 16 %, angle 361, 90 / 80, tailles 60 et 70 ; effet
//!    n° 129 à 30 de haut, taille 5 ; quatre explosions, son n° 92) ;
//! 4. Landmaster : il roule à gauche / à droite, saute et plane (saut tenu), tire (attaque ou spécial : canon du
//!    Landmaster, ici le rayon de Neo), fait un tonneau (bouclier) ; il pousse et frappe ceux qu'il percute ;
//! 5. FinalEnd : Neo disparaît, Metal revient (effet n° 44 à 8 de haut, son n° 58).
//! Même méthode que le Final Smash de Master Chief (finale.rs) : statut FINAL de Sonic remplacé (Main / Exec / End),
//! début annoncé au jeu (sinon la Smash Ball reste), scripts de Super Sonic vidés.
//! Neo : objets du modèle du skin (add_neo.py), os NeoN sous Trans ; montrés par la piste de visibilité des animations
//! du Final Smash (neo_vis.py). Metal est caché en réduisant son os Rot (tout son corps est dessous, pas NeoN).

use smash::app::lua_bind::*;
use smash::app::*;
use smash::lib::lua_const::*;
use smash::lib::L2CValue;
use smash::lua2cpp::*;
use smash::phx::{Hash40, Vector3f};
use smash_script::macros::*;
use smash_script::*;
use smashline::*;

unsafe fn slot(boma: *mut BattleObjectModuleAccessor) -> usize {
    (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7)
}

// ------------------------------------------------------------------ réglages (images de REX)
const G_AURA: i32 = 40;
const G_NEO: i32 = 50;
const G_BOUM: i32 = 70;
/// durée du Landmaster après le souffle (Wolf : une douzaine de secondes)
const DUREE: i32 = 660;
const G_FIN_ANIM: f32 = 119.0;
const FIN_DUREE: i32 = 26;
/// déplacement : vitesse au sol / en l'air, accélération, poussée du saut, gravité, plafond au-dessus du départ
const V_MAX: f32 = 1.9;
const V_AIR: f32 = 1.5;
const ACCEL: f32 = 0.18;
const POUSSEE: f32 = 0.42;
const VY_MAX: f32 = 2.4;
const GRAVITE: f32 = 0.11;
const VY_CHUTE: f32 = -2.6;
const PLAFOND: f32 = 70.0;
/// tir : cadence (REX FinalFire 40 images), rayon devant lui à hauteur du torse de Neo
const TIR_CADENCE: i32 = 40;
const TIR_DEPART: i32 = 8;
const RAYON_Y: f32 = 20.0;
const RAYON_DEBUT: f32 = 10.0;
const RAYON_LONG: f32 = 90.0;
/// tonneau (REX FinalRoll 50 images) : un tour complet en 30 images, en fonçant
const TONNEAU: i32 = 30;

const BLEU: (f32, f32, f32) = (0.45, 0.65, 1.0);
const ROSE: (f32, f32, f32) = (1.0, 0.35, 0.85);
const BLANC: (f32, f32, f32) = (1.0, 1.0, 1.0);
const AURA: [(f32, f32, f32); 4] = [(0.45, 0.65, 1.0), (0.75, 0.3, 1.0), (1.0, 0.35, 0.85), (0.3, 0.9, 1.0)];

#[derive(Clone, Copy)]
struct St {
    g: i32,
    vx: f32,
    vy: f32,
    y0: f32,
    tir: i32,
    tonneau: i32,
    aura: u32,
    fin: i32,
    contact: i32,
}
const ST0: St = St { g: 0, vx: 0.0, vy: 0.0, y0: 0.0, tir: 0, tonneau: 0, aura: 0, fin: 0, contact: 0 };
static mut ST: [St; 8] = [ST0; 8];
/// Metal caché (os Rot réduit) : remis à la normale par chaque_image si le Final Smash est interrompu
static mut CACHE: [bool; 8] = [false; 8];

unsafe fn fx_w(boma: *mut BattleObjectModuleAccessor, name: &str, x: f32, y: f32, sc: f32, c: (f32, f32, f32)) {
    let pos = Vector3f { x, y, z: 0.0 };
    let rot = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    let h = EffectModule::req(boma, Hash40::new(name), &pos, &rot, sc, 0, 0, false, 0) as u32;
    if h != 0 {
        EffectModule::set_rgb(boma, h, c.0, c.1, c.2);
    }
}

unsafe fn fx_top(boma: *mut BattleObjectModuleAccessor, name: &str, p: (f32, f32, f32), sc: f32, c: (f32, f32, f32)) -> u32 {
    let v = Vector3f { x: p.0, y: p.1, z: p.2 };
    let r = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    let h = EffectModule::req_follow(boma, Hash40::new(name), Hash40::new("top"), &v, &r, sc, true, 0, 0, 0, 0, 0, false, false) as u32;
    if h != 0 {
        EffectModule::set_rgb(boma, h, c.0, c.1, c.2);
    }
    h
}

unsafe fn metal_cache(boma: *mut BattleObjectModuleAccessor, oui: bool) {
    let k = if oui { 0.001 } else { 1.0 };
    ModelModule::set_joint_scale(boma, Hash40::new("rot"), &Vector3f { x: k, y: k, z: k });
    CACHE[slot(boma)] = oui;
}

unsafe fn tourne_neo(boma: *mut BattleObjectModuleAccessor, angle: f32) {
    let r = Vector3f { x: 0.0, y: 0.0, z: angle };
    ModelModule::set_joint_rotate(boma, Hash40::new("neon"), &r,
        MotionNodeRotateCompose { _address: *MOTION_NODE_ROTATE_COMPOSE_AFTER as u8 },
        MotionNodeRotateOrder { _address: *MOTION_NODE_ROTATE_ORDER_XYZ as u8 });
}

unsafe fn en_air(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    fighter.set_situation(SITUATION_KIND_AIR.into());
    GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
    KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_AIR_STOP);
    KineticModule::unable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_STOP);
}

#[allow(clippy::too_many_arguments)]
unsafe fn coup(fighter: &mut L2CFighterCommon, id: u64, dmg: f32, ang: u64, kbg: i32, bkb: i32, taille: f32,
               p: (f32, f32), p2: Option<(f32, f32)>, attr: &str, son: i32) {
    let (y2, z2) = match p2 { Some((y, z)) => (Some(y), Some(z)), None => (None, None) };
    ATTACK(fighter, id, 0, Hash40::new("top"), dmg, ang, kbg, 0, bkb, taille, 0.0, p.0, p.1,
           if p2.is_some() { Some(0.0) } else { None }, y2, z2, 1.5, 1.0,
           *ATTACK_SETOFF_KIND_OFF, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true,
           *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
           Hash40::new(attr), *ATTACK_SOUND_LEVEL_L, son, *ATTACK_REGION_BOMB);
}

unsafe extern "C" fn final_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    notify_event_msc_cmd!(fighter, Hash40::new_raw(0x201bc9217c));
    AreaModule::set_whole(boma, false);
    let air = StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR;
    let p = *PostureModule::pos(boma);
    ST[slot(boma)] = St { y0: p.y, ..ST0 };
    if air {
        en_air(fighter);
    } else {
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_GROUND_STOP);
        GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_GROUND_CLIFF_STOP));
    }
    sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0_f32, 0.0_f32);
    HitModule::set_whole(boma, HitStatus(*HIT_STATUS_XLU), 0);
    let mot = if air { "final_air_start" } else { "final_start" };
    MotionModule::change_motion(boma, Hash40::new(mot), 0.0, 1.0, false, 0.0, false, false);
    fighter.sub_shift_status_main(L2CValue::Ptr(final_loop as *const () as _))
}

unsafe extern "C" fn final_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    ST[e].g += 1;
    let g = ST[e].g;
    // l'animation (Neo montré à partir de 50) reste figée à la fin pendant tout le Landmaster
    if ST[e].fin == 0 && MotionModule::frame(boma) >= G_FIN_ANIM {
        MotionModule::set_rate(boma, 0.0);
        MotionModule::set_frame(boma, G_FIN_ANIM, false);
    }
    if g < G_NEO {
        sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0_f32, 0.0_f32);
    }
    debut(fighter, g);
    if g >= G_NEO && ST[e].fin == 0 {
        metal_cache(boma, true);
    }
    if g > G_BOUM && ST[e].fin == 0 {
        landmaster(fighter);
        if g >= G_BOUM + DUREE {
            fin(fighter);
        }
    }
    if ST[e].fin > 0 {
        ST[e].fin += 1;
        sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0_f32, 0.0_f32);
        if ST[e].fin >= FIN_DUREE {
            let st = if StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR { FIGHTER_STATUS_KIND_FALL } else { FIGHTER_STATUS_KIND_WAIT };
            fighter.change_status(st.into(), false.into());
            return 1.into();
        }
    }
    0.into()
}

/// FinalStart de REX : aura, éclats, disparition de Metal, apparition de Neo, souffle
unsafe fn debut(fighter: &mut L2CFighterCommon, g: i32) {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    if g == 1 {
        FLASH(fighter, 0.45, 0.6, 1.0, 0.35);
    }
    if g == 8 {
        COL_NORMAL(fighter);
    }
    if g == G_AURA {
        PLAY_SE(fighter, Hash40::new("se_sonic_final01"));
        ST[e].aura = fx_top(boma, "sys_final_aura", (0.0, 8.0, 0.0), 1.2, AURA[0]);
    }
    if g >= G_AURA && g <= G_NEO && (g - G_AURA) % 5 == 0 {
        fx_top(boma, "sys_smash_flash_s", (0.0, 8.0, 0.0), 1.3, BLEU);
        fx_top(boma, "sys_hit_elec", (0.0, 8.0, 0.0), 0.8, ROSE);
    }
    if ST[e].aura != 0 && g % 8 == 0 {
        let c = AURA[((g / 8) as usize) % AURA.len()];
        EffectModule::set_rgb(boma, ST[e].aura, c.0, c.1, c.2);
    }
    if g == G_NEO {
        // il devient Neo Metal Sonic : éclat et onde
        let p = *PostureModule::pos(boma);
        fx_w(boma, "sys_smash_flash", p.x, p.y + 15.0, 3.0, BLANC);
        fx_w(boma, "sys_bomb_b", p.x, p.y + 12.0, 1.6, BLEU);
        LANDING_EFFECT(fighter, Hash40::new("sys_crown"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 1.2, 0, 0, 0, 0, 0, 0, false);
        QUAKE(fighter, *CAMERA_QUAKE_KIND_M);
    }
    if g == G_NEO + 2 {
        PLAY_SE(fighter, Hash40::new(crate::sons::V_CHARGE));
    }
    if g == G_BOUM {
        // REX : souffle (3 % puis 16 %, angle 361, tailles 60 et 70), effet n° 129 à 30 de haut, quatre explosions
        let p = *PostureModule::pos(boma);
        PLAY_SE(fighter, Hash40::new("se_sonic_final03"));
        PLAY_SE(fighter, Hash40::new("se_common_bomb_l"));
        fx_w(boma, "sys_smash_flash", p.x, p.y + 30.0, 4.0, (0.8, 0.85, 1.0));
        fx_w(boma, "sys_bomb_a", p.x, p.y + 20.0, 3.5, BLANC);
        for (dx, dy) in [(-22.0, 10.0), (22.0, 10.0), (0.0, 40.0)] {
            fx_w(boma, "sys_bomb_a", p.x + dx, p.y + dy, 2.0, BLANC);
        }
        LANDING_EFFECT(fighter, Hash40::new("sys_crown_collision"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 2.0, 0, 0, 0, 0, 0, 0, false);
        QUAKE(fighter, *CAMERA_QUAKE_KIND_XL);
        coup(fighter, 0, 3.0, 69, 50, 30, 60.0, (5.0, 0.0), None, "collision_attr_normal", *COLLISION_SOUND_ATTR_FIRE);
    }
    if g == G_BOUM + 5 {
        AttackModule::clear_all(boma);
        coup(fighter, 1, 16.0, 361, 90, 80, 70.0, (5.0, 0.0), None, "collision_attr_normal", *COLLISION_SOUND_ATTR_FIRE);
    }
    if g == G_BOUM + 10 {
        AttackModule::clear_all(boma);
        EFFECT_OFF_KIND(fighter, Hash40::new("sys_final_aura"), false, false);
        ST[e].aura = 0;
        en_air(fighter);
    }
}

/// le Landmaster de Wolf avec Neo dessus : rouler, sauter / planer, tirer, tonneau
unsafe fn landmaster(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    let s = ST[e];
    let sol = GroundModule::is_touch(boma, *GROUND_TOUCH_FLAG_DOWN as u32);
    let sx = ControlModule::get_stick_x(boma);
    let mut lr = PostureModule::lr(boma);
    // demi-tour avec le stick (pas pendant un tir ni un tonneau)
    if sx * lr < -0.3 && s.tir == 0 && s.tonneau == 0 {
        PostureModule::reverse_lr(boma);
        PostureModule::update_rot_y_lr(boma);
        lr = -lr;
        ST[e].vx = -ST[e].vx * 0.3;
    }
    // vitesse horizontale (x dans sa direction)
    let vmax = if sol { V_MAX } else { V_AIR };
    let cible = if s.tonneau > 0 { V_MAX * 1.6 } else if sx.abs() > 0.2 { sx.abs() * vmax } else { 0.0 };
    let vx = ST[e].vx + (cible - ST[e].vx).clamp(-ACCEL, ACCEL);
    ST[e].vx = vx;
    // saut / vol plané : saut tenu (ou stick en haut) = poussée vers le haut, sinon il retombe
    let monte = ControlModule::check_button_on(boma, *CONTROL_PAD_BUTTON_JUMP) || ControlModule::get_stick_y(boma) > 0.6;
    let haut = (*PostureModule::pos(boma)).y - s.y0;
    let mut vy = ST[e].vy;
    if monte && haut < PLAFOND {
        if sol && vy <= 0.0 {
            vy = 1.6;
            LANDING_EFFECT(fighter, Hash40::new("sys_jump_smoke"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 1.4, 0, 0, 0, 0, 0, 0, false);
        }
        vy = (vy + POUSSEE).min(VY_MAX);
        if s.g % 4 == 0 {
            fx_top(boma, "sys_flame", (0.0, 2.0, -6.0), 0.8, (0.7, 0.4, 1.0));
        }
    } else {
        vy = (vy - GRAVITE).max(VY_CHUTE);
    }
    if sol && vy < 0.0 {
        vy = 0.0;
    }
    if haut >= PLAFOND && vy > 0.0 {
        vy = 0.0;
    }
    ST[e].vy = vy;
    sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, vx * lr, vy);
    // poussière / flammes du réacteur quand il roule
    if sol && vx.abs() > 0.6 && s.g % 6 == 0 {
        FOOT_EFFECT(fighter, Hash40::new("sys_dash_smoke"), Hash40::new("top"), -8, 0, 0, 0, 0, 0, 1.0, 0, 0, 0, 0, 0, 0, false);
    }
    if vx.abs() > 0.6 && s.g % 3 == 0 {
        fx_top(boma, "sys_flame", (0.0, 16.0, -9.0), 0.5, (0.7, 0.4, 1.0));
    }
    // tir : attaque ou spécial
    let appui = ControlModule::check_button_trigger(boma, *CONTROL_PAD_BUTTON_ATTACK)
        || ControlModule::check_button_trigger(boma, *CONTROL_PAD_BUTTON_SPECIAL);
    if s.tir == 0 && s.tonneau == 0 && appui {
        ST[e].tir = 1;
    }
    if ST[e].tir > 0 {
        tir(fighter, ST[e].tir);
        ST[e].tir += 1;
        if ST[e].tir > TIR_CADENCE {
            ST[e].tir = 0;
        }
    }
    // tonneau : bouclier
    if s.tonneau == 0 && s.tir == 0 && ControlModule::check_button_trigger(boma, *CONTROL_PAD_BUTTON_GUARD) {
        ST[e].tonneau = 1;
        PLAY_SE(fighter, Hash40::new(crate::sons::REACTEUR));
        AttackModule::clear_all(boma);
        coup(fighter, 2, 10.0, 50, 80, 60, 13.0, (14.0, 2.0), None, "collision_attr_normal", *COLLISION_SOUND_ATTR_KICK);
    }
    if ST[e].tonneau > 0 {
        let k = ST[e].tonneau;
        tourne_neo(boma, 360.0 * k as f32 / TONNEAU as f32);
        if k % 3 == 0 {
            fx_top(boma, "sys_attack_speedline", (0.0, 14.0, -6.0), 1.2, BLEU);
        }
        ST[e].tonneau += 1;
        if ST[e].tonneau > TONNEAU {
            ST[e].tonneau = 0;
            tourne_neo(boma, 0.0);
            AttackModule::clear_all(boma);
        }
    }
    // il percute ceux qu'il croise en fonçant (coup renouvelé toutes les 40 images)
    if s.tir == 0 && s.tonneau == 0 {
        if vx.abs() > 1.0 {
            if ST[e].contact <= 0 {
                AttackModule::clear_all(boma);
                coup(fighter, 3, 8.0, 60, 70, 50, 12.0, (12.0, 4.0), None, "collision_attr_normal", *COLLISION_SOUND_ATTR_PUNCH);
                ST[e].contact = 40;
            }
        } else if ST[e].contact > 0 {
            AttackModule::clear_all(boma);
            ST[e].contact = 0;
        }
        ST[e].contact -= 1;
    }
}

/// tir du canon (rayon d'énergie de Neo devant lui)
unsafe fn tir(fighter: &mut L2CFighterCommon, t: i32) {
    let boma = fighter.module_accessor;
    let p = *PostureModule::pos(boma);
    let lr = PostureModule::lr(boma);
    if t == 1 {
        fx_top(boma, "sys_smash_flash_s", (0.0, RAYON_Y, RAYON_DEBUT), 1.5, BLEU);
        PLAY_SE(fighter, Hash40::new(crate::sons::N_CHARGE));
    }
    if t == TIR_DEPART {
        SoundModule::stop_se(boma, Hash40::new(crate::sons::N_CHARGE), 0);
        PLAY_SE(fighter, Hash40::new(crate::sons::N_DECHARGE));
        PLAY_SE(fighter, Hash40::new("se_common_bomb_m"));
        AttackModule::clear_all(boma);
        coup(fighter, 4, 18.0, 40, 75, 70, 6.0, (RAYON_Y, RAYON_DEBUT), Some((RAYON_Y, RAYON_DEBUT + RAYON_LONG)),
             "collision_attr_elec", *COLLISION_SOUND_ATTR_ELEC);
        fx_top(boma, "sys_smash_flash", (0.0, RAYON_Y, RAYON_DEBUT), 2.0, BLEU);
        QUAKE(fighter, *CAMERA_QUAKE_KIND_M);
        // recul du tir
        ST[slot(boma)].vx -= 0.8;
    }
    if t >= TIR_DEPART && t < TIR_DEPART + 10 {
        // le rayon : éclats tout le long, explosion au bout
        let mut d = RAYON_DEBUT;
        while d <= RAYON_DEBUT + RAYON_LONG {
            fx_w(boma, if t % 2 == 0 { "sys_hit_elec" } else { "sys_smash_flash_s" }, p.x + d * lr, p.y + RAYON_Y, 0.9, BLEU);
            d += 12.0;
        }
        if t == TIR_DEPART + 2 {
            fx_w(boma, "sys_bomb_a", p.x + (RAYON_DEBUT + RAYON_LONG) * lr, p.y + RAYON_Y, 1.4, BLEU);
        }
    }
    if t == TIR_DEPART + 10 {
        AttackModule::clear_all(boma);
        ST[slot(boma)].contact = 0;
    }
}

/// FinalEnd de REX : Neo disparaît, Metal revient (effet n° 44 à 8 de haut, son n° 58)
unsafe fn fin(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    ST[e].fin = 1;
    AttackModule::clear_all(boma);
    tourne_neo(boma, 0.0);
    metal_cache(boma, false);
    SoundModule::stop_se(boma, Hash40::new(crate::sons::N_CHARGE), 0);
    let air = StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR && !GroundModule::is_touch(boma, *GROUND_TOUCH_FLAG_DOWN as u32);
    let mot = if air { "final_air_end" } else { "final_end" };
    MotionModule::change_motion(boma, Hash40::new(mot), 0.0, 1.0, false, 0.0, false, false);
    let p = *PostureModule::pos(boma);
    fx_w(boma, "sys_smash_flash", p.x, p.y + 10.0, 2.2, BLANC);
    fx_w(boma, "sys_bomb_b", p.x, p.y + 12.0, 1.2, BLEU);
    PLAY_SE(fighter, Hash40::new("se_sonic_final01"));
}

unsafe extern "C" fn final_exec(_fighter: &mut L2CFighterCommon) -> L2CValue {
    0.into()
}

unsafe extern "C" fn final_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    HitModule::set_whole(boma, HitStatus(*HIT_STATUS_NORMAL), 0);
    KineticModule::enable_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
    AttackModule::clear_all(boma);
    EFFECT_OFF_KIND(fighter, Hash40::new("sys_final_aura"), false, false);
    COL_NORMAL(fighter);
    SoundModule::stop_se(boma, Hash40::new(crate::sons::N_CHARGE), 0);
    tourne_neo(boma, 0.0);
    metal_cache(boma, false);
    ST[e] = ST0;
    smashline::original_status(End, fighter, *FIGHTER_STATUS_KIND_FINAL)(fighter)
}

/// chaque image (ss_frame) : si le Final Smash a été interrompu, Metal revient
pub unsafe fn chaque_image(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    if CACHE[slot(boma)] && StatusModule::status_kind(boma) != *FIGHTER_STATUS_KIND_FINAL {
        metal_cache(boma, false);
        tourne_neo(boma, 0.0);
    }
}

unsafe extern "C" fn vide(_agent: &mut L2CAgentBase) {}

pub fn install(a: &mut Agent) {
    a.status(Main, *FIGHTER_STATUS_KIND_FINAL, final_main);
    a.status(Exec, *FIGHTER_STATUS_KIND_FINAL, final_exec);
    a.status(End, *FIGHTER_STATUS_KIND_FINAL, final_end);
    // scripts de Super Sonic (émeraudes, transformation, caméra) : vidés
    for m in ["finalstart", "finalairstart", "finalend", "finalairend"] {
        for p in ["game_", "effect_", "sound_", "expression_"] {
            let n: &'static str = Box::leak(format!("{}{}", p, m).into_boxed_str());
            a.acmd(n, vide, Priority::Low);
        }
    }
}
