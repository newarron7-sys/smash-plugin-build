//! Knuckle Joe : projectile du smash avant « Smash Punch ».
//! Demandé (16/11) : un vrai projectile qui part tout seul, comme le f-smash de Primid (son plugin fait sortir le
//! projectile de son perso de base avec ArticleModule::generate_article). Ici : le PK Feu de Ness (lancé par lib.rs
//! à l'image du coup de REX), ses scripts remplacés pour le costume de Knuckle Joe :
//! - en vol : le coup de REX (AttackS4S, coup sur ThrowN : 9 %, 361°, croissance 112, recul 65, feu, taille 4) et
//!   une boule d'énergie bleue (flamme teintée + éclat) au lieu de l'éclair orange du PK Feu ;
//! - quand il touche : pas de colonne de feu à plusieurs coups (pas dans REX), juste une petite explosion bleue.

use smash::app::sv_animcmd::frame;
use smash::lib::lua_const::*;
use smash::lua2cpp::*;
use smash::phx::Hash40;
use smash_script::macros::*;
use smash_script::*;
use smashline::*;

const BLEU: (f32, f32, f32) = (0.45, 0.7, 1.0);

/// B neutre (speciaux.rs) : boule plus forte et plus grosse avec la charge (niveaux 0 à 5) ; smash avant : 9 % de REX
unsafe extern "C" fn g_shoot(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        let (dmg, kbg, bkb, taille) = if crate::speciaux::N_TIR {
            let k = crate::speciaux::N_NIVEAU as f32;
            (6.0 + 3.0 * k, 70 + 6 * crate::speciaux::N_NIVEAU, 40 + 6 * crate::speciaux::N_NIVEAU, 3.0 + 0.6 * k)
        } else {
            (9.0, 112, 65, 4.0)
        };
        ATTACK(agent, 0, 0, Hash40::new("top"), dmg, 361, kbg, 0, bkb, taille * crate::ECHELLE, 0.0, 0.0, 0.0, None, None, None,
               0.7, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, true, true, false, false, false,
               *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
               Hash40::new("collision_attr_fire"), *ATTACK_SOUND_LEVEL_M, *COLLISION_SOUND_ATTR_FIRE, *ATTACK_REGION_BOMB);
    }
}

unsafe extern "C" fn e_shoot(agent: &mut L2CAgentBase) {
    let lua = agent.lua_state_agent;
    let mut f = 1.0;
    while f < 60.0 {
        frame(lua, f);
        if is_excute(agent) {
            let sc = if crate::speciaux::N_TIR { 0.6 + 0.15 * crate::speciaux::N_NIVEAU as f32 } else { 0.9 };
            EFFECT_FOLLOW(agent, Hash40::new("sys_flame"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, sc, true);
            LAST_EFFECT_SET_COLOR(agent, BLEU.0, BLEU.1, BLEU.2);
            EFFECT_FOLLOW(agent, Hash40::new("sys_smash_flash_s"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.8, true);
            LAST_EFFECT_SET_COLOR(agent, 0.8, 0.9, 1.0);
        }
        f += 2.0;
    }
}

unsafe extern "C" fn e_pillar(agent: &mut L2CAgentBase) {
    if is_excute(agent) {
        EFFECT(agent, Hash40::new("sys_bomb_b"), Hash40::new("top"), 0, 3, 0, 0, 0, 0, 0.6, 0, 0, 0, 0, 0, 0, true);
        LAST_EFFECT_SET_COLOR(agent, BLEU.0, BLEU.1, BLEU.2);
    }
}

unsafe extern "C" fn vide(_agent: &mut L2CAgentBase) {}

pub fn install() {
    let mut w = Agent::new("ness_pkfire");
    w.set_costume(crate::SS_COSTUMES.to_vec());
    w.acmd("game_shoot", g_shoot, Priority::Low);
    w.acmd("effect_shoot", e_shoot, Priority::Low);
    w.acmd("game_pillar", vide, Priority::Low);
    w.acmd("effect_pillar", e_pillar, Priority::Low);
    w.install();
}
