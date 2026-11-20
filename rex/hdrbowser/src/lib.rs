//! Moveset HDR (HewDraw Remix v0.50.10, Smash 13.0.4) de Bowser sur le seul slot c55 (Dark Bowser),
//! SANS le moteur commun HDR (les autres persos, les autres Bowser et les mécaniques globales restent ceux du jeu).
//! Code HDR sous licence MIT (voir LICENSE-HDR). Même extraction que hdr4 (Lucario, Palutena, Terry, Ken).
#![allow(unused)]

extern crate common;

#[skyline::main(name = "hdr_dark_bowser")]
pub fn main() {
    // Modules HDR (variables, paramètres hdr.xml) ; pas l'affichage des jauges (Bowser n'en a pas)
    utils::init_lite();
    common::install_lite();
    koopa::install();
    // motion_patch.yaml de Bowser HDR : seulement c55 (vendor/runtime-motion-patcher)
    runtime_motion_patcher::install(true);
}
