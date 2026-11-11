//! Super Sonic : pas d'effets de Sonic sur ses déplacements ni sur ses coups (même procédure que Super Shadow) ;
//! seulement la poussière normale du jeu et son aura dorée de Super forme.

use smash::app::sv_animcmd::frame;
use smash::lua2cpp::*;
use smash::phx::Hash40;
use smash::hash40;
use smash_script::macros::*;
use smash_script::*;
use smashline::*;

unsafe extern "C" fn vide(_agent: &mut L2CAgentBase) {}

unsafe extern "C" fn dash(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        FOOT_EFFECT(agent, Hash40::new("sys_dash_smoke"), Hash40::new("top"), -3, 0, 0, 0, 0, 0, 0.63, 0, 0, 0, 0, 0, 0, false);
    }
}

unsafe extern "C" fn course(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    for f in [2.0, 10.0] {
        frame(lua, f);
        if is_excute(agent) {
            FOOT_EFFECT(agent, Hash40::new("sys_run_smoke"), Hash40::new("top"), -2, 0, 0, 0, 0, 0, 0.8, 0, 0, 0, 0, 0, 0, false);
        }
    }
}

unsafe extern "C" fn freinage(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        FOOT_EFFECT(agent, Hash40::new("sys_turn_smoke"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.8, 0, 0, 0, 0, 0, 0, false);
    }
}

unsafe extern "C" fn saut(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        LANDING_EFFECT(agent, Hash40::new("sys_jump_smoke"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, false);
    }
}

pub fn install(a: &mut Agent) {
    a.acmd("effect_dash", dash, Priority::Low);
    a.acmd("effect_turndash", dash, Priority::Low);
    a.acmd("effect_run", course, Priority::Low);
    a.acmd("effect_runbrakel", freinage, Priority::Low);
    a.acmd("effect_runbraker", freinage, Priority::Low);
    a.acmd("effect_turnrun", freinage, Priority::Low);
    a.acmd("effect_turnrunbrake", freinage, Priority::Low);
    a.acmd("effect_jumpfront", saut, Priority::Low);
    a.acmd("effect_jumpback", saut, Priority::Low);
    a.acmd("effect_jumpfrontmini", saut, Priority::Low);
    a.acmd("effect_jumpbackmini", saut, Priority::Low);
    a.acmd("effect_jumpaerialfront", vide, Priority::Low);
    a.acmd("effect_jumpaerialback", vide, Priority::Low);
    install_coups(a);
}

// ------------------------------------------------------------------ aura dorée permanente (étape effets 1, demandé)
// Dans REX, son effet n° 26 (aura de Super forme) tourne en continu (attente, course, sauts…) : ici l'aura du Final
// Smash prêt du jeu, teintée or, accrochée aux hanches, plus de petites étincelles dorées sur le corps.

use smash::app::lua_bind::*;
use smash::app::*;
use smash::lib::lua_const::*;
use smash::phx::Vector3f;

const OR: (f32, f32, f32) = (1.0, 0.78, 0.22);
static mut AURA: [u32; 8] = [0; 8];
static mut T: [u32; 8] = [0; 8];

unsafe fn sans_aura(boma: *mut BattleObjectModuleAccessor) -> bool {
    let st = StatusModule::status_kind(boma);
    st == *FIGHTER_STATUS_KIND_DEAD || st == *FIGHTER_STATUS_KIND_STANDBY || st == *FIGHTER_STATUS_KIND_ENTRY
        || st == *FIGHTER_STATUS_KIND_REBIRTH
}

