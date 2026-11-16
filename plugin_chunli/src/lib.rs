#![feature(proc_macro_hygiene)]
#![allow(unused, non_snake_case, non_upper_case_globals, static_mut_refs, unused_unsafe, unused_must_use)]

//! Chun-Li (mod Melee m-ex de GameBanana) sur Ken, costumes c70-c77 (entrée « chun-li » de l'écran des persos du
//! joueur) : usage perso. Coups du mod Melee (generated.rs, créé par chunli/tools/gen_chunli.py, jamais dans le dépôt)
//! sur ses animations de Melee (retarget_melee.py) ; modèle = skin Ultimate (SquidEnthusiast).
//! Étape 1 : jabs, attaque en courant, tilts, smashs, aériens, prises, coup pendant la prise.
//! Étape 2 : spéciaux sur ceux de Ken (Kikoken = Hadoken, Spinning Bird Kick = Tatsumaki, haut B = Shoryuken,
//! bas B = Focus), coups et animations de Melee.
//! Étape 3 : lancers de Melee ; côté B avec l'avancée de Melee ; coups x 1,3.
//! v8 : côté B à position imposée, tour plus lent ; adversaire tenu droit, couché sur le dos au lancer bas.
//! v12 : côté B qui retient l'adversaire, un peu plus long ; bas B en l'air = plongée Hazanshu (sans charge).
//! v13 : +6 images d'étourdissement sur tous ses coups (STUN_PLUS).

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
/// spéciaux : tir du projectile (Projectile Flag de Melee), état du corps (id 0 normal, 1 invincible, 2 intangible),
/// passage en l'air (Set Jump State)
pub const K_SHOOT: u8 = 10;
pub const K_BODY: u8 = 11;
pub const K_JUMP: u8 = 12;
/// drapeaux des statuts de Ken (ajoutés par le plugin) : id 1 / 2 chute du Hadoken en l'air on / off,
/// 3 armure du Focus coupée
pub const K_RYU: u8 = 13;
/// lancers : ATTACK_ABS (id 0 lancer, 1 lâcher si l'adversaire se libère) et moment du lâcher
pub const K_THROW: u8 = 14;
pub const K_RELEASE: u8 = 15;

/// taille des coups x 1,3 (demande du joueur : « élargis ses hitbox en général, elle touche pas assez bien »)
pub const AGRANDI: f32 = 1.3;

/// images d'étourdissement (hitstun) ajoutées à tous ses coups (demande du joueur, 16/11 : « ses coups ne stun pas
/// assez, l'adversaire peut réagir beaucoup trop vite ») ; réglage commun, pas les lancers
pub const STUN_PLUS: f32 = 6.0;

/// intangibilité posée par un spécial (haut B, bas B) : remise à la normale si le statut est quitté avant la fin
static mut INTANGIBLE: [bool; 8] = [false; 8];

