#![feature(proc_macro_hygiene)]
#![allow(unused, non_snake_case, non_upper_case_globals, static_mut_refs, unused_unsafe, unused_must_use)]

//! Chun-Li (mod Melee m-ex de GameBanana) sur Ken, 1re couleur (c00) pour l'instant : usage perso. Coups du mod Melee (generated.rs, créé par chunli/tools/gen_chunli.py, jamais dans le dépôt)
//! sur ses animations de Melee (retarget_melee.py) ; modèle = skin Ultimate (SquidEnthusiast).
//! Étape 1 : jabs, attaque en courant, tilts, smashs, aériens, prises, coup pendant la prise.

use smash::app::sv_animcmd::{frame, wait};
use smash::app::lua_bind::*;
use smash::app::*;
use smash::lib::lua_const::*;
use smash::lua2cpp::*;
use smash::phx::{Hash40, Vector3f};
use smash::hash40;
use smash_script::macros::*;
use smash_script::*;
use smashline::*;

pub const K_HIT: u8 = 0;
pub const K_CLEAR: u8 = 1;
pub const K_COMBO: u8 = 2;
pub const K_HOLD: u8 = 3;
pub const K_LAND_ON: u8 = 4;
pub const K_LAND_OFF: u8 = 5;
pub const K_IASA: u8 = 6;
pub const K_CATCH: u8 = 7;
pub const K_SE: u8 = 8;
pub const K_VOIX: u8 = 9;

#[derive(Clone, Copy)]
pub struct Ev {
    pub f: f32,
    pub k: u8,
    pub id: u64,
    pub grp: u64,
    pub bone: &'static str,
    pub dmg: f32,
    pub angle: u64,
    pub kbg: i32,
    pub fkb: i32,
    pub bkb: i32,
    pub size: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub attr: &'static str,
    /// bruit : 0 poing, 1 pied, 2 lame, 3 électrique, 4 feu
    pub snd: u8,
    /// force du bruit : 0 faible, 1 moyen, 2 fort, 3 très fort
    pub lvl: u8,
    pub shield: i32,
    /// 1 au sol, 2 en l'air, 3 les deux
    pub sit: u8,
}

pub const E: Ev = Ev { f: 0.0, k: K_HIT, id: 0, grp: 0, bone: "top", dmg: 0.0, angle: 361, kbg: 100, fkb: 0, bkb: 0, size: 3.0,
    x: 0.0, y: 0.0, z: 0.0, attr: "collision_attr_normal", snd: 0, lvl: 1, shield: 0, sit: 3 };

pub struct Mv {
    pub script: &'static str,
    pub total: f32,
    /// 0 coup au sol, 1 smash, 2 aérien, 3 prise
    pub kind: u8,
    pub evs: &'static [Ev],
}

include!("generated.rs");

pub const BASE: &str = "ken";
/// étape par étape (demande du joueur) : d'abord sur la 1re couleur de Ken (c00) ; sa propre case à la fin
/// (les cases ajoutées à Ken plantent chez lui, même Ken normal sur c70 : test du 15/11)
pub const COSTUMES: [usize; 1] = [0];

pub(crate) fn find(script: &str) -> Option<&'static Mv> {
    MOVES.iter().find(|m| m.script == script)
}

pub(crate) unsafe fn costume(boma: *mut BattleObjectModuleAccessor) -> bool {
    let c = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR) as usize;
    COSTUMES.contains(&c)
}

fn son(snd: u8) -> i32 {
    unsafe {
        match snd {
            1 => *COLLISION_SOUND_ATTR_KEN_KICK,
            2 => *COLLISION_SOUND_ATTR_CUTUP,
            3 => *COLLISION_SOUND_ATTR_ELEC,
            4 => *COLLISION_SOUND_ATTR_FIRE,
            _ => *COLLISION_SOUND_ATTR_KEN_PUNCH,
        }
    }
}

fn niveau(lvl: u8) -> i32 {
    unsafe {
        match lvl {
            0 => *ATTACK_SOUND_LEVEL_S,
            1 => *ATTACK_SOUND_LEVEL_M,
            2 => *ATTACK_SOUND_LEVEL_L,
            _ => *ATTACK_SOUND_LEVEL_LL,
        }
    }
}

fn situation(sit: u8) -> i32 {
    unsafe {
        match sit {
            1 => *COLLISION_SITUATION_MASK_G,
            2 => *COLLISION_SITUATION_MASK_A,
            _ => *COLLISION_SITUATION_MASK_GA,
        }
    }
}

/// image ACMD d'un événement de Melee : même numérotation (une commande à l'image N de Melee agit à l'image N+1,
/// comme frame(N) dans Ultimate) ; jamais avant l'image 1
fn img(f: f32) -> f32 {
    f.max(1.0)
}

