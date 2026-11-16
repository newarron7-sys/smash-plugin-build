#![feature(proc_macro_hygiene)]

//! Lylat Cruise sans inclinaison, pour Smash Ultimate 13.0.4.
//! Repris de HewDraw Remix (fighters/common/src/function_hooks/stage_hazards.rs, `lylat_no_rot`,
//! offset de la version 13.0.4, avant le passage en 13.0.5) ; HewDraw Remix est sous licence AGPL-3.0.
//! Le mode de rotation 3 (vaisseau qui penche) est remplacé par le mode 5.

#[skyline::hook(offset = 0x2981EDC, inline)]
unsafe fn lylat_no_rot(ctx: &mut skyline::hooks::InlineCtx) {
    if ctx.registers[8].x() == 3 {
        ctx.registers[8].set_x(5);
    }
}

#[skyline::main(name = "lylat_sans_inclinaison")]
pub fn main() {
    skyline::install_hooks!(lylat_no_rot);
}