unsafe fn entree(boma: *mut BattleObjectModuleAccessor) -> usize {
    (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7)
}

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
/// entrée « chun-li » du ui_chara_db.prc du joueur : fighter_kind_ken, color_start_index 70 (validé en jeu le 15/11 ;
/// les plantages d'avant venaient de fichiers coupés par le transfert FTP, pas de la case)
pub const COSTUMES: [usize; 8] = [70, 71, 72, 73, 74, 75, 76, 77];

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
    // spéciaux : drapeaux que le statut de Ken attend, aux images de Chun-Li
    let mut liste: Vec<Ev> = mv.evs.to_vec();
    if let Some(t) = mv.evs.iter().find(|e| e.k == K_SHOOT) {
        // Hadoken : comme chez Ken (chute en l'air de 3 images avant le tir à 15 images après)
        liste.push(Ev { f: (t.f - 3.0).max(1.0), k: K_RYU, id: 1, ..E });
        liste.push(Ev { f: t.f + 15.0, k: K_RYU, id: 2, ..E });
    }
    if script == "game_speciallwstart" || script == "game_specialairlwstart" {
        // Focus de Ken : pas d'armure (Chun-Li est intangible comme dans Melee)
        liste.push(Ev { f: 1.0, k: K_RYU, id: 3, ..E });
    }
    liste.sort_by(|a, b| a.f.partial_cmp(&b.f).unwrap_or(core::cmp::Ordering::Equal));
    for e in liste.iter() {
        if matches!(e.k, K_SE | K_VOIX) {
            continue;
        }
        // charge du smash : le jeu passe à la charge une image après le drapeau ; posé une image plus tôt que dans
        // Melee, la charge garde la pose de l'image de Melee (animation de charge = cette image)
        let af = if e.k == K_HOLD { (e.f - 1.0).max(1.0) } else if e.k == K_THROW { 0.0 } else { img(e.f) };
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
            K_THROW => {
                let kind = if e.id == 0 { *FIGHTER_ATTACK_ABSOLUTE_KIND_THROW } else { *FIGHTER_ATTACK_ABSOLUTE_KIND_CATCH };
                // lancer arrière : éjecté derrière elle (Melee), sans la retourner
                let lr = if script == "game_throwb" && e.id == 0 { *ATTACK_LR_CHECK_B } else { *ATTACK_LR_CHECK_F };
                smash_script::macros::ATTACK_ABS(agent, kind, 0, e.dmg, e.angle, e.kbg, e.fkb, e.bkb, 0.0, 1.0, lr, 0.0,
                                                 true, Hash40::new(e.attr), *ATTACK_SOUND_LEVEL_S,
                                                 *COLLISION_SOUND_ATTR_NONE, *ATTACK_REGION_THROW);
            }
            K_RELEASE => {
                smash_script::macros::CHECK_FINISH_CAMERA(agent, 10, 7);
                let target = WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_OBJECT);
                let group = WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_HIT_GROUP);
                let no = WorkModule::get_int64(boma, *FIGHTER_STATUS_THROW_WORK_INT_TARGET_HIT_NO);
                smash_script::macros::ATK_HIT_ABS(agent, *FIGHTER_ATTACK_ABSOLUTE_KIND_THROW, Hash40::new("throw"),
                                                  target, group, no);
                AttackModule::clear_all(boma);
            }
            K_SHOOT => { WorkModule::on_flag(boma, *FIGHTER_RYU_STATUS_WORK_ID_SPECIAL_N_FLAG_SHOOT); }
            K_BODY => {
                let st = match e.id { 1 => *HIT_STATUS_INVINCIBLE, 2 => *HIT_STATUS_XLU, _ => *HIT_STATUS_NORMAL };
                HitModule::set_whole(boma, HitStatus(st), 0);
                INTANGIBLE[entree(boma)] = e.id != 0;
            }
            K_JUMP => {
                // Shoryuken de Ken : demi-tour possible, force décidée, saut (Chun-Li quitte le sol à cette image)
                WorkModule::on_flag(boma, *FIGHTER_RYU_STATUS_WORK_ID_SPECIAL_HI_FLAG_REVERSE_LR);
                WorkModule::on_flag(boma, *FIGHTER_RYU_STATUS_WORK_ID_SPECIAL_HI_FLAG_DECIDE_STRENGTH);
                WorkModule::on_flag(boma, *FIGHTER_RYU_STATUS_WORK_ID_SPECIAL_HI_FLAG_JUMP);
            }
            K_RYU => {
                match e.id {
                    1 => { WorkModule::on_flag(boma, *FIGHTER_RYU_STATUS_WORK_ID_SPECIAL_N_FLAG_SPECIAL_FALL); }
                    2 => { WorkModule::off_flag(boma, *FIGHTER_RYU_STATUS_WORK_ID_SPECIAL_N_FLAG_SPECIAL_FALL); }
                    _ => { WorkModule::on_flag(boma, *FIGHTER_RYU_STATUS_WORK_ID_SPECIAL_LW_FLAG_DISABLE_SUPER_ARMOR); }
                }
            }
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
                ATTACK(agent, e.id, e.grp, Hash40::new(e.bone), e.dmg, e.angle, e.kbg, e.fkb, e.bkb, e.size * AGRANDI, e.x, e.y,
                       e.z, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, e.shield, 0.0,
                       0, false, false, false, false, true, situation(e.sit), *COLLISION_CATEGORY_MASK_ALL,
                       *COLLISION_PART_MASK_ALL, false, Hash40::new(e.attr), niveau(e.lvl), son(e.snd),
                       if e.snd == 1 { *ATTACK_REGION_KICK } else { *ATTACK_REGION_PUNCH });
                AttackModule::set_add_reaction_frame(boma, e.id as i32, STUN_PLUS, false);
                if catch_only || mv.kind == 5 {
                    // coup pendant la prise / le lancer : seulement sur l'adversaire tenu (comme Ken)
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

static mut SBK: [bool; 8] = [false; 8];
static mut SBK_X: [f32; 8] = [0.0; 8];
static mut SBK_LR: [f32; 8] = [1.0; 8];
static mut SBK_D: [f32; 8] = [0.0; 8];
/// tour du Spinning Bird Kick moins rapide que la fin du départ de Melee (v7 : « beaucoup trop loin et trop vite,
/// l'adversaire ne reste pas dans les coups »)
/// v12 (16/11, « augmente un peu la distance ») : 0,6 -> 0,8
pub const SBK_BOUCLE: f32 = 0.8;

fn depl(cle: &str) -> &'static [f32] {
    DEPL_S.iter().find(|(k, _)| *k == cle).map(|(_, v)| *v).unwrap_or(&[])
}

/// côté B (Spinning Bird Kick) sur le Tatsumaki de Ken : sa position est IMPOSÉE à chaque image (départ + avancée
/// cumulée) : la vitesse du statut de Ken ne s'ajoute plus (v7 : elle s'ajoutait, beaucoup trop loin / trop vite).
/// Avancée de Melee image par image au départ et à la fin, UN tour à SBK_BOUCLE x la vitesse de fin du départ
unsafe fn spinning_bird_kick(boma: *mut BattleObjectModuleAccessor, st: i32) {
    let e = entree(boma);
    let depart = st == *FIGHTER_STATUS_KIND_SPECIAL_S || st == *FIGHTER_RYU_STATUS_KIND_SPECIAL_S_COMMAND;
    let boucle = st == *FIGHTER_RYU_STATUS_KIND_SPECIAL_S_LOOP;
    let fin = st == *FIGHTER_RYU_STATUS_KIND_SPECIAL_S_END;
    if !(depart || boucle || fin) {
        if SBK[e] {
            KineticModule::resume_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_STOP);
            SBK[e] = false;
        }
        return;
    }
    if !SBK[e] {
        SBK[e] = true;
        SBK_X[e] = PostureModule::pos_x(boma);
        SBK_LR[e] = PostureModule::lr(boma);
        SBK_D[e] = 0.0;
    }
    KineticModule::suspend_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_STOP);
    KineticModule::clear_speed_energy_id(boma, *FIGHTER_KINETIC_ENERGY_ID_STOP);
    let air = StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR;
    let f = MotionModule::frame(boma).max(0.0) as usize;
    let v = if boucle {
        if MotionModule::frame(boma) >= MotionModule::end_frame(boma) - 1.0 {
            StatusModule::change_status_request_from_script(boma, *FIGHTER_RYU_STATUS_KIND_SPECIAL_S_END, true);
        }
        SBK_BOUCLE * if air { DEPL_S_BOUCLE_AIR } else { DEPL_S_BOUCLE }
    } else {
        let t = depl(match (depart, air) { (true, false) => "s_start", (true, true) => "s_air_start",
                                            (false, false) => "s_end", (false, true) => "s_air_end" });
        if t.is_empty() { 0.0 } else { t[f.min(t.len() - 1)] }
    };
    SBK_D[e] += v;
    let pos = Vector3f { x: SBK_X[e] + SBK_D[e] * SBK_LR[e], y: PostureModule::pos_y(boma), z: PostureModule::pos_z(boma) };
    PostureModule::set_pos(boma, &pos);
}

/// bas B en l'air = Hazanshu de Melee (vidéo du mod, 16/11) : elle plonge en avant vers le bas avec son coup de pied
/// (coup final de Melee, 10 %). Le Focus de Ken (charge avec armure) n'est pas gardé : dès la fin du départ (3
/// images), l'attaque part ; plongée imposée de l'image 5 de Melee (2 de l'attaque) jusqu'au sol, puis atterrissage
pub const HAZ_VX: f32 = 1.6;
pub const HAZ_VY: f32 = -1.3;
static mut HAZ: [bool; 8] = [false; 8];

unsafe fn hazanshu_air(boma: *mut BattleObjectModuleAccessor, st: i32) {
    let e = entree(boma);
    let air = StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR;
    if st == *FIGHTER_STATUS_KIND_SPECIAL_LW && air {
        HAZ[e] = true;
        if MotionModule::frame(boma) >= MotionModule::end_frame(boma) - 1.0 {
            StatusModule::change_status_request_from_script(boma, *FIGHTER_RYU_STATUS_KIND_SPECIAL_LW_ATTACK, false);
        }
        return;
    }
    if st != *FIGHTER_RYU_STATUS_KIND_SPECIAL_LW_ATTACK || !HAZ[e] {
        HAZ[e] = false;
        return;
    }
    if !air {
        // elle touche le sol : atterrissage (comme dans Melee)
        HAZ[e] = false;
        StatusModule::change_status_request_from_script(boma, *FIGHTER_STATUS_KIND_LANDING, false);
        return;
    }
    KineticModule::clear_speed_all(boma);
    if MotionModule::frame(boma) >= 2.0 {
        let lr = PostureModule::lr(boma);
        PostureModule::add_pos(boma, &Vector3f { x: HAZ_VX * lr, y: HAZ_VY, z: 0.0 });
    }
}

/// Ken a des jabs « de près » (autres animations) : Chun-Li n'a que les siens -> jab normal
unsafe extern "C" fn cl_frame(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let st = StatusModule::status_kind(boma);
    if INTANGIBLE[entree(boma)] && st != *FIGHTER_STATUS_KIND_SPECIAL_HI && st != *FIGHTER_STATUS_KIND_SPECIAL_LW
        && st != *FIGHTER_RYU_STATUS_KIND_SPECIAL_LW_ATTACK && st != *FIGHTER_RYU_STATUS_KIND_SPECIAL_HI_COMMAND
        && st != *FIGHTER_RYU_STATUS_KIND_SPECIAL_HI_JUMP {
        HitModule::set_whole(boma, HitStatus(*HIT_STATUS_NORMAL), 0);
        INTANGIBLE[entree(boma)] = false;
    }
    spinning_bird_kick(boma, st);
    hazanshu_air(boma, st);
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
                    // spéciaux : sons et voix de Ken gardés (le skin a remplacé ses voix par celles de Chun-Li) ;
                    // effets de Ken gardés pour le Hadoken (boule de Ken), remplacés pour les autres
                    if !r.starts_with("special") && !r.starts_with("throw") {
                        a.acmd(Box::leak(format!("sound_{}", r).into_boxed_str()), $s, Priority::Low);
                    }
                    if !(r == "specialn" || r == "specialairn") {
                        a.acmd(Box::leak(format!("effect_{}", r).into_boxed_str()), $e, Priority::Low);
                    }
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
    g26, s26, e26 => "game_specialn",
    g27, s27, e27 => "game_specialairn",
    g28, s28, e28 => "game_specialsstart",
    g29, s29, e29 => "game_specialairsstart",
    g30, s30, e30 => "game_specials",
    g31, s31, e31 => "game_specialairs",
    g32, s32, e32 => "game_specialsend",
    g33, s33, e33 => "game_specialairsend",
    g34, s34, e34 => "game_specialhi",
    g35, s35, e35 => "game_specialhicommand",
    g36, s36, e36 => "game_specialairhi",
    g37, s37, e37 => "game_specialairhicommand",
    g38, s38, e38 => "game_speciallwstart",
    g39, s39, e39 => "game_speciallw",
    g40, s40, e40 => "game_specialairlwstart",
    g41, s41, e41 => "game_specialairlw",
    g42, s42, e42 => "game_throwf",
    g43, s43, e43 => "game_throwb",
    g44, s44, e44 => "game_throwhi",
    g45, s45, e45 => "game_throwlw",
}

#[skyline::main(name = "chun_li_melee")]
pub fn main() {
    install_scripts();
}