/// coups de Chun-Li (Melee) sur son animation de Melee
pub(crate) unsafe fn run(agent: &mut L2CAgentBase, script: &str) {
    let mv: &Mv = match find(script) { Some(m) => m, None => return };
    let lua = agent.lua_state_agent;
    let boma = agent.module_accessor;
    let catch_only = script == "game_catchattack";
    let has_land = mv.evs.iter().any(|e| e.k == K_LAND_ON);
    let mut cur = 0.0f32;
    let mut landing_on = false;
    let mut rebound = false;
    for e in mv.evs {
        if matches!(e.k, K_SE | K_VOIX) {
            continue;
        }
        // charge du smash : le jeu passe à la charge une image après le drapeau ; posé une image plus tôt que dans
        // Melee, la charge garde la pose de l'image de Melee (animation de charge = cette image)
        let af = if e.k == K_HOLD { (e.f - 1.0).max(1.0) } else { img(e.f) };
        if af > cur {
            frame(lua, af);
            cur = af;
        }
        if !is_excute(agent) {
            continue;
        }
        match e.k {
            K_CLEAR => {
                AttackModule::clear_all(boma);
                if mv.kind == 3 {
                    grab!(agent, *MA_MSC_CMD_GRAB_CLEAR_ALL);
                    WorkModule::on_flag(boma, *FIGHTER_STATUS_CATCH_FLAG_CATCH_WAIT);
                    GrabModule::set_rebound(boma, false);
                }
            }
            K_COMBO => { WorkModule::on_flag(boma, *FIGHTER_STATUS_ATTACK_FLAG_ENABLE_COMBO); }
            K_HOLD => { WorkModule::on_flag(boma, *FIGHTER_STATUS_ATTACK_FLAG_START_SMASH_HOLD); }
            K_LAND_ON => {
                WorkModule::on_flag(boma, *FIGHTER_STATUS_ATTACK_AIR_FLAG_ENABLE_LANDING);
                landing_on = true;
            }
            K_LAND_OFF => { WorkModule::off_flag(boma, *FIGHTER_STATUS_ATTACK_AIR_FLAG_ENABLE_LANDING); }
            K_IASA => { CancelModule::enable_cancel(boma); }
            K_CATCH => {
                if !rebound {
                    GrabModule::set_rebound(boma, true);
                    rebound = true;
                }
                CATCH(agent, e.id as i32, Hash40::new(e.bone), e.size, e.x, e.y, e.z, None, None, None,
                      *FIGHTER_STATUS_KIND_CAPTURE_PULLED, *COLLISION_SITUATION_MASK_GA);
            }
            _ => {
                if mv.kind == 2 && !has_land && !landing_on {
                    WorkModule::on_flag(boma, *FIGHTER_STATUS_ATTACK_AIR_FLAG_ENABLE_LANDING);
                    landing_on = true;
                }
                // Melee : sens de l'éjection d'après la position de l'adversaire (ATTACK_LR_CHECK_POS)
                ATTACK(agent, e.id, e.grp, Hash40::new(e.bone), e.dmg, e.angle, e.kbg, e.fkb, e.bkb, e.size, e.x, e.y,
                       e.z, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, e.shield, 0.0,
                       0, false, false, false, false, true, situation(e.sit), *COLLISION_CATEGORY_MASK_ALL,
                       *COLLISION_PART_MASK_ALL, false, Hash40::new(e.attr), niveau(e.lvl), son(e.snd),
                       if e.snd == 1 { *ATTACK_REGION_KICK } else { *ATTACK_REGION_PUNCH });
                if catch_only {
                    AttackModule::set_catch_only_all(boma, true, false);
                }
            }
        }
    }
}

/// bruits : élan du coup (sons de Ken, poing / pied selon le coup de Melee) et voix du skin (séquences de Ken)
pub(crate) unsafe fn run_son(agent: &mut L2CAgentBase, script: &str) {
    let mv: &Mv = match find(script) { Some(m) => m, None => return };
    let lua = agent.lua_state_agent;
    let mut cur = 0.0f32;
    let mut voix = false;
    if mv.kind == 3 {
        // prise : bruit d'élan de Ken (se_common_swing_06), 3 images avant la prise comme chez lui
        if let Some(c) = mv.evs.iter().find(|e| e.k == K_CATCH) {
            let af = (c.f - 3.0).max(1.0);
            frame(lua, af);
            if is_excute(agent) {
                PLAY_SE(agent, Hash40::new("se_common_swing_06"));
            }
        }
        return;
    }
    for e in mv.evs {
        if !matches!(e.k, K_SE | K_VOIX) {
            continue;
        }
        let af = img(e.f);
        if af > cur {
            frame(lua, af);
            cur = af;
        }
        if !is_excute(agent) {
            continue;
        }
        if e.k == K_VOIX {
            PLAY_SEQUENCE(agent, Hash40::new("seq_ken_rnd_attack_ll"));
            voix = true;
            continue;
        }
        let se = match (e.snd == 1, e.lvl) {
            (false, 0) => "se_ken_swing_punch_s",
            (false, 1) => "se_ken_swing_punch_m",
            (false, _) => "se_ken_swing_punch_l",
            (true, 0) => "se_ken_swing_kick_s",
            (true, 1) => "se_ken_swing_kick_m",
            (true, _) => "se_ken_swing_kick_l",
        };
        PLAY_SE(agent, Hash40::new(se));
        if !voix && mv.kind != 3 && !script.starts_with("game_catchattack") {
            let v = match (mv.kind, e.lvl) {
                (1, _) => "seq_ken_rnd_attack_ll",
                (_, 0) => "seq_ken_rnd_attack_s",
                (_, 1) => "seq_ken_rnd_attack_m",
                _ => "seq_ken_rnd_attack_l",
            };
            PLAY_SEQUENCE(agent, Hash40::new(v));
            voix = true;
        }
    }
}