pub unsafe fn aura(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let id = (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7);
    let h = AURA[id];
    if sans_aura(boma) {
        if h != 0 && EffectModule::is_exist_effect(boma, h) {
            EffectModule::kill(boma, h, false, false);
        }
        AURA[id] = 0;
        return;
    }
    if h == 0 || !EffectModule::is_exist_effect(boma, h) {
        let z = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
        let n = EffectModule::req_follow(boma, Hash40::new("sys_final_aura"), Hash40::new("hip"), &z, &z, 0.9, false,
            0, 0, 0, 0, 0, false, false) as u32;
        if n != 0 {
            EffectModule::set_rgb(boma, n, OR.0, OR.1, OR.2);
        }
        AURA[id] = n;
    }
    // étincelles dorées, une toutes les 8 images, à tour de rôle sur le corps
    T[id] = T[id].wrapping_add(1);
    if T[id] % 8 == 0 {
        let os = ["head", "bust", "hip", "handl", "handr", "footl", "footr"];
        let b = os[((T[id] / 8) as usize) % os.len()];
        let z = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
        let n = EffectModule::req_follow(boma, Hash40::new("sys_smash_flash_s"), Hash40::new(b), &z, &z, 0.6, false,
            0, 0, 0, 0, 0, false, false) as u32;
        if n != 0 {
            EffectModule::set_rgb(boma, n, OR.0, OR.1, OR.2);
            EffectModule::set_alpha(boma, n, 0.7);
        }
    }
}


// ------------------------------------------------------------------ side B (demandé) : l'effet du Boost du f-smash
// pendant toute la course du spin dash (Super Sonic : même effet que le side B de Super Shadow). Aura couchée devant lui, traînée,
// éclat au départ, fumée au sol toutes les 4 images ; tout est retiré quand la course s'arrête.
static mut BOOST: [[u32; 2]; 8] = [[0; 2]; 8];
static mut BOOST_T: [u32; 8] = [0; 8];
static mut BOOST_SIDE: [bool; 8] = [false; 8];

pub unsafe fn boost_side_b(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let id = (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7);
    let st = StatusModule::status_kind(boma);
    // side B : B côté -> charge (SPECIAL_S_HOLD) -> course ; le bas B passe par sa propre charge
    if st == *FIGHTER_STATUS_KIND_SPECIAL_S || st == *FIGHTER_SONIC_STATUS_KIND_SPECIAL_S_HOLD {
        BOOST_SIDE[id] = true;
    } else if st == *FIGHTER_STATUS_KIND_SPECIAL_LW {
        BOOST_SIDE[id] = false;
    }
    let course = crate::speciaux::side_b_course(boma);
    let _ = (st, BOOST_SIDE[id]);
    if !course {
        for h in BOOST[id].iter_mut() {
            if *h != 0 && EffectModule::is_exist_effect(boma, *h) {
                EffectModule::kill(boma, *h, false, false);
            }
            *h = 0;
        }
        BOOST_T[id] = 0;
        return;
    }
    let z = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    if BOOST_T[id] == 0 {
        let p = Vector3f { x: 0.0, y: 7.0, z: 4.0 };
        let n = EffectModule::req_follow(boma, Hash40::new("sys_smash_flash"), Hash40::new("top"), &p, &z, 1.0, false,
            0, 0, 0, 0, 0, false, false) as u32;
        if n != 0 {
            EffectModule::set_rgb(boma, n, OR.0, OR.1, OR.2);
        }
    }
    BOOST_T[id] = BOOST_T[id].wrapping_add(1);
    let h = BOOST[id][0];
    if h == 0 || !EffectModule::is_exist_effect(boma, h) {
        // demandé : plus devant lui (elle touchait sa tête)
        let p = Vector3f { x: 0.0, y: 6.0, z: 11.0 };
        let r = Vector3f { x: -90.0, y: 0.0, z: 0.0 };
        let n = EffectModule::req_follow(boma, Hash40::new("sys_score_aura"), Hash40::new("top"), &p, &r, 0.9, false,
            0, 0, 0, 0, 0, false, false) as u32;
        if n != 0 {
            EffectModule::set_rgb(boma, n, OR.0, OR.1, OR.2);
        }
        BOOST[id][0] = n;
    }
    // demandé : la traînée jaune du f-smash derrière lui (sonic_spintrace, jaune avec le fichier d'effets du mod) et
    // ses traits de vitesse dorés
    let h = BOOST[id][1];
    if h == 0 || !EffectModule::is_exist_effect(boma, h) {
        let p = Vector3f { x: 0.0, y: 7.0, z: 0.0 };
        let n = EffectModule::req_follow(boma, Hash40::new("sonic_spintrace"), Hash40::new("top"), &p, &z, 0.8, false,
            0, 0, 0, 0, 0, false, false) as u32;
        if n != 0 {
            EffectModule::set_alpha(boma, n, 0.7);
        }
        BOOST[id][1] = n;
    }
    if BOOST_T[id] % 6 == 1 {
        let p = Vector3f { x: 0.0, y: 8.0, z: -4.0 };
        let n = EffectModule::req_follow(boma, Hash40::new("sys_attack_speedline"), Hash40::new("top"), &p, &z, 1.2, false,
            0, 0, 0, 0, 0, false, false) as u32;
        if n != 0 {
            EffectModule::set_rgb(boma, n, OR.0, OR.1, OR.2);
        }
    }
    if BOOST_T[id] % 4 == 1 && StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND {
        // demandé : la fumée au sol un peu plus derrière lui
        let p = Vector3f { x: 0.0, y: 0.0, z: -11.0 };
        EffectModule::req_follow(boma, Hash40::new("sys_dash_smoke"), Hash40::new("top"), &p, &z, 0.8, false,
            0, 0, 0, 0, 0, false, false);
    }
}

