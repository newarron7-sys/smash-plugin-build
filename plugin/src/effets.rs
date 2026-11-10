//! Étape 4 partie 1 (demandé) : plus d'effets de Sonic sur ses déplacements (traînées bleues/rouges du dash et de la
//! course, tourbillon des sauts). Il ne reste que la poussière normale du jeu, comme pour tous les persos.

use smash::app::sv_animcmd::frame;
use smash::lua2cpp::*;
use smash::phx::Hash40;
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

// ------------------------------------------------------------------ effets de ses coups (étape effets 2, demandé)
// Mêmes images et mêmes os que le script d'effets (GFX) de REX ; ses effets de Brawl (traces, cercles, flammes des
// chaussures) sont refaits avec des effets du jeu teintés or. Remplacent ceux de Sonic.

unsafe fn fx(agent: &mut L2CAgentBase, eff: &str, bone: &str, x: f32, y: f32, z: f32, rx: f32, ry: f32, rz: f32, sc: f32) {
    if is_excute(agent) {
        EFFECT_FOLLOW(agent, Hash40::new(eff), Hash40::new(bone), x, y, z, rx, ry, rz, sc, true);
        LAST_EFFECT_SET_COLOR(agent, OR.0, OR.1, OR.2);
    }
}

unsafe fn fin(agent: &mut L2CAgentBase, eff: &str) {
    if is_excute(agent) {
        EFFECT_OFF_KIND(agent, Hash40::new(eff), false, false);
    }
}

unsafe fn poussiere(agent: &mut L2CAgentBase, eff: &str, sc: f32) {
    if is_excute(agent) {
        LANDING_EFFECT(agent, Hash40::new(eff), Hash40::new("top"), 0, 0, 0, 0, 0, 0, sc, 0, 0, 0, 0, 0, 0, false);
    }
}

// jabs 1 et 2 (REX : trace du coup n° 14 devant lui, images 1,5 / 2,5)
unsafe extern "C" fn e_attack11(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 2.0);
    fx(agent, "sys_attack_speedline", "top", 0.0, 8.0, 3.0, 0.0, 0.0, 0.0, 0.6);
}
unsafe extern "C" fn e_attack12(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 3.0);
    fx(agent, "sys_attack_speedline", "top", 0.0, 8.0, 3.0, 0.0, 0.0, 0.0, 0.6);
}
// jab 3 (demandé) : sys_explosion_sign devant lui (sa « Shadow Ball » de REX), image 11
unsafe extern "C" fn e_attack13(agent: &mut L2CAgentBase) {
    // demandé : rafale de 4 petits coups puis le coup final ; l'effet est celui du nair du Shadow d'Ultimate (explosion
    // rouge + flamme), à l'endroit de sa « Shadow Ball » devant lui
    let p = (0.0, 8.35, 16.5);
    for f in [10.0, 13.0, 16.0, 19.0] {
        salve_courte(agent, f, "top", p, 0.6);
    }
    salve_courte(agent, 22.0, "top", p, 1.1);
    frame(agent.lua_state_agent, 27.0);
    if is_excute(agent) {
        EFFECT_OFF_KIND(agent, Hash40::new("sys_smartbomb_finish"), true, true);
        EFFECT_OFF_KIND(agent, Hash40::new("sys_flame"), true, true);
    }
}