/// effets : éclair d'impact sur le coup le plus fort de chaque image de coup (ceux de Ken, flammes comprises, sont
/// retirés : ils suivaient ses propres coups)
pub(crate) unsafe fn run_effet(agent: &mut L2CAgentBase, script: &str) {
    let mv: &Mv = match find(script) { Some(m) => m, None => return };
    let lua = agent.lua_state_agent;
    let mut cur = 0.0f32;
    let mut last = -1.0f32;
    for (i, e) in mv.evs.iter().enumerate() {
        if e.k != K_HIT || e.f == last {
            continue;
        }
        last = e.f;
        let best = mv.evs.iter().filter(|o| o.k == K_HIT && o.f == e.f)
            .fold(*e, |a, o| if o.dmg * o.size > a.dmg * a.size { *o } else { a });
        let af = img(e.f);
        if af > cur {
            frame(lua, af);
            cur = af;
        }
        if is_excute(agent) {
            EFFECT_ALPHA(agent, Hash40::new("sys_attack_impact"), Hash40::new(best.bone), best.x, best.y, best.z, 0, 0, 0,
                         0.55 + 0.03 * best.dmg.min(15.0), 0, 0, 0, 0, 0, 360, false, 0.5);
        }
    }
}

/// Ken a des jabs « de près » (autres animations) : Chun-Li n'a que les siens -> jab normal
unsafe extern "C" fn cl_frame(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    if StatusModule::status_kind(boma) != *FIGHTER_STATUS_KIND_ATTACK {
        return;
    }
    let m = MotionModule::motion_kind(boma);
    let fort = m == hash40("attack_11_near_s") || m == hash40("attack_near_s");
    let faible = m == hash40("attack_near_w") || m == hash40("attack_11_near_w");
    if fort || faible {
        let cible = if fort { "attack_11_s" } else { "attack_11_w" };
        MotionModule::change_motion(boma, Hash40::new(cible), 0.0, 1.0, false, 0.0, false, false);
    }
}

macro_rules! scripts {
    ($($g:ident, $s:ident, $e:ident => $n:literal),* $(,)?) => {
        $(
            unsafe extern "C" fn $g(agent: &mut L2CAgentBase) { run(agent, $n); }
            unsafe extern "C" fn $s(agent: &mut L2CAgentBase) { run_son(agent, $n); }
            unsafe extern "C" fn $e(agent: &mut L2CAgentBase) { run_effet(agent, $n); }
        )*
        fn install_scripts() {
            let mut a = Agent::new(BASE);
            a.set_costume(COSTUMES.to_vec());
            $(
                if find($n).is_some() {
                    a.acmd($n, $g, Priority::Low);
                    let r = &$n[5..];
                    a.acmd(Box::leak(format!("sound_{}", r).into_boxed_str()), $s, Priority::Low);
                    a.acmd(Box::leak(format!("effect_{}", r).into_boxed_str()), $e, Priority::Low);
                }
            )*
            a.on_line(Main, cl_frame);
            a.install();
        }
    };
}

scripts! {
    g01, s01, e01 => "game_attack11w",
    g02, s02, e02 => "game_attack11s",
    g03, s03, e03 => "game_attack11nears",
    g04, s04, e04 => "game_attacknearw",
    g05, s05, e05 => "game_attack12",
    g06, s06, e06 => "game_attack13",
    g07, s07, e07 => "game_attackdash",
    g08, s08, e08 => "game_attacks3w",
    g09, s09, e09 => "game_attacks3s",
    g10, s10, e10 => "game_attackhi3w",
    g11, s11, e11 => "game_attackhi3s",
    g12, s12, e12 => "game_attacklw3w",
    g13, s13, e13 => "game_attacklw3s",
    g14, s14, e14 => "game_attacks4",
    g15, s15, e15 => "game_attackhi4",
    g16, s16, e16 => "game_attacklw4",
    g17, s17, e17 => "game_attackairn",
    g18, s18, e18 => "game_attackairf",
    g19, s19, e19 => "game_attackairb",
    g20, s20, e20 => "game_attackairhi",
    g21, s21, e21 => "game_attackairlw",
    g22, s22, e22 => "game_catch",
    g23, s23, e23 => "game_catchdash",
    g24, s24, e24 => "game_catchturn",
    g25, s25, e25 => "game_catchattack",
}

#[skyline::main(name = "chun_li_melee")]
pub fn main() {
    install_scripts();
}
