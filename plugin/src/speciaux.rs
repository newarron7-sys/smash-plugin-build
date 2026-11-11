//! Spéciaux de Master Chief (REX, FitChief.pac : actions 0x113 à 0x115 ; vidéos du joueur, noms des coups affichés).
//!
//! Dans REX, ses objets sont des objets de Brawl créés par son module (ft_chief.rel), pas des modèles à lui :
//! - side B « Plasma Grenade » (Garnade toss) : la bombe collante de Brawl (objet « Gooey Bomb »), prise en main à
//!   l'image 3, lancée à l'image 19 (commande 1F010300 : lancer d'objet, vitesse 1,877, angle visé au stick ; flèche
//!   verte de visée) ; ~20 % (vidéo). Ultimate a la même bombe collante (ITEM_KIND_CHEWING) : lueur bleue électrique
//!   ajoutée dessus (vidéo : « boule bleue électrique »).
//! - down B « Fusion Coil » : posée à l'image 9 (drapeau V12000044) : la caisse explosive de Brawl (on la porte, elle
//!   explose quand on la frappe : ~10-19 %) = ITEM_KIND_POWDERBOX d'Ultimate ; une à la fois.
//! - up B « Gravity Lift » : posé (sol) ou lancé en diagonale vers le bas (air) à l'image 13 (V12000043) ; colonne de
//!   lumière qui le propulse très haut à chaque passage, reste un bon moment ; s'il est déjà posé : « NotReady »
//!   (drapeau V22000010 de REX). En l'air il ne monte pas lui-même : il retombe (vidéo).
//! Statuts de Snake remplacés (Nikita, Cypher, C4 jamais créés) ; animations de REX : retarget_exo.py chief_speciaux.

use smash::app::lua_bind::*;
use smash::app::sv_animcmd::frame;
use smash::app::*;
use smash::hash40;
use smash::lib::lua_const::*;
use smash::lib::L2CValue;
use smash::lua2cpp::*;
use smash::phx::{Hash40, Vector2f, Vector3f};
use smash_script::macros::*;
use smash_script::*;
use smashline::*;

const BLEU: (f32, f32, f32) = (0.35, 0.7, 1.0);
const VIOLET: (f32, f32, f32) = (0.75, 0.35, 1.0);

unsafe fn slot(boma: *mut BattleObjectModuleAccessor) -> usize {
    (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7)
}

/// sol / air pendant un spécial : physique et animation suivent (il peut tomber d'un bord ou atterrir)
unsafe fn situation(fighter: &mut L2CFighterCommon, sol_mot: &str, air_mot: &str, init: bool) {
    let boma = fighter.module_accessor;
    let sol = StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND;
    if !init && !StatusModule::is_situation_changed(boma) {
        return;
    }
    if sol {
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_GROUND_STOP);
        GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_GROUND_CLIFF_STOP));
    } else {
        // en l'air : il tombe normalement (contrôle gardé), comme dans REX
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_FALL);
        GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
    }
    let m = Hash40::new(if sol { sol_mot } else { air_mot });
    if init {
        MotionModule::change_motion(boma, m, 0.0, 1.0, false, 0.0, false, false);
    } else {
        MotionModule::change_motion_inherit_frame(boma, m, -1.0, 1.0, 0.0, false, false);
    }
}

unsafe fn finir(fighter: &mut L2CFighterCommon) -> bool {
    let boma = fighter.module_accessor;
    if !MotionModule::is_end(boma) {
        return false;
    }
    if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND {
        fighter.change_status(FIGHTER_STATUS_KIND_WAIT.into(), false.into());
    } else {
        fighter.change_status(FIGHTER_STATUS_KIND_FALL.into(), false.into());
    }
    true
}

unsafe extern "C" fn rien_status(_fighter: &mut L2CFighterCommon) -> L2CValue {
    0.into()
}

// ------------------------------------------------------------------ side B : Plasma Grenade
const G_PRISE: f32 = 3.0;
const G_LANCER: f32 = 19.0;
static mut G_ETAT: [u8; 8] = [0; 8]; // 0 rien, 1 en main, 2 lancée
static mut G_ID: [u32; 8] = [0; 8];
static mut G_T: [i32; 8] = [0; 8];

