//! Cloud Mario : le moveset de Mario d'Ultimate S (code de chrispo-git/ult-s, dossier src/mario, licence AGPL-3.0,
//! voir LICENSE.md) sur le seul slot c120 (Cloud Mario, écho de Mario), sans le plugin Ultimate S.
//! Ce qu'Ultimate S fournissait autour (util.rs) est refait ici au plus simple :
//! - get_marked_costumes : toujours [120] (Ultimate S lit des fichiers .marker) ;
//! - VariableModule : variables par combattant ;
//! - transition_set / transition_reset (side B une fois par saut) : interdit le side B en l'air chaque image ;
//! - param_cache (jump_speed_x_mul 1,2) : config_param.toml lu par libparam_config (aucun hook de paramètres ici).
#![allow(unused, non_snake_case, non_upper_case_globals, static_mut_refs, unused_unsafe, unused_must_use)]
#![feature(proc_macro_hygiene)]

#[macro_use]
mod util;
mod mario;

#[skyline::main(name = "cloud_mario_ults")]
pub fn main() {
    util::install();
    mario::install();
}
