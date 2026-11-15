//! Metal Sonic : voix et bruitages de REX (banque 18E), joués aux images de ses scripts de REX (relevé du 15/11 sur
//! toutes ses sous-actions : rex/metalsonic_sons.md). Les ondes de REX sont dans les banques de Sonic de sa case
//! (rex/tools/sons_metal.py) ; les noms sont ceux des pistes de Sonic, EN MINUSCULES (piège de PROCEDURE.md).
//! Les déplacements (pas, course, sauts, esquives) passent par les scripts de Sonic, qui jouent ces mêmes pistes.

use smash::app::sv_animcmd::frame;
use smash::lua2cpp::*;
use smash::phx::Hash40;
use smash_script::macros::*;
use smash_script::*;
use smashline::*;

// bruitages de REX (n° dans sa banque)
pub const SWING_S: &str = "se_sonic_swing_s";          // 110
pub const SWING_M: &str = "se_sonic_swing_m";          // 108
pub const SWING_L: &str = "se_sonic_swing_l";          // 109
pub const SWING_LL: &str = "se_sonic_swing_ll";        // 107
pub const SIFFLE: &str = "se_sonic_attackair_h01";     // 47
pub const REACTEUR: &str = "se_sonic_attackair_l01";   // 98
pub const SMASH_HAUT: &str = "se_sonic_smash_h01";     // 3
pub const SMASH_BAS: &str = "se_sonic_smash_l01";      // 0
pub const N_CHARGE: &str = "se_sonic_special_n01";     // 82
pub const N_DECHARGE: &str = "se_sonic_special_n02";   // 93
pub const N_PETITE: &str = "se_sonic_special_h02";     // 83
pub const HI_CHARGE: &str = "se_sonic_special_h01";    // 6
pub const S_DEPART: &str = "se_sonic_special_s01";     // 97
pub const BOUCLIER: &str = "se_sonic_special_s03";     // 87
// voix de REX
pub const V_CHARGE: &str = "vc_sonic_001";             // 3
pub const V_SMASH_HAUT: &str = "vc_sonic_002";         // 7
pub const V_COUP: &str = "vc_sonic_attack01";          // 8
pub const V_COUP2: &str = "vc_sonic_attack02";         // 9
pub const V_RUEE: &str = "vc_sonic_attack04";          // 39
pub const V_BIP: &str = "vc_sonic_attack07";           // 46
pub const V_PROVOC_HAUT: &str = "vc_sonic_appeal01";   // 2
pub const V_PROVOC_COTE: &str = "vc_sonic_appeal02";   // 5
pub const V_PROVOC_BAS: &str = "vc_sonic_appeal03";    // 20

/// joue les sons d'une liste (image de l'animation, piste), dans l'ordre des images
unsafe fn joue(agent: &mut L2CAgentBase, l: &[(f32, &str)]) {
    let lua = agent.lua_state_agent;
    for (f, n) in l {
        frame(lua, *f);
        if is_excute(agent) {
            PLAY_SE(agent, Hash40::new(n));
        }
    }
}

macro_rules! sons {
    ($($f:ident => [$(($im:expr, $n:expr)),* $(,)?]),* $(,)?) => {
        $( unsafe extern "C" fn $f(agent: &mut L2CAgentBase) { joue(agent, &[$(($im, $n)),*]); } )*
    };
}

// images de REX ; smashs : + la durée de leur début (AttackS4Start 5, AttackHi4Start 14, AttackLw4Start 10), mis bout
// à bout dans l'animation
sons! {
    s_attack11 => [(3.0, SWING_S)],
    s_attack12 => [(3.0, SWING_M)],
    s_attackdash => [(4.0, SWING_M)],
    s_attacks3 => [(6.0, SIFFLE)],
    s_attackhi3 => [(3.0, SWING_M)],
    s_attacklw3 => [(5.0, SWING_LL)],
    s_attacks4 => [(16.0, V_COUP), (16.0, SWING_L)],
    s_attackhi4 => [(20.0, V_SMASH_HAUT), (20.0, SMASH_HAUT)],
    s_attacklw4 => [(10.0, V_COUP2), (11.0, SMASH_BAS)],
    s_attackairf => [(10.0, SWING_L)],
    s_attackairb => [(10.0, V_COUP), (10.0, REACTEUR)],
    s_attackairhi => [(22.0, V_COUP), (23.0, SIFFLE)],
    s_attackairlw => [(10.0, V_COUP2), (10.0, SIFFLE)],
    s_catch => [(5.0, "se_common_swing_06")],
    s_throwf => [(1.0, "se_common_throw_01"), (18.0, V_COUP)],
    s_throwhi => [(1.0, "se_common_throw_01"), (24.0, REACTEUR)],
    s_throwlw => [(1.0, "se_common_throw_01"), (22.0, "se_common_down_soil_s")],
    s_appealhi => [(6.0, V_BIP), (35.0, V_PROVOC_HAUT)],
    s_appeals => [(30.0, V_PROVOC_COTE)],
    s_appeallw => [(8.0, V_PROVOC_BAS)],
}

pub fn install(a: &mut Agent) {
    a.acmd("sound_attack11", s_attack11, Priority::Low);
    a.acmd("sound_attack12", s_attack12, Priority::Low);
    a.acmd("sound_attackdash", s_attackdash, Priority::Low);
    for n in ["sound_attacks3", "sound_attacks3hi", "sound_attacks3lw"] {
        a.acmd(n, s_attacks3, Priority::Low);
    }
    a.acmd("sound_attackhi3", s_attackhi3, Priority::Low);
    a.acmd("sound_attacklw3", s_attacklw3, Priority::Low);
    for n in ["sound_attacks4", "sound_attacks4hi", "sound_attacks4lw"] {
        a.acmd(n, s_attacks4, Priority::Low);
    }
    a.acmd("sound_attackhi4", s_attackhi4, Priority::Low);
    a.acmd("sound_attacklw4", s_attacklw4, Priority::Low);
    a.acmd("sound_attackairf", s_attackairf, Priority::Low);
    a.acmd("sound_attackairb", s_attackairb, Priority::Low);
    a.acmd("sound_attackairhi", s_attackairhi, Priority::Low);
    a.acmd("sound_attackairlw", s_attackairlw, Priority::Low);
    for n in ["sound_catch", "sound_catchdash", "sound_catchturn"] {
        a.acmd(n, s_catch, Priority::Low);
    }
    a.acmd("sound_throwf", s_throwf, Priority::Low);
    a.acmd("sound_throwhi", s_throwhi, Priority::Low);
    a.acmd("sound_throwlw", s_throwlw, Priority::Low);
    for n in ["sound_appealhil", "sound_appealhir"] {
        a.acmd(n, s_appealhi, Priority::Low);
    }
    for n in ["sound_appealsl", "sound_appealsr"] {
        a.acmd(n, s_appeals, Priority::Low);
    }
    for n in ["sound_appeallwl", "sound_appeallwr"] {
        a.acmd(n, s_appeallw, Priority::Low);
    }
}