// ------------------------------------------------------------------ effets de ses coups (demandé, v2)
// Méthode des scripts du jeu (Sonic, Mario, Captain Falcon, HewDraw-Remix) : traits et arcs du jeu (sys_attack_line,
// sys_attack_arc*) teintés, plus une vraie traînée : un petit éclat posé à chaque image sur le poing / le pied qui
// frappe (EFFECT, pas FOLLOW : il reste là où le membre est passé, ce qui dessine sa trajectoire).
// Images = images de ses animations de REX (début des smashs compris).

const ROUGE: (f32, f32, f32) = (1.0, 0.12, 0.05);
type C = (f32, f32, f32);

unsafe fn fx(agent: &mut L2CAgentBase, eff: &str, bone: &str, p: (f32, f32, f32), r: (f32, f32, f32), sc: f32, c: C) {
    if is_excute(agent) {
        EFFECT_FOLLOW(agent, Hash40::new(eff), Hash40::new(bone), p.0, p.1, p.2, r.0, r.1, r.2, sc, true);
        LAST_EFFECT_SET_COLOR(agent, c.0, c.1, c.2);
    }
}
unsafe fn fx_sol(agent: &mut L2CAgentBase, eff: &str, sc: f32) {
    if is_excute(agent) {
        LANDING_EFFECT(agent, Hash40::new(eff), Hash40::new("top"), 0, 0, 0, 0, 0, 0, sc, 0, 0, 0, 0, 0, 0, false);
    }
}
unsafe fn off(agent: &mut L2CAgentBase, eff: &str) {
    if is_excute(agent) {
        EFFECT_OFF_KIND(agent, Hash40::new(eff), false, false);
    }
}
/// traînée : un éclat par image sur chaque os, de f0 à f1 (l'effet n'est pas accroché : il marque le passage du membre)
unsafe fn trainee(agent: &mut L2CAgentBase, os: &[&str], f0: f32, f1: f32, sc: f32, c: C) {
    let lua = agent.lua_state_agent;
    let mut f = f0;
    while f <= f1 {
        frame(lua, f);
        if is_excute(agent) {
            for b in os {
                EFFECT(agent, Hash40::new("sys_smash_flash_s"), Hash40::new(b), 0, 0, 0, 0, 0, 0, sc, 0, 0, 0, 0, 0, 0, true);
                LAST_EFFECT_SET_COLOR(agent, c.0, c.1, c.2);
            }
        }
        f += 1.0;
    }
}