unsafe extern "C" fn s_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let e = slot(fighter.module_accessor);
    G_ETAT[e] = 0;
    situation(fighter, "special_s_start", "special_air_s_start", true);
    fighter.sub_shift_status_main(L2CValue::Ptr(s_loop as *const () as _))
}

unsafe extern "C" fn s_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    situation(fighter, "special_s_start", "special_air_s_start", false);
    let f = MotionModule::frame(boma);
    // vitesses de l'animation de REX (FSM) : visée ralentie (0,5) tant que B est tenu, puis 1,15 et 1,63 après le lancer
    let rate = if f < 10.0 {
        1.0
    } else if f < G_LANCER {
        if ControlModule::check_button_on(boma, *CONTROL_PAD_BUTTON_SPECIAL) { 0.5 } else { 1.0 }
    } else if f < 22.0 {
        1.15
    } else {
        1.63
    };
    MotionModule::set_rate(boma, rate);
    if G_ETAT[e] == 0 && f >= G_PRISE && !ItemModule::is_have_item(boma, 0) {
        ItemModule::have_item(boma, ItemKind(*ITEM_KIND_CHEWING), 0, 0, false, false);
        G_ETAT[e] = 1;
    }
    if G_ETAT[e] == 1 && f >= G_LANCER {
        // angle visé au stick (REX : V11000030, flèche verte) : 35° par défaut, de 5° (bas) à 75° (haut)
        let sy = ControlModule::get_stick_y(boma);
        let angle = (35.0 + 40.0 * sy).clamp(5.0, 75.0);
        let id = ItemModule::get_have_item_id(boma, 0) as u32;
        ItemModule::throw_item(boma, angle, 1.877, 1.0, 0, true, 0.0);
        if ItemModule::is_have_item(boma, 0) {
            ItemModule::drop_item(boma, angle, 1.877, 0);
        }
        G_ID[e] = id;
        G_T[e] = 0;
        G_ETAT[e] = 2;
    }
    finir(fighter);
    0.into()
}

unsafe extern "C" fn s_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    // interrompu avant le lancer : la bombe ne reste pas dans sa main
    if G_ETAT[e] == 1 && ItemModule::is_have_item(boma, 0) {
        ItemModule::remove_item(boma, 0);
    }
    G_ETAT[e] = 0;
    0.into()
}

/// lueur bleue électrique sur la bombe lancée (tant qu'elle existe)
unsafe fn lueur_grenade(boma: *mut BattleObjectModuleAccessor) {
    let e = slot(boma);
    let id = G_ID[e];
    if id == 0 {
        return;
    }
    if !sv_battle_object::is_active(id) || G_T[e] > 600 {
        G_ID[e] = 0;
        return;
    }
    G_T[e] += 1;
    if G_T[e] % 6 != 1 {
        return;
    }
    let ib = sv_battle_object::module_accessor(id);
    if ib.is_null() {
        G_ID[e] = 0;
        return;
    }
    let z = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    let h = EffectModule::req_follow(ib, Hash40::new("sys_hit_elec"), Hash40::new("top"), &z, &z, 0.3, true, 0, 0, 0, 0, 0, false, false);
    EffectModule::set_rgb(ib, h as u32, BLEU.0, BLEU.1, BLEU.2);
    let h2 = EffectModule::req_follow(ib, Hash40::new("sys_smash_flash_s"), Hash40::new("top"), &z, &z, 0.4, true, 0, 0, 0, 0, 0, false, false);
    EffectModule::set_rgb(ib, h2 as u32, BLEU.0, BLEU.1, BLEU.2);
}

// ------------------------------------------------------------------ down B : Fusion Coil
const C_POSE: f32 = 9.0;
static mut C_FAIT: [bool; 8] = [false; 8];
static mut C_ID: [u32; 8] = [0; 8];

unsafe extern "C" fn lw_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let e = slot(fighter.module_accessor);
    C_FAIT[e] = false;
    situation(fighter, "special_lw_start", "special_air_lw_start", true);
    fighter.sub_shift_status_main(L2CValue::Ptr(lw_loop as *const () as _))
}

