#![deny(deprecated)]
#![allow(unused)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]
#![allow(static_mut_refs)]
#![feature(repr_simd)]
#![feature(simd_ffi)]
use smash::app::lua_bind::*;
use smash::lua2cpp::*;
use smash::lib::{*, lua_const::*};
use smash::phx::*;
use smash::app::*;
use smash::app;
use smash::hash40;
use smash::app::sv_animcmd::*;
use utils::{
    *,
    util::*,
    ext::*,
    consts::*,
};
use smashline::*;

#[macro_use] extern crate smash_script;

pub mod djc;
pub mod opff;
pub mod misc;
pub mod general_statuses;
pub mod function_hooks;
pub mod shoto_status;
// pub mod tag;
pub mod acmd;

// for storing what team color the last attacker had. used in a couple different common files
pub static mut LAST_ATTACK_TEAM_COLOR: i32 = 0;

extern "C" fn common_init(fighter: &mut L2CFighterCommon) {
    VarModule::set_int(fighter.battle_object, vars::common::instance::LEDGE_ID, -1);
    VarModule::off_flag(fighter.battle_object, vars::common::instance::IS_INIT);
}

pub fn install() {
    djc::install();
    misc::install();
    // tag::install();
    general_statuses::install();
    function_hooks::install();
    opff::install();
    acmd::install();

    Agent::new("fighter")
        .on_start(common_init)
        .install();
}
// ==========================================================================================
// Installation réduite (extraction 4 persos : lucario, palutena, dolly, ken).
// N'installe PAS le moteur commun HDR (statuts généraux, hooks globaux, get_param) :
// seulement ce dont ces 4 persos ont besoin, limité à eux.
// ==========================================================================================
pub const HDR_LITE_KINDS: [&str; 1] = ["koopa"];
pub const HDR_LITE_SLOT: usize = 55;

pub fn is_hdr_lite_kind(kind: i32) -> bool {
    kind == *FIGHTER_KIND_KOOPA
}

pub fn install_lite() {
    for name in HDR_LITE_KINDS {
        Agent::new(name)
            .set_costume(vec![HDR_LITE_SLOT])
            .on_start(common_init)
            .on_start(misc::fighter_reset)
            .install();
    }
    function_hooks::install_lite();
}