// jabs (demandé : traînée rouge ; REX : trait rouge n° 1C devant lui, hauteur 7,45)
unsafe extern "C" fn e_attack11(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 1.0);
    fx(agent, "sys_attack_line", "top", (0.0, 7.5, 1.0), (0.0, 0.0, 0.0), 0.9, ROUGE);
    trainee(agent, &["handl"], 1.0, 4.0, 0.6, ROUGE);
}
unsafe extern "C" fn e_attack12(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 0.0);
    fx(agent, "sys_attack_line", "top", (0.0, 7.5, 1.0), (0.0, 0.0, 0.0), 0.9, ROUGE);
    trainee(agent, &["handr"], 0.0, 3.0, 0.6, ROUGE);
}
unsafe extern "C" fn e_attack13(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 2.0);
    fx(agent, "sys_attack_arc_b", "footl", (-6.0, -5.0, 0.0), (90.0, 180.0, 60.0), 0.8, ROUGE);
    trainee(agent, &["footl"], 2.0, 7.0, 0.7, ROUGE);
}
// attaque en courant (poing gauche tendu tout du long)
unsafe extern "C" fn e_attackdash(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 1.0);
    fx(agent, "sys_attack_speedline", "top", (0.0, 8.0, -4.0), (0.0, 0.0, 0.0), 1.1, OR);
    fx_sol(agent, "sys_dash_smoke", 0.9);
    trainee(agent, &["handl"], 1.0, 28.0, 0.5, OR);
}
// tilt avant (demandé : électricité ; ses coups sont électriques dans REX) : la main gauche se charge, décharge devant
unsafe extern "C" fn e_attacks3(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 2.0);
    fx(agent, "sys_thunder", "handl", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 0.6, (1.0, 1.0, 1.0));
    frame(lua, 7.0);
    if is_excute(agent) {
        EFFECT(agent, Hash40::new("sys_hit_elec"), Hash40::new("top"), 0, 5, 16, 0, 0, 0, 1.0, 0, 0, 0, 0, 0, 0, true);
        EFFECT_FOLLOW(agent, Hash40::new("sys_damage_elec"), Hash40::new("handl"), 0, 0, 0, 0, 0, 0, 1.4, true);
        EFFECT_FOLLOW(agent, Hash40::new("sys_thunder"), Hash40::new("top"), 0, 5, 12, 0, 0, 90, 1.2, true);
    }
    trainee(agent, &["handl"], 7.0, 12.0, 0.6, (1.0, 0.95, 0.4));
    frame(lua, 31.0);
    off(agent, "sys_damage_elec");
    off(agent, "sys_thunder");
}
unsafe extern "C" fn e_attackhi3(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 4.0);
    fx(agent, "sys_attack_arc_d", "top", (0.0, 11.0, 2.0), (0.0, 10.0, 50.0), 1.0, OR);
    trainee(agent, &["footl"], 4.0, 10.0, 0.6, OR);
}
unsafe extern "C" fn e_attacklw3(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 4.0);
    fx_sol(agent, "sys_atk_smoke", 0.6);
    trainee(agent, &["footr"], 4.0, 10.0, 0.6, OR);
}

