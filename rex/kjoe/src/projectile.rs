//! Knuckle Joe : projectile du smash avant « Smash Punch ».
//! Demandé (16/11) : un vrai projectile qui part tout seul, comme le f-smash de Primid (son plugin fait sortir le
//! projectile de son perso de base avec ArticleModule::generate_article). Ici : le PK Feu de Ness (lancé par lib.rs
//! à l'image du coup de REX), ses scripts remplacés pour le costume de Knuckle Joe :
//! - en vol : le coup de REX (AttackS4S, coup sur ThrowN : 9 %, 361°, croissance 112, recul 65, feu, taille 4) et
//!   une boule d'énergie bleue (flamme teintée + éclat) au lieu de l'éclair orange du PK Feu ;
//! - quand il touche : pas de colonne de feu à plusieurs coups (pas dans REX), juste une petite explosion bleue.

use smash::app::lua_bind::*;
use smash::app::sv_animcmd::frame;
use smash::app::BattleObjectModuleAccessor;
use smash::hash40;
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
        ATTACK(agent, 0, 0, Hash40::new("top"), dmg, 361, kbg, 0, bkb, taille, 0.0, 0.0, 0.0, None, None, None,
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

// ---------------------------------------------------------------- portée du B neutre
// vidéo du 16/11 : la boule du B neutre n'allait pas assez loin (durée de vie du PK Feu de Ness). Les boules du B
// neutre sont repérées à leur naissance (marquer_neutre) puis vivent plus longtemps et vont plus vite.
const NEUTRE_VIE: i32 = 45;
const NEUTRE_BONUS: f32 = 0.45;
static mut NEUTRES: [u32; 8] = [u32::MAX; 8];
static mut NEUTRES_I: usize = 0;
static mut A_MARQUER: bool = false;

/// appelé par le B neutre juste après generate_article : la prochaine boule de ce joueur est une boule du B neutre
pub unsafe fn marquer_neutre(_boma: *mut BattleObjectModuleAccessor) {
    A_MARQUER = true;
}

unsafe extern "C" fn pk_frame(weapon: &mut L2CFighterBase) {
    let boma = weapon.module_accessor;
    if MotionModule::motion_kind(boma) != hash40("shoot") {
        return;
    }
    let oid = (*weapon.battle_object).battle_object_id;
    let f = MotionModule::frame(boma);
    if f < 1.0 {
        if A_MARQUER && !NEUTRES.contains(&oid) {
            A_MARQUER = false;
            NEUTRES[NEUTRES_I % 8] = oid;
            NEUTRES_I += 1;
            WorkModule::set_int(boma, NEUTRE_VIE, *WEAPON_INSTANCE_WORK_ID_INT_LIFE);
        }
    }
    if NEUTRES.contains(&oid) {
        let lr = PostureModule::lr(boma);
        PostureModule::add_pos(boma, &smash::phx::Vector3f { x: NEUTRE_BONUS * lr, y: 0.0, z: 0.0 });
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
    w.on_line(Main, pk_frame);
    w.install();
}