unsafe extern "C" fn lw_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    situation(fighter, "special_lw_start", "special_air_lw_start", false);
    if !C_FAIT[e] && MotionModule::frame(boma) >= C_POSE {
        C_FAIT[e] = true;
        // une seule à la fois (REX) ; pas s'il tient déjà un objet
        let existe = C_ID[e] != 0 && sv_battle_object::is_active(C_ID[e]);
        if !existe && !ItemModule::is_have_item(boma, 0) {
            ItemModule::have_item(boma, ItemKind(*ITEM_KIND_POWDERBOX), 0, 0, false, false);
            C_ID[e] = ItemModule::get_have_item_id(boma, 0) as u32;
            ItemModule::drop_item(boma, 0.0, 0.0, 0);
        }
    }
    finir(fighter);
    0.into()
}

// ------------------------------------------------------------------ up B : Gravity Lift
const L_POSE: f32 = 13.0;
const L_DUREE: i32 = 600;           // « reste en place un bon moment » : 10 s
const L_VOL: i32 = 14;              // lancé en l'air : images jusqu'au sol
const L_LARGE: f32 = 7.0;
const L_HAUT: f32 = 45.0;
const L_ELAN: f32 = 4.2;            // propulsé très haut
static mut L_ETAT: [u8; 8] = [0; 8];    // 0 rien, 1 en vol, 2 posé
static mut L_POS: [(f32, f32); 8] = [(0.0, 0.0); 8];
static mut L_DE: [(f32, f32); 8] = [(0.0, 0.0); 8];
static mut L_T: [i32; 8] = [0; 8];
static mut L_RECH: [i32; 8] = [0; 8];
static mut L_ELAN_ATT: [bool; 8] = [false; 8];
static mut HI_FAIT: [bool; 8] = [false; 8];
static mut HI_PAS_PRET: [bool; 8] = [false; 8];

unsafe extern "C" fn hi_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let e = slot(fighter.module_accessor);
    HI_FAIT[e] = false;
    HI_PAS_PRET[e] = L_ETAT[e] != 0;
    if HI_PAS_PRET[e] {
        situation(fighter, "special_air_hi_hang", "special_air_hi_cut", true);
    } else {
        situation(fighter, "special_hi_start", "special_air_hi_start", true);
    }
    fighter.sub_shift_status_main(L2CValue::Ptr(hi_loop as *const () as _))
}

unsafe extern "C" fn hi_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    if HI_PAS_PRET[e] {
        situation(fighter, "special_air_hi_hang", "special_air_hi_cut", false);
    } else {
        situation(fighter, "special_hi_start", "special_air_hi_start", false);
    }
    if !HI_PAS_PRET[e] && !HI_FAIT[e] && MotionModule::frame(boma) >= L_POSE {
        HI_FAIT[e] = true;
        let p = PostureModule::pos(boma);
        let (x, y) = ((*p).x, (*p).y);
        if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND {
            L_POS[e] = (x, y);
            L_ETAT[e] = 2;
            L_T[e] = L_DUREE;
        } else {
            // lancé en diagonale vers le bas : il se pose là où la diagonale touche le sol
            let lr = PostureModule::lr(boma);
            let de = Vector2f { x: x + 3.0 * lr, y: y + 6.0 };
            let dir = Vector2f { x: 60.0 * lr, y: -90.0 };
            let mut hit = Vector2f { x: 0.0, y: 0.0 };
            if GroundModule::ray_check_hit_pos(boma, &de, &dir, &mut hit, true) {
                L_DE[e] = (de.x, de.y);
                L_POS[e] = (hit.x, hit.y);
                L_ETAT[e] = 1;
                L_T[e] = L_VOL;
            }
        }
    }
    finir(fighter);
    0.into()
}

unsafe fn eff_monde(boma: *mut BattleObjectModuleAccessor, eff: &str, x: f32, y: f32, rx: f32, sc: f32, c: (f32, f32, f32)) {
    let p = Vector3f { x, y, z: 0.0 };
    let r = Vector3f { x: rx, y: 0.0, z: 0.0 };
    let h = EffectModule::req(boma, Hash40::new(eff), &p, &r, sc, 0, 0, false, 0) as u32;
    if h != 0 {
        EffectModule::set_rgb(boma, h, c.0, c.1, c.2);
    }
}