// smash avant (demandé : les mêmes effets que le smash avant de Super Shadow) : éclat de charge, puis le Boost doré
// pendant qu'il fonce (images 32 à 46 : REX AttackS4S 24 à 38, + 8 images de AttackS4Start)
unsafe extern "C" fn e_attacks4(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 14.0);
    fx(agent, "sys_smash_flash", "footr", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 1.0, OR);
    frame(lua, 32.0);
    if is_excute(agent) {
        EFFECT_FOLLOW(agent, Hash40::new("sys_smash_flash"), Hash40::new("top"), 0, 7, 4, 0, 0, 0, 1.0, true);
        LAST_EFFECT_SET_COLOR(agent, OR.0, OR.1, OR.2);
        EFFECT_FOLLOW(agent, Hash40::new("sys_score_aura"), Hash40::new("top"), 0, 6, 4, -90, 0, 0, 1.2, true);
        LAST_EFFECT_SET_COLOR(agent, OR.0, OR.1, OR.2);
        EFFECT_FOLLOW(agent, Hash40::new("sonic_spintrace"), Hash40::new("top"), 0, 7, 0, 0, 0, 0, 0.8, true);
        LAST_EFFECT_SET_ALPHA(agent, 0.7);
    }
    let mut f = 32.0;
    while f < 46.0 {
        frame(lua, f);
        if is_excute(agent) {
            if (f - 32.0) % 6.0 == 0.0 {
                EFFECT_FOLLOW(agent, Hash40::new("sys_attack_speedline"), Hash40::new("top"), 0, 8, -4, 0, 0, 0, 1.2, true);
                LAST_EFFECT_SET_COLOR(agent, OR.0, OR.1, OR.2);
            }
            FOOT_EFFECT(agent, Hash40::new("sys_dash_smoke"), Hash40::new("top"), -4, 0, 0, 0, 0, 0, 0.8, 0, 0, 0, 0, 0, 0, false);
        }
        f += 3.0;
    }
    frame(lua, 46.0);
    off(agent, "sys_score_aura");
    off(agent, "sonic_spintrace");
}
// smash haut : la boule monte (os « sphere ») : éclat, tourbillon et traînée dorés comme le smash haut de Sonic
unsafe extern "C" fn e_attackhi4(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 14.0);
    fx(agent, "sys_smash_flash", "top", (0.0, 10.0, 2.0), (0.0, 0.0, 0.0), 1.0, OR);
    frame(lua, 19.0);
    fx_sol(agent, "sys_v_smoke_a", 1.0);
    fx(agent, "sys_spin_wind", "sphere", (0.0, 0.0, 0.0), (0.0, -160.0, 270.0), 0.75, OR);
    fx(agent, "sonic_spintrace", "sphere", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 1.0, OR);
    frame(lua, 22.0);
    fx(agent, "sonic_spinwind", "sphere", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 1.0, OR);
    frame(lua, 32.0);
    fx(agent, "sys_attack_impact", "sphere", (0.0, 3.0, 0.0), (0.0, 0.0, 0.0), 1.6, OR);
    frame(lua, 42.0);
    fx(agent, "sys_attack_impact", "sphere", (0.0, -3.0, 0.0), (0.0, 0.0, 0.0), 1.6, OR);
    frame(lua, 46.0);
    off(agent, "sonic_spinwind");
    off(agent, "sys_spin_wind");
    off(agent, "sonic_spintrace");
    frame(lua, 50.0);
    fx_sol(agent, "sys_down_smoke", 1.0);
}
// smash bas « Sonic Flare » (vidéo REX) : anneau de lumière blanc pendant la charge, puis une lame de feu ROUGE au ras
// du sol à chaque coup de pied (devant à l'image 10, derrière à l'image 24 : REX + 6 images de AttackLw4Start)
const BLANC: (f32, f32, f32) = (1.0, 1.0, 1.0);
unsafe extern "C" fn e_attacklw4(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 2.0);
    fx(agent, "sys_smash_flash", "hip", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 1.3, BLANC);
    for (f, sens) in [(9.0, 0.0), (23.0, 180.0)] {
        frame(lua, f);
        fx_sol(agent, "sys_atk_smoke", 0.8);
        fx(agent, "sys_attack_line", "top", (0.0, 1.5, 2.0), (0.0, sens, 0.0), 1.3, ROUGE);
        fx(agent, "sys_attack_arc_b", "footl", (-6.0, -5.0, 0.0), (90.0, 180.0, 60.0), 1.1, ROUGE);
        if is_excute(agent) {
            EFFECT(agent, Hash40::new("sys_flame"), Hash40::new("footl"), 0, 0, 0, 0, 0, 0, 0.6, 0, 0, 0, 0, 0, 0, true);
            LAST_EFFECT_SET_COLOR(agent, 1.0, 0.25, 0.1);
        }
        trainee(agent, &["footl"], f, f + 8.0, 0.8, ROUGE);
    }
}
// aériens : traînée dorée sur les membres qui frappent pendant leurs coups
unsafe extern "C" fn e_attackairn(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 5.0);
    fx(agent, "sonic_spintrace", "hip", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 0.8, OR);
    trainee(agent, &["footl", "footr"], 5.0, 37.0, 0.5, OR);
    off(agent, "sonic_spintrace");
}
unsafe extern "C" fn e_attackairf(agent: &mut L2CAgentBase) {
    trainee(agent, &["footr"], 6.0, 13.0, 0.7, OR);
}
unsafe extern "C" fn e_attackairb(agent: &mut L2CAgentBase) {
    trainee(agent, &["footl"], 11.0, 21.0, 0.7, OR);
}
unsafe extern "C" fn e_attackairhi(agent: &mut L2CAgentBase) {
    trainee(agent, &["footl", "footr"], 3.0, 14.0, 0.6, OR);
}
unsafe extern "C" fn e_attackairlw(agent: &mut L2CAgentBase) {
    trainee(agent, &["footl", "footr"], 7.0, 24.0, 0.6, OR);
}