// attaque en courant (REX : traînée n° 12 devant lui de l'image 0 à 18)
unsafe extern "C" fn e_attackdash(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 1.0);
    poussiere(agent, "sys_dash_smoke", 0.7);
    for f in [1.0, 6.0, 11.0] {
        frame(lua, f);
        fx(agent, "sys_attack_speedline", "top", 0.0, 10.0, 6.0, 0.0, 0.0, 0.0, 0.8);
    }
    frame(lua, 18.0);
    fin(agent, "sys_attack_speedline");
}
// f tilt « Shadow Swipe » (REX : 3 cercles n° 15 sur la main droite, images 13 à 25)
unsafe extern "C" fn e_attacks3(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 13.0);
    fx(agent, "sys_attack_impact", "handr", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.3);
    fx(agent, "sys_smash_flash_s", "handr", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.2);
    frame(lua, 25.0);
    fin(agent, "sys_attack_impact");
}
// up tilt : rien dans REX (son aura seulement)
// down tilt (REX : flammes n° 39 / 37 aux deux pieds, image 3)
#[allow(dead_code)]
unsafe extern "C" fn e_attacklw3(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 3.0);
    fx(agent, "sys_attack_impact", "footl", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5);
    fx(agent, "sys_attack_impact", "footr", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5);
    poussiere(agent, "sys_atk_smoke", 0.6);
}
// smash avant (REX : éclat de charge sur la jambe droite à l'image 6, puis il fonce)
unsafe extern "C" fn e_attacks4(agent: &mut L2CAgentBase) {
    // demandé : l'effet du down B aérien du Shadow d'Ultimate (corps rouge brûlant, traînée rouge), mais droit comme un
    // dash, à partir de l'image 17, avec de la fumée blanche au sol qui le suit
    let lua = agent.lua_state_agent;
    frame(lua, 6.0);
    fx(agent, "sys_smash_flash", "legr", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    frame(lua, 17.0);
    if is_excute(agent) {
        BURN_COLOR(agent, 2.0, 0.0, 0.1, 0.9);
    }
    let mut f = 17.0;
    while f < 39.0 {
        frame(lua, f);
        if is_excute(agent) {
            EFFECT_FOLLOW(agent, Hash40::new("sys_attack_speedline"), Hash40::new("top"), 0, 8, -4, 0, 0, 0, 1.2, true);
            LAST_EFFECT_SET_COLOR(agent, 2.0, 0.0, 0.1);
            EFFECT_FOLLOW(agent, Hash40::new("sys_attack_line_b"), Hash40::new("top"), 0, 8, 2, 0, 0, 0, 1.0, true);
            LAST_EFFECT_SET_COLOR(agent, 2.0, 0.0, 0.1);
            LAST_EFFECT_SET_ALPHA(agent, 0.6);
            FOOT_EFFECT(agent, Hash40::new("sys_dash_smoke"), Hash40::new("top"), -4, 0, 0, 0, 0, 0, 0.8, 0, 0, 0, 0, 0, 0, false);
        }
        f += 3.0;
    }
    frame(lua, 39.0);
    if is_excute(agent) {
        BURN_COLOR_NORMAL(agent);
        EFFECT_OFF_KIND(agent, Hash40::new("sys_attack_line_b"), false, false);
    }
}

// charges des smashs (REX : éclats répétés)
unsafe extern "C" fn e_charge_jambe(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    for f in [5.0, 10.0, 15.0, 20.0, 25.0, 30.0, 35.0, 40.0, 45.0, 50.0, 55.0] {
        frame(lua, f);
        fx(agent, "sys_smash_flash_s", "legr", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.9);
    }
}
unsafe extern "C" fn e_charge_corps(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    for f in [5.0, 10.0, 15.0, 20.0, 25.0, 30.0, 35.0, 40.0, 45.0, 50.0, 55.0] {
        frame(lua, f);
        fx(agent, "sys_smash_flash_s", "top", 0.0, 10.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    }
}
// ------------------------------------------------------------------ effets repris du Shadow d'Ultimate (mod de
// mRamas, lus dans son plugin) : explosion de la Smart Bomb (sys_smartbomb_finish) teintée rouge-magenta, doublée
// d'une teinte cyan très transparente, et flamme rouge (sys_flame) ; mêmes couleurs, tailles et vitesses que lui
unsafe fn sb(agent: &mut L2CAgentBase, eff: &str, bone: &str, y: f32, sc: f32, c: (f32, f32, f32), alpha: f32, rate: f32) {
    if is_excute(agent) {
        EFFECT_FOLLOW(agent, Hash40::new(eff), Hash40::new(bone), 0, y, 0, 0, 0, 0, sc, true);
        LAST_EFFECT_SET_COLOR(agent, c.0, c.1, c.2);
        LAST_EFFECT_SET_ALPHA(agent, alpha);
        if rate > 0.0 {
            LAST_EFFECT_SET_RATE(agent, rate);
        }
    }
}
unsafe fn sbz(agent: &mut L2CAgentBase, eff: &str, bone: &str, p: (f32, f32, f32), sc: f32, c: (f32, f32, f32), alpha: f32, rate: f32) {
    if is_excute(agent) {
        EFFECT_FOLLOW(agent, Hash40::new(eff), Hash40::new(bone), p.0, p.1, p.2, 0, 0, 0, sc, true);
        LAST_EFFECT_SET_COLOR(agent, c.0, c.1, c.2);
        LAST_EFFECT_SET_ALPHA(agent, alpha);
        if rate > 0.0 {
            LAST_EFFECT_SET_RATE(agent, rate);
        }
    }
}
const ROUGE: (f32, f32, f32) = (3.5, 0.0, 0.2);
const ROUGE2: (f32, f32, f32) = (4.5, 0.0, 0.2);
const CYAN: (f32, f32, f32) = (0.0, 1.0, 1.0);

/// salve courte (2 images) à une position quelconque : pour les coups en rafale
unsafe fn salve_courte(agent: &mut L2CAgentBase, f0: f32, bone: &str, p: (f32, f32, f32), k: f32) {
    let lua = agent.lua_state_agent;
    frame(lua, f0);
    sbz(agent, "sys_smartbomb_finish", bone, p, 0.25 * k, ROUGE, 0.4, 0.3);
    sbz(agent, "sys_smartbomb_finish", bone, p, 0.25 * k, CYAN, 0.3, 0.5);
    sbz(agent, "sys_flame", bone, p, 0.9 * k, ROUGE, 0.3, 0.0);
    frame(lua, f0 + 1.0);
    sbz(agent, "sys_smartbomb_finish", bone, p, 0.2 * k, ROUGE2, 0.4, 0.6);
}

/// onde de choc au sol (demandé)
unsafe fn onde(agent: &mut L2CAgentBase, sc: f32) {
    if is_excute(agent) {
        LANDING_EFFECT(agent, Hash40::new("sys_ground_shockwave"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, sc, 0, 0, 0, 0, 0, 0, false);
    }
}

/// une salve d'explosions + flamme comme celles du Shadow d'Ultimate, à partir de l'image f0, sur l'os et la hauteur
/// donnés, éteinte 4 images plus tard
unsafe fn salve(agent: &mut L2CAgentBase, f0: f32, bone: &str, y: f32, k: f32) {
    let lua = agent.lua_state_agent;
    frame(lua, f0);
    sb(agent, "sys_smartbomb_finish", bone, y, 0.25 * k, ROUGE, 0.4, 0.3);
    sb(agent, "sys_smartbomb_finish", bone, y, 0.25 * k, CYAN, 0.3, 0.5);
    sb(agent, "sys_flame", bone, y, 0.9 * k, ROUGE, 0.3, 0.0);
    frame(lua, f0 + 1.0);
    sb(agent, "sys_smartbomb_finish", bone, y, 0.1 * k, CYAN, 0.3, 0.8);
    frame(lua, f0 + 2.0);
    sb(agent, "sys_smartbomb_finish", bone, y, 0.2 * k, ROUGE2, 0.4, 0.6);
    frame(lua, f0 + 3.0);
    sb(agent, "sys_smartbomb_finish", bone, y, 0.1 * k, CYAN, 0.4, 1.2);
    frame(lua, f0 + 4.0);
    if is_excute(agent) {
        EFFECT_OFF_KIND(agent, Hash40::new("sys_smartbomb_finish"), true, true);
    }
}

// smash haut (demandé : celui du Shadow d'Ultimate) : fumée au sol, puis au coup explosions rouges sur les hanches,
// flamme au-dessus de lui et tourbillons ; ses coups de REX touchent aux images 33 et 52
unsafe extern "C" fn e_attackhi4(agent: &mut L2CAgentBase) {
    // demandé : coups en rafale de l'image 33 jusqu'au coup final (52), avec les effets du Shadow d'Ultimate à chaque
    // coup, et une onde de choc au sol
    let lua = agent.lua_state_agent;
    frame(lua, 22.0);
    poussiere(agent, "sys_landing_smoke", 1.0);
    if is_excute(agent) {
        LAST_EFFECT_SET_RATE(agent, 0.8);
        LANDING_EFFECT(agent, Hash40::new("sys_jump_smoke"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, false);
    }
    frame(lua, 33.0);
    onde(agent, 1.0);
    sb(agent, "sys_smartbomb_finish", "hip", 0.0, 0.36, (5.5, 0.0, 0.2), 0.2, 0.1);
    sb(agent, "sys_smartbomb_finish", "hip", 0.0, 0.26, CYAN, 0.1, 0.3);
    sb(agent, "sys_flame", "top", 8.0, 1.0, (5.5, 0.0, 0.2), 0.3, 0.0);
    if is_excute(agent) {
        EFFECT_FLIP(agent, Hash40::new("sys_whirlwind_l"), Hash40::new("sys_whirlwind_r"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, true, *EF_FLIP_YZ);
    }
    // demandé : moins d'effets (le jeu ralentissait) : une salve toutes les 8 images au lieu de 4, et les
    // explosions précédentes éteintes avant chaque salve (elles s'accumulaient)
    for f in [39.0, 47.0] {
        frame(lua, f - 1.0);
        if is_excute(agent) {
            EFFECT_OFF_KIND(agent, Hash40::new("sys_smartbomb_finish"), true, true);
            EFFECT_OFF_KIND(agent, Hash40::new("sys_flame"), true, true);
        }
        salve_courte(agent, f, "hip", (0.0, 0.0, 0.0), 1.0);
    }
    frame(lua, 51.0);
    if is_excute(agent) {
        EFFECT_OFF_KIND(agent, Hash40::new("sys_smartbomb_finish"), true, true);
        EFFECT_OFF_KIND(agent, Hash40::new("sys_flame"), true, true);
    }
    frame(lua, 52.0);
    onde(agent, 1.3);
    salve(agent, 52.0, "hip", 0.0, 1.3);
    frame(lua, 64.0);
    poussiere(agent, "sys_landing_smoke", 1.0);
}

// smash bas (REX : éclat à l'image 2, puis traces n° 10 aux deux mains, images 12 à 21)
unsafe extern "C" fn e_attacklw4(agent: &mut L2CAgentBase) {
    // demandé : la bulle rose-rouge du down B du Shadow d'Ultimate (sys_smartbomb_finish) sur ses DEUX mains, à ses
    // coups de REX (images 12 à 22, 2 coups : 12 et 16)
    let lua = agent.lua_state_agent;
    frame(lua, 2.0);
    fx(agent, "sys_smash_flash", "top", 0.0, 5.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    frame(lua, 12.0);
    for m in ["handl", "handr"] {
        sb(agent, "sys_smartbomb_finish", m, 0.0, 0.3, (5.5, 0.0, 0.2), 0.2, 0.1);
        sb(agent, "sys_smartbomb_finish", m, 0.0, 0.22, CYAN, 0.1, 0.3);
    }
    frame(lua, 14.0);
    for m in ["handl", "handr"] {
        sb(agent, "sys_smartbomb_finish", m, 0.0, 0.14, CYAN, 0.1, 0.6);
    }
    frame(lua, 16.0);
    for m in ["handl", "handr"] {
        sb(agent, "sys_smartbomb_finish", m, 0.0, 0.22, (6.5, 0.0, 0.2), 0.2, 0.4);
    }
    frame(lua, 19.0);
    for m in ["handl", "handr"] {
        sb(agent, "sys_smartbomb_finish", m, 0.0, 0.14, CYAN, 0.2, 1.0);
    }
    // demandé : onde de choc au sol quand il frappe (sa hitbox vient maintenant à l'image 20)
    frame(lua, 20.0);
    onde(agent, 1.1);
    poussiere(agent, "sys_atk_smoke", 0.8);
    frame(lua, 30.0);
    if is_excute(agent) {
        EFFECT_OFF_KIND(agent, Hash40::new("sys_smartbomb_finish"), true, true);
    }
}
// aériens
// nair (demandé : feu et fumée du Shadow d'Ultimate) : une salve à chacun de ses 2 coups de REX (images 5 et 14)
unsafe extern "C" fn e_attackairn(agent: &mut L2CAgentBase) {
    // demandé : le feu et la fumée du back air du Shadow d'Ultimate (fumée de la ceinture-fusée, flamme) qui suivent
    // ses DEUX pieds pendant le coup
    let lua = agent.lua_state_agent;
    frame(lua, 5.0);
    if is_excute(agent) {
        for pied in ["toel", "toer"] {
            EFFECT_FOLLOW(agent, Hash40::new("sys_rocketbelt_smoke_loop"), Hash40::new(pied), 0, 0, 0, 0, 0, 0, 0.8, true);
            EFFECT_FOLLOW(agent, Hash40::new("sys_flame"), Hash40::new(pied), 0, 1.5, 0, 0, 0, 0, 0.5, true);
            LAST_EFFECT_SET_COLOR(agent, 3.5, 0.6, 0.1);
        }
    }
    // demandé : plus d'explosion, seulement la fumée et le feu sur ses pieds
    frame(lua, 40.0);
    if is_excute(agent) {
        EFFECT_OFF_KIND(agent, Hash40::new("sys_rocketbelt_smoke_loop"), false, false);
        EFFECT_OFF_KIND(agent, Hash40::new("sys_flame"), false, false);
    }
}

// fair (demandé : celui du Shadow d'Ultimate) : salve sur le poing gauche à son coup de REX (image 19)
unsafe extern "C" fn e_attackairf(agent: &mut L2CAgentBase) {
    salve(agent, 19.0, "handl", -1.5, 1.0);
}
unsafe extern "C" fn e_attackairb(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 5.0);
    fx(agent, "sys_attack_impact", "footr", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.6);
    fx(agent, "sys_attack_arc_b", "footr", 0.0, 0.0, 0.0, 180.0, 90.0, 0.0, 0.6);
}
#[allow(dead_code)]
unsafe extern "C" fn e_attackairhi(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 11.0);
    fx(agent, "sys_attack_arc", "footr", 0.0, 0.0, 0.0, 180.0, 0.0, 0.0, 1.0);
    fx(agent, "sys_attack_impact", "footr", 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.4);
    frame(lua, 13.0);
    fx(agent, "sys_attack_speedline", "top", 0.0, 12.0, 0.0, -90.0, 0.0, 0.0, 0.6);
}
unsafe extern "C" fn e_attackairlw(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 2.0);
    fx(agent, "sys_smash_flash_s", "footl", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.75);
    frame(lua, 17.0);
    fx(agent, "sys_attack_impact", "top", 0.0, 5.0, 1.0, -15.0, 0.0, 0.0, 1.0);
}
unsafe extern "C" fn e_vide(_agent: &mut L2CAgentBase) {}

pub fn install_coups(a: &mut Agent) {
    a.acmd("effect_attack11", e_attack11, Priority::Low);
    a.acmd("effect_attack12", e_attack12, Priority::Low);
    a.acmd("effect_attack13", e_attack13, Priority::Low);
    a.acmd("effect_attackdash", e_attackdash, Priority::Low);
    a.acmd("effect_attacks3", e_attacks3, Priority::Low);
    a.acmd("effect_attacks3hi", e_attacks3, Priority::Low);
    a.acmd("effect_attacks3lw", e_attacks3, Priority::Low);
    a.acmd("effect_attacks4", e_attacks4, Priority::Low);
    a.acmd("effect_attacks4hi", e_attacks4, Priority::Low);
    a.acmd("effect_attacks4lw", e_attacks4, Priority::Low);
    a.acmd("effect_attacks4charge", e_charge_jambe, Priority::Low);
    a.acmd("effect_attackhi4", e_attackhi4, Priority::Low);
    a.acmd("effect_attackhi4charge", e_charge_corps, Priority::Low);
    a.acmd("effect_attacklw4", e_attacklw4, Priority::Low);
    a.acmd("effect_attacklw4charge", e_charge_corps, Priority::Low);
    a.acmd("effect_attackairn", e_attackairn, Priority::Low);
    a.acmd("effect_attackairf", e_attackairf, Priority::Low);
    a.acmd("effect_attackairb", e_attackairb, Priority::Low);
    a.acmd("effect_attackairlw", e_attackairlw, Priority::Low);
}