/// l'ascenseur : vol (lancé en l'air), colonne de lumière, propulsion de Chief quand il est dedans
unsafe fn ascenseur(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    if L_RECH[e] > 0 {
        L_RECH[e] -= 1;
    }
    match L_ETAT[e] {
        1 => {
            L_T[e] -= 1;
            let t = 1.0 - (L_T[e].max(0) as f32) / (L_VOL as f32);
            let x = L_DE[e].0 + (L_POS[e].0 - L_DE[e].0) * t;
            let y = L_DE[e].1 + (L_POS[e].1 - L_DE[e].1) * t;
            eff_monde(boma, "sys_smash_flash_s", x, y, 0.0, 0.5, VIOLET);
            if L_T[e] <= 0 {
                L_ETAT[e] = 2;
                L_T[e] = L_DUREE;
                eff_monde(boma, "sys_smash_flash", L_POS[e].0, L_POS[e].1 + 1.0, 0.0, 0.8, VIOLET);
            }
        }
        2 => {
            L_T[e] -= 1;
            if L_T[e] <= 0 {
                L_ETAT[e] = 0;
                return;
            }
            let (bx, by) = L_POS[e];
            // l'objet violet au sol et la colonne de lumière blanche / bleue (rafraîchis)
            if L_T[e] % 8 == 0 {
                eff_monde(boma, "sys_smash_flash_s", bx, by + 1.0, 0.0, 0.7, VIOLET);
                for k in 0..4 {
                    eff_monde(boma, "sys_attack_line", bx, by + 6.0 + 11.0 * k as f32, -90.0, 1.1, (0.75, 0.9, 1.0));
                }
            }
            // propulsion : dans la colonne, pas déjà lancé vers le haut
            let p = PostureModule::pos(boma);
            let (x, y) = ((*p).x, (*p).y);
            let dedans = (x - bx).abs() < L_LARGE && y >= by - 1.0 && y < by + L_HAUT;
            if dedans && L_RECH[e] == 0 && !StatusModule::is_situation_changed(boma) {
                let st = StatusModule::status_kind(boma);
                let libre = [*FIGHTER_STATUS_KIND_WAIT, *FIGHTER_STATUS_KIND_WALK, *FIGHTER_STATUS_KIND_DASH,
                             *FIGHTER_STATUS_KIND_RUN, *FIGHTER_STATUS_KIND_TURN, *FIGHTER_STATUS_KIND_TURN_DASH,
                             *FIGHTER_STATUS_KIND_TURN_RUN, *FIGHTER_STATUS_KIND_SQUAT, *FIGHTER_STATUS_KIND_SQUAT_WAIT,
                             *FIGHTER_STATUS_KIND_LANDING, *FIGHTER_STATUS_KIND_LANDING_LIGHT, *FIGHTER_STATUS_KIND_FALL,
                             *FIGHTER_STATUS_KIND_FALL_AERIAL, *FIGHTER_STATUS_KIND_JUMP, *FIGHTER_STATUS_KIND_JUMP_AERIAL,
                             *FIGHTER_STATUS_KIND_JUMP_SQUAT, *FIGHTER_STATUS_KIND_FALL_SPECIAL];
                if libre.contains(&st) {
                    L_RECH[e] = 40;
                    L_ELAN_ATT[e] = true;
                    if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND {
                        // au sol : il décolle (saut), l'élan de la colonne remplace celui du saut
                        StatusModule::change_status_request_from_script(boma, *FIGHTER_STATUS_KIND_JUMP, true);
                    } else if st == *FIGHTER_STATUS_KIND_FALL_SPECIAL {
                        StatusModule::change_status_request_from_script(boma, *FIGHTER_STATUS_KIND_FALL, true);
                    }
                    eff_monde(boma, "sys_smash_flash", bx, by + 3.0, 0.0, 0.9, (0.75, 0.9, 1.0));
                    PLAY_SE(fighter, Hash40::new("se_snake_special_H01"));
                }
            }
        }
        _ => {}
    }
    // élan appliqué une fois en l'air (après le changement de statut éventuel)
    if L_ELAN_ATT[e] && StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR
        && StatusModule::status_kind(boma) != *FIGHTER_STATUS_KIND_JUMP_SQUAT {
        L_ELAN_ATT[e] = false;
        sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY, L_ELAN);
    } else if L_ELAN_ATT[e] && L_RECH[e] < 30 {
        L_ELAN_ATT[e] = false;
    }
}