fn install_coups(a: &mut Agent) {
    a.acmd("effect_attack11", e_attack11, Priority::Low);
    a.acmd("effect_attack12", e_attack12, Priority::Low);
    a.acmd("effect_attack13", e_attack13, Priority::Low);
    a.acmd("effect_attackdash", e_attackdash, Priority::Low);
    a.acmd("effect_attacks3", e_attacks3, Priority::Low);
    a.acmd("effect_attacks3hi", e_attacks3, Priority::Low);
    a.acmd("effect_attacks3lw", e_attacks3, Priority::Low);
    a.acmd("effect_attackhi3", e_attackhi3, Priority::Low);
    a.acmd("effect_attacklw3", e_attacklw3, Priority::Low);
    a.acmd("effect_attacks4", e_attacks4, Priority::Low);
    a.acmd("effect_attacks4hi", e_attacks4, Priority::Low);
    a.acmd("effect_attacks4lw", e_attacks4, Priority::Low);
    a.acmd("effect_attackhi4", e_attackhi4, Priority::Low);
    a.acmd("effect_attacklw4", e_attacklw4, Priority::Low);
    a.acmd("effect_attackairn", e_attackairn, Priority::Low);
    a.acmd("effect_attackairf", e_attackairf, Priority::Low);
    a.acmd("effect_attackairb", e_attackairb, Priority::Low);
    a.acmd("effect_attackairhi", e_attackairhi, Priority::Low);
    a.acmd("effect_attackairlw", e_attackairlw, Priority::Low);
    a.acmd("expression_attackhi4", x_attackhi4, Priority::Low);
    a.acmd("expression_attacks4", x_attacks4, Priority::Low);
    a.acmd("expression_attacks4hi", x_attacks4, Priority::Low);
    a.acmd("expression_attacks4lw", x_attacks4, Priority::Low);
}

// smash haut : en boule comme dans REX (son corps est remplacé par la boule de l'image 21 à 46 : REX 7 à 32 + 14),
// au lieu des images de Sonic (sa boule apparaissait au sol)
unsafe extern "C" fn x_attackhi4(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 21.0);
    if is_excute(agent) {
        VisibilityModule::set_int64(agent.module_accessor, hash40("body"), hash40("body_sphere"));
        crate::boule(agent.module_accessor, true);
    }
    frame(lua, 46.0);
    if is_excute(agent) {
        VisibilityModule::set_int64(agent.module_accessor, hash40("body"), hash40("body_normal"));
        crate::boule(agent.module_accessor, false);
    }
}

// smash avant « Drop Dash » (REX AttackS4S : changement de modèle en boule à l'image 6, retour à 25 ; + 8 images de
// AttackS4Start ; vidéo REX : boule dorée qui charge sur place puis fonce)
unsafe extern "C" fn x_attacks4(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 14.0);
    if is_excute(agent) {
        VisibilityModule::set_int64(agent.module_accessor, hash40("body"), hash40("body_sphere"));
        crate::boule(agent.module_accessor, true);
    }
    frame(lua, 33.0);
    if is_excute(agent) {
        VisibilityModule::set_int64(agent.module_accessor, hash40("body"), hash40("body_normal"));
        crate::boule(agent.module_accessor, false);
    }
}
