//! Knuckle Joe : effets (étape 1). Les scripts d'effets des coups de Ness (éclats PSI, batte, yo-yo) sont vidés :
//! ses coups de REX sont des coups de poing et de pied, leurs effets viendront ensuite (procédure de Metal Sonic).

use smash::lua2cpp::*;
use smashline::*;

unsafe extern "C" fn vide(_agent: &mut L2CAgentBase) {}

pub fn install(a: &mut Agent) {
    for n in ["effect_attack11", "effect_attack12", "effect_attack13", "effect_attackdash", "effect_attacks3",
              "effect_attacks3hi", "effect_attacks3lw", "effect_attackhi3", "effect_attacklw3", "effect_attacks4",
              "effect_attacks4hi", "effect_attacks4lw", "effect_attacks4charge", "effect_attackhi4",
              "effect_attackhi4charge", "effect_attacklw4", "effect_attacklw4charge", "effect_attackairn",
              "effect_attackairf", "effect_attackairb", "effect_attackairhi", "effect_attackairlw"] {
        a.acmd(n, vide, Priority::Low);
    }
}