pub unsafe fn chaque_image(fighter: &mut L2CFighterCommon) {
    lueur_grenade(fighter.module_accessor);
    ascenseur(fighter);
}

// ------------------------------------------------------------------ scripts (effets / sons aux images de REX)
unsafe extern "C" fn rien(_agent: &mut L2CAgentBase) {}

unsafe extern "C" fn son_grenade(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 3.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_snake_special_S01"));
    }
    frame(agent.lua_state_agent, 19.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_snake_special_S02"));
    }
}

unsafe extern "C" fn eff_grenade(agent: &mut L2CAgentBase) {
    // flèche verte de visée (REX : effet 0x1FB0008) : éclat vert devant lui pendant la visée
    frame(agent.lua_state_agent, 4.0);
    if is_excute(agent) {
        EFFECT_FOLLOW(agent, Hash40::new("sys_smash_flash_s"), Hash40::new("haver"), 0, 0, 0, 0, 0, 0, 0.5, true);
        LAST_EFFECT_SET_COLOR(agent, BLEU.0, BLEU.1, BLEU.2);
    }
    frame(agent.lua_state_agent, 10.0);
    if is_excute(agent) {
        EFFECT(agent, Hash40::new("sys_attack_line"), Hash40::new("top"), 0, 14, 6, -35, 0, 0, 0.5, 0, 0, 0, 0, 0, 0, true);
        LAST_EFFECT_SET_COLOR(agent, 0.3, 1.0, 0.3);
    }
}

unsafe extern "C" fn son_coil(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 9.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_snake_special_L01"));
    }
    frame(agent.lua_state_agent, 20.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_snake_special_L02"));
    }
}

unsafe extern "C" fn son_lift(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 1.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_snake_special_H01"));
    }
}

unsafe extern "C" fn eff_lift(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 13.0);
    if is_excute(agent) {
        EFFECT_FOLLOW(agent, Hash40::new("sys_smash_flash"), Hash40::new("haver"), 0, 0, 0, 0, 0, 0, 0.6, true);
        LAST_EFFECT_SET_COLOR(agent, VIOLET.0, VIOLET.1, VIOLET.2);
    }
}

pub fn install(a: &mut Agent) {
    // scripts de Snake (Nikita, Cypher, C4) remplacés pour ces animations
    for m in ["specialsstart", "specialairsstart", "specialhistart", "specialairhistart", "specialairhicut",
              "specialairhihang", "speciallwstart", "specialairlwstart"] {
        for k in ["game_", "effect_", "sound_", "expression_"] {
            let n: &'static str = Box::leak(format!("{}{}", k, m).into_boxed_str());
            let f: unsafe extern "C" fn(&mut L2CAgentBase) = match (k, m) {
                ("sound_", "specialsstart") | ("sound_", "specialairsstart") => son_grenade,
                ("effect_", "specialsstart") | ("effect_", "specialairsstart") => eff_grenade,
                ("sound_", "speciallwstart") | ("sound_", "specialairlwstart") => son_coil,
                ("sound_", "specialhistart") | ("sound_", "specialairhistart") => son_lift,
                ("effect_", "specialhistart") | ("effect_", "specialairhistart") => eff_lift,
                _ => rien,
            };
            a.acmd(n, f, Priority::Low);
        }
    }
    for st in [*FIGHTER_STATUS_KIND_SPECIAL_S, *FIGHTER_STATUS_KIND_SPECIAL_HI, *FIGHTER_STATUS_KIND_SPECIAL_LW] {
        a.status(Init, st, rien_status);
        a.status(Exec, st, rien_status);
    }
    a.status(Main, *FIGHTER_STATUS_KIND_SPECIAL_S, s_main);
    a.status(End, *FIGHTER_STATUS_KIND_SPECIAL_S, s_end);
    a.status(Main, *FIGHTER_STATUS_KIND_SPECIAL_HI, hi_main);
    a.status(End, *FIGHTER_STATUS_KIND_SPECIAL_HI, rien_status);
    a.status(Main, *FIGHTER_STATUS_KIND_SPECIAL_LW, lw_main);
    a.status(End, *FIGHTER_STATUS_KIND_SPECIAL_LW, rien_status);
}
