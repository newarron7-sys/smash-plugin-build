//! Knuckle Joe : effets (étape 1). Les scripts d'effets des coups de Ness (éclats PSI, batte, yo-yo) sont vidés :
//! ses coups de REX sont des coups de poing et de pied, leurs effets viendront ensuite (procédure de Metal Sonic).

use smash::app::sv_animcmd::frame;
use smash::lua2cpp::*;
use smash::phx::Hash40;
use smash_script::macros::*;
use smash_script::*;
use smashline::*;

const BLEU: (f32, f32, f32) = (0.45, 0.7, 1.0);

/// smash avant « Smash Punch » (vidéo du showcase, 16/11 : « son f-smash envoie une boule de feu bleue ») : dans REX
/// (AttackS4S, après les 7 images de AttackS4Start) une boule d'énergie (effet n° 0x1E de son fichier, sur ThrowN)
/// part du poing à l'image 3 et file devant lui jusqu'à l'image 14 (ThrowN : 14 -> 36 devant, à 7 de haut), avec
/// un coup de 9 % (feu) sur elle de 6 à 14 ; ici images + 7. L'os « throw » suit ThrowN (retarget_exo) : la boule
/// et son coup sont posés dessus (lib.rs)
unsafe extern "C" fn e_attacks4(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    frame(lua, 10.0);
    if is_excute(agent) {
        LANDING_EFFECT(agent, Hash40::new("sys_atk_smoke"), Hash40::new("top"), 0, 0, -2, 0, 0, 0, 0.8, 0, 0, 0, 0, 0, 0, false);
        EFFECT(agent, Hash40::new("sys_smash_flash"), Hash40::new("throw"), 0, 0, 0, 0, 0, 0, 1.0, 0, 0, 0, 0, 0, 0, true);
        LAST_EFFECT_SET_COLOR(agent, BLEU.0, BLEU.1, BLEU.2);
    }
    let mut f = 10.0;
    while f <= 21.0 {
        frame(lua, f);
        if is_excute(agent) {
            EFFECT_FOLLOW(agent, Hash40::new("sys_flame"), Hash40::new("throw"), 0, 0, 0, 0, 0, 0, 0.9, true);
            LAST_EFFECT_SET_COLOR(agent, BLEU.0, BLEU.1, BLEU.2);
            EFFECT_FOLLOW(agent, Hash40::new("sys_smash_flash_s"), Hash40::new("throw"), 0, 0, 0, 0, 0, 0, 0.9, true);
            LAST_EFFECT_SET_COLOR(agent, 0.8, 0.9, 1.0);
        }
        f += 2.0;
    }
    frame(lua, 21.0);
    if is_excute(agent) {
        EFFECT(agent, Hash40::new("sys_bomb_b"), Hash40::new("throw"), 0, 0, 0, 0, 0, 0, 0.5, 0, 0, 0, 0, 0, 0, true);
        LAST_EFFECT_SET_COLOR(agent, BLEU.0, BLEU.1, BLEU.2);
    }
}

unsafe extern "C" fn vide(_agent: &mut L2CAgentBase) {}

unsafe extern "C" fn saut_aerien(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        EFFECT(agent, Hash40::new("sys_jump_aerial"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.8, 0, 0, 0, 0, 0, 0, false);
    }
}

pub fn install(a: &mut Agent) {
    for n in ["effect_attack11", "effect_attack12", "effect_attack13", "effect_attackdash", "effect_attacks3",
              "effect_attacks3hi", "effect_attacks3lw", "effect_attackhi3", "effect_attacklw3", "effect_attacks4charge", "effect_attackhi4",
              "effect_attackhi4charge", "effect_attacklw4", "effect_attacklw4charge", "effect_attackairn",
              "effect_attackairf", "effect_attackairb", "effect_attackairhi", "effect_attackairlw"] {
        a.acmd(n, vide, Priority::Low);
    }
    // vidéo du 16/11 (« son deuxième saut bug ») : le double saut de Ness l'entoure d'une bulle PSI bleue : retirée
    // (Knuckle Joe n'a pas de PSI), un petit nuage à la place, comme le double saut des autres persos
    for n in ["effect_jumpaerialfront", "effect_jumpaerialback"] {
        a.acmd(n, saut_aerien, Priority::Low);
    }
    for n in ["effect_attacks4", "effect_attacks4hi", "effect_attacks4lw"] {
        a.acmd(n, e_attacks4, Priority::Low);
    }
}
