#![deny(deprecated)]
#![allow(unused)]
#![allow(non_snake_case)]

pub mod acmd;

pub mod opff;
pub mod status;

// articles

mod breath;

use smash::{
    lib::{
        L2CValue,
        LuaConst,
    },
    app::{
        *,
        self,
        sv_animcmd::{
            frame,
            wait
        },
        lua_bind::*
    },
    hash40,
    lib::lua_const::*,
    lua2cpp::*,
    phx::*
};
use smash_script::{
    *,
    macros::*
};
use smash_script::macros::ATTACK_ABS;
use utils::{
    *,
    util::*,
    ext::*,
    consts::*,
};
use smashline::*;
#[macro_use] extern crate smash_script;

pub const MAX_COOLDOWN : i32 = 900;

// Dark Bowser : animations à la vitesse normale. HDR accélère beaucoup de coups (FT_MOTION_RATE < 1) :
// ces accélérations sont annulées, les ralentissements d'HDR sont gardés.
pub unsafe fn ralenti(agent: &mut L2CAgentBase, rate: f32) {
    smash_script::macros::FT_MOTION_RATE(agent, rate.max(1.0));
}
pub unsafe fn ralenti_range(agent: &mut L2CAgentBase, debut: f32, fin: f32, frames: f32) {
    if frames >= fin - debut {
        smash_script::macros::FT_MOTION_RATE_RANGE(agent, debut, fin, frames);
    } else {
        smash_script::macros::FT_MOTION_RATE(agent, 1.0);
    }
}


pub fn install() {
    let agent = &mut Agent::new("koopa");
    agent.set_costume(vec![55]);
    acmd::install(agent);
    opff::install(agent);
    status::install(agent);
    agent.install();

    breath::install();
}