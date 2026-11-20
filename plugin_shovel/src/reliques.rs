//! Bas B de Shovel Knight : les reliques (REX, FitShovel.pac : actions 0x115, 0x11C-0x120, sous-actions Think /
//! Select / reliques).
//!
//! REX : chaque coup qui touche rapporte de l'or (V11000035 : +50 à +250 selon le coup). Bas B sans relique = menu
//! (Think) : le stick pointe une relique (V11000044, 10 cases), B l'achète si l'or suffit (prix 1000 / 2000 / 3000 /
//! 4000) -> Select, relique gardée (V11000045), magie à 100 (V11000038) ; sinon « NOT ENOUGH » ; B au neutre ferme le
//! menu. Bas B avec relique = la relique (coût en magie) ; magie à 0 -> relique perdue (menu au bas B suivant).
//! Golden Armour : armure dorée gardée (modèle doré), armure pendant ses coups (seuil de REX par coup, V11000063).
//!
//! Affichage (comme le portail de Primid) : carrés du modèle (menu_shovel.py, textures de REX) montrés / cachés ici.
//! v9 (étape 1) : menu, or, magie, Morph Lance, Blade Charge, Healing Chalice, Golden Armour. Les reliques à projectile
//! (Throwing Anchor, Bubbles, Mobile Gear, Buzzsaw, Chaos Sphere, Fishing Rod) : à l'étape 2 (pas encore en vente).

use smash::app::lua_bind::*;
use smash::app::*;
use smash::hash40;
use smash::lib::lua_const::*;
use smash::lib::L2CValue;
use smash::lua2cpp::*;
use smash::phx::{Hash40, Vector3f};
use smash_script::macros::*;
use smash_script::*;
use smashline::*;

use crate::speciaux::{rien, slot};

// ------------------------------------------------------------------ morceaux de special_lw / special_air_lw
// Think (61) | Select (30) | Morph Lance (58) | Blade Charge = ShovelBurst (33) | Drink (40) | ToadTaunt (60) au sol,
// AppealAirGoldArmor (127) en l'air
const M_FIN: f32 = 60.0;
const A_DEB: f32 = 61.0;
const A_FIN: f32 = 90.0;
const L_DEB: f32 = 91.0;
const L_LONG: f32 = 58.0;
const B_DEB: f32 = 149.0;
const B_LONG: f32 = 33.0;
const D_DEB: f32 = 182.0;
const D_LONG: f32 = 40.0;
const G_DEB: f32 = 222.0;
const G_LONG_SOL: f32 = 60.0;
const G_LONG_AIR: f32 = 127.0;

const OR_MAX: i32 = 9999;

static mut OR: [i32; 8] = [0; 8];
static mut MANA: [i32; 8] = [0; 8];
static mut RELIQUE: [i32; 8] = [0; 8];
static mut CURSEUR: [i32; 8] = [10; 8];
static mut PAUVRE: [i32; 8] = [0; 8];
static mut MENU: [bool; 8] = [false; 8];
static mut PHASE: [u8; 8] = [0; 8];          // 0 menu, 1 achat, 2 relique
static mut T: [i32; 8] = [0; 8];             // images depuis le début du bas B
static mut PREC: [f32; 8] = [0.0; 8];        // image précédente (événements des coups)
static mut ELAN: [f32; 8] = [0.0; 8];        // élan de la Morph Lance
static mut AFF: [u64; 8] = [u64::MAX; 8];    // clé de l'affichage en place
static mut LANCE_AVANT: [bool; 8] = [false; 8];
static mut LANCER_PREC: [bool; 8] = [false; 8];

/// reliques en vente à cette étape
fn disponible(r: i32) -> bool {
    matches!(r, 3 | 7 | 8 | 10)
}

/// prix de REX (Think : V1100003A) selon la case du menu
fn prix(curseur: i32) -> i32 {
    match curseur {
        0..=2 => 1000,
        3..=6 => 2000,
        7 | 8 => 3000,
        9 => 4000,
        _ => 0,
    }
}

/// or rapporté par un coup qui touche (REX : 12070200 n V11000035 sous « a touché »)
unsafe fn or_du_coup(boma: *mut BattleObjectModuleAccessor) -> i32 {
    let m = MotionModule::motion_kind(boma);
    let f = MotionModule::frame(boma);
    let k = |n: &str| m == hash40(n);
    if k("attack_11") || k("attack_12") || k("attack_13") || k("attack_air_n") || k("attack_air_hi") || k("attack_lw3") {
        80
    } else if k("attack_dash") {
        250
    } else if k("attack_s3_s") || k("attack_air_f") || k("attack_air_b") || k("attack_hi4") {
        100
    } else if k("attack_hi3") {
        150
    } else if k("attack_s4_s") || k("attack_s4_s2") {
        200
    } else if k("attack_lw4") {
        // AttackLw4 (saut) 50, AttackLw42 (retombée) 200
        if f >= crate::LW42_DEBUT { 200 } else { 50 }
    } else if k("attack_air_lw") {
        if f < crate::AIRLW_DEBUT { 100 } else { 200 }
    } else if k("catch_attack") {
        50
    } else if k("special_hi_start") {
        200
    } else if k("special_air_hi") || k("special_hi") || k("special_hi_hold") {
        100
    } else if k("special_s2") || k("special_air_s2") {
        50
    } else if k("special_lw") || k("special_air_lw") {
        if f >= B_DEB && f < B_DEB + B_LONG { 150 } else { 0 }
    } else {
        0
    }
}

/// armure de la Golden Armour pendant ses coups (REX : 1E000200 3 V11000063, seuil en %)
unsafe fn armure(boma: *mut BattleObjectModuleAccessor) -> f32 {
    let m = MotionModule::motion_kind(boma);
    let f = MotionModule::frame(boma);
    let k = |n: &str| m == hash40(n);
    if k("attack_11") || k("attack_12") || k("attack_lw4_hold") || k("attack_s4_hold") || k("attack_hi4_hold") {
        8.0
    } else if k("attack_air_lw") || k("attack_air_n") {
        9.0
    } else if k("attack_13") || k("attack_air_f") || k("attack_s3_s") || k("special_hi_start") || k("special_air_hi")
        || k("special_hi") || k("special_hi_hold") || k("special_s2") || k("special_air_s2") {
        10.0
    } else if k("attack_lw3") {
        11.0
    } else if k("attack_hi3") {
        13.0
    } else if k("attack_air_b") || k("attack_air_hi") || k("attack_dash") {
        14.0
    } else if k("attack_s4_s") || k("attack_s4_s2") || k("attack_hi4") || k("attack_lw4") {
        20.0
    } else if k("special_s1") || k("special_air_s1") {
        7.0
    } else if k("special_lw") || k("special_air_lw") {
        if f >= G_DEB { 30.0 } else { 10.0 }
    } else if k("appeal_hi_l") || k("appeal_hi_r") || k("appeal_s_l") || k("appeal_s_r") || k("appeal_lw_l") || k("appeal_lw_r") {
        10.0
    } else {
        0.0
    }
}

unsafe fn remise_a_zero(e: usize) {
    OR[e] = 0;
    MANA[e] = 0;
    RELIQUE[e] = 0;
    CURSEUR[e] = 10;
    PAUVRE[e] = 0;
    MENU[e] = false;
}

// ------------------------------------------------------------------ affichage (carrés du modèle)
fn noms_tous(cote: &str) -> Vec<String> {
    let mut v = Vec::new();
    for r in [1, 2, 4, 5, 6, 7, 8, 10] {
        v.push(format!("sk_grille{}_{}", r, cote));
    }
    for r in 1..=10 {
        v.push(format!("sk_choix{}_{}", r, cote));
    }
    for p in [1000, 2000, 3000, 4000] {
        v.push(format!("sk_prix{}_{}", p, cote));
    }
    v.push(format!("sk_pauvre_{}", cote));
    for m in (10..=100).step_by(10) {
        v.push(format!("sk_mana{}_{}", m, cote));
    }
    for n in 1..=4 {
        for k in 0..n {
            for c in 0..10 {
                v.push(format!("sk_or{}{}_{}_{}", n, k, c, cote));
            }
        }
    }
    v
}

unsafe fn vis(boma: *mut BattleObjectModuleAccessor, n: &str, on: bool) {
    ModelModule::set_mesh_visibility(boma, Hash40::new(n), on);
}

unsafe fn affichage(boma: *mut BattleObjectModuleAccessor, e: usize) {
    let lr = PostureModule::lr(boma);
    let cote = if lr >= 0.0 { "d" } else { "g" };
    let or = OR[e].clamp(0, OR_MAX);
    let mana = if RELIQUE[e] > 0 { ((MANA[e] + 9) / 10 * 10).clamp(0, 100) } else { 0 };
    let menu = MENU[e];
    let cur = if menu { CURSEUR[e] } else { 10 };
    let pauvre = menu && PAUVRE[e] > 0;
    let cle = (or as u64) | ((mana as u64) << 14) | ((cur as u64) << 21) | ((menu as u64) << 26)
        | ((pauvre as u64) << 27) | (((lr >= 0.0) as u64) << 28);
    if AFF[e] == cle {
        return;
    }
    AFF[e] = cle;
    for c in ["d", "g"] {
        for n in noms_tous(c) {
            vis(boma, &n, false);
        }
    }
    if or > 0 || menu {
        let s = format!("{}", or);
        let n = s.len();
        for (k, ch) in s.chars().enumerate() {
            vis(boma, &format!("sk_or{}{}_{}_{}", n, k, ch, cote), true);
        }
    }
    if mana >= 10 {
        vis(boma, &format!("sk_mana{}_{}", mana, cote), true);
    }
    if menu {
        for r in [1, 2, 4, 5, 6, 7, 8, 10] {
            vis(boma, &format!("sk_grille{}_{}", r, cote), true);
        }
        if pauvre {
            vis(boma, &format!("sk_pauvre_{}", cote), true);
        } else if cur < 10 {
            vis(boma, &format!("sk_choix{}_{}", cur + 1, cote), true);
            vis(boma, &format!("sk_prix{}_{}", prix(cur), cote), true);
        }
    }
}

/// armure dorée (Golden Armour) : corps et pelle dorés ; pelle cachée pendant le Healing Chalice (REX : 0B000200 2 5)
unsafe fn modele(boma: *mut BattleObjectModuleAccessor, e: usize) {
    let dore = RELIQUE[e] == 10;
    let m = MotionModule::motion_kind(boma);
    let f = MotionModule::frame(boma);
    let lw = (m == hash40("special_lw") || m == hash40("special_air_lw")) && StatusModule::status_kind(boma) == *FIGHTER_STATUS_KIND_SPECIAL_LW;
    let boit = lw && f >= D_DEB && f < D_DEB + D_LONG;
    let poings = m == hash40("attack_s3_s") || m == hash40("attack_air_f") || m == hash40("attack_air_b");
    // side B (REX : pelle cachée, Propeller Dagger dans la main gauche)
    let dague = m == hash40("special_s1") || m == hash40("special_air_s1") || m == hash40("special_s2") || m == hash40("special_air_s2");
    vis(boma, "sk_dague", dague);
    let pelle = !poings && !boit && !dague;
    vis(boma, "shovelbody", !dore);
    vis(boma, "goldbody", dore);
    vis(boma, "shovelr", pelle && !dore);
    vis(boma, "goldshovel", pelle && dore);
}

// ------------------------------------------------------------------ chaque image (Shovel Knight)
pub unsafe fn image(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    let st = StatusModule::status_kind(boma);
    if st == *FIGHTER_STATUS_KIND_DEAD || st == *FIGHTER_STATUS_KIND_REBIRTH || st == *FIGHTER_STATUS_KIND_ENTRY {
        remise_a_zero(e);
    }
    // or : chaque coup qui touche
    if AttackModule::is_infliction(boma, *COLLISION_KIND_MASK_HIT) {
        OR[e] = (OR[e] + or_du_coup(boma)).min(OR_MAX);
    }
    // lancers : or à l'entrée du lancer (REX : dans le script du lancer, sans condition)
    let lancer = st == *FIGHTER_STATUS_KIND_THROW;
    if lancer && !LANCER_PREC[e] {
        let m = MotionModule::motion_kind(boma);
        OR[e] = (OR[e] + if m == hash40("throw_f") { 100 } else { 50 }).min(OR_MAX);
    }
    LANCER_PREC[e] = lancer;
    if st != *FIGHTER_STATUS_KIND_SPECIAL_LW {
        MENU[e] = false;
    }
    // Golden Armour : armure pendant ses coups
    if RELIQUE[e] == 10 {
        let a = armure(boma);
        if a > 0.0 {
            damage!(fighter, *MA_MSC_DAMAGE_DAMAGE_NO_REACTION, *DAMAGE_NO_REACTION_MODE_DAMAGE_POWER, a);
        } else {
            damage!(fighter, *MA_MSC_DAMAGE_DAMAGE_NO_REACTION, *DAMAGE_NO_REACTION_MODE_NORMAL, 0);
        }
    }
    modele(boma, e);
    affichage(boma, e);
}

// ------------------------------------------------------------------ statut du bas B
unsafe fn sol(boma: *mut BattleObjectModuleAccessor) -> bool {
    StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND
}

unsafe fn physique(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    if sol(boma) {
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_GROUND_STOP);
        GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_GROUND_CLIFF_STOP));
    } else {
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_FALL);
        GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
    }
}

unsafe fn finir(fighter: &mut L2CFighterCommon) {
    if sol(fighter.module_accessor) {
        fighter.change_status(FIGHTER_STATUS_KIND_WAIT.into(), false.into());
    } else {
        fighter.change_status(FIGHTER_STATUS_KIND_FALL.into(), false.into());
    }
}

/// stick -> case du menu (REX, sous-routine de Think : 8 directions, haut / bas forts et faibles, neutre = 10)
unsafe fn curseur(boma: *mut BattleObjectModuleAccessor) -> i32 {
    let x = ControlModule::get_stick_x(boma);
    let y = ControlModule::get_stick_y(boma);
    if x < -0.4 && y > -0.4 && y < 0.4 {
        0
    } else if x < -0.4 && y >= 0.4 {
        1
    } else if x < -0.4 && y <= -0.4 {
        5
    } else if x > 0.4 && y <= -0.4 {
        7
    } else if x > 0.4 && y >= 0.4 {
        3
    } else if x > 0.4 {
        4
    } else if y < -0.8 {
        6
    } else if y > 0.8 {
        2
    } else if y > 0.4 {
        8
    } else if y < -0.4 {
        9
    } else {
        10
    }
}

/// coups de REX d'une relique (scripts générés : game_sk_lance / game_sk_burst…), images (prec, f] du morceau
unsafe fn coups(fighter: &mut L2CFighterCommon, script: &str, prec: f32, f: f32) {
    let mv = match crate::find(script) { Some(m) => m, None => return };
    for ev in mv.evs.iter() {
        if ev.rate > 0.0 || ev.body >= 0 || !(ev.f > prec && ev.f <= f) {
            continue;
        }
        if ev.clear {
            AttackModule::clear_all(fighter.module_accessor);
            continue;
        }
        let (x2, y2, z2) = if ev.cap { (Some(ev.x2), Some(ev.y2), Some(ev.z2)) } else { (None, None, None) };
        ATTACK(fighter, ev.id, 0, Hash40::new(ev.bone), ev.dmg, ev.angle, ev.kbg, ev.fkb, ev.bkb, ev.size, ev.x, ev.y, ev.z,
               x2, y2, z2, ev.hitlag, ev.sdi, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_F, false, 0, ev.trip, 0,
               false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL,
               *COLLISION_PART_MASK_ALL, false, Hash40::new(ev.attr), *ATTACK_SOUND_LEVEL_L, crate::son(ev.attr),
               *ATTACK_REGION_SWORD);
    }
}

unsafe fn effet(boma: *mut BattleObjectModuleAccessor, nom: &str, dy: f32, dz: f32, taille: f32) {
    let p = PostureModule::pos(boma);
    let lr = PostureModule::lr(boma);
    EffectModule::req(boma, Hash40::new(nom), &Vector3f { x: (*p).x + dz * lr, y: (*p).y + dy, z: 0.0 },
                      &Vector3f { x: 0.0, y: 0.0, z: 0.0 }, taille, 0, 0, false, 0);
}

unsafe fn debut_relique(fighter: &mut L2CFighterCommon, e: usize) {
    let boma = fighter.module_accessor;
    PHASE[e] = 2;
    let f = match RELIQUE[e] {
        3 => L_DEB,
        7 => B_DEB,
        8 => D_DEB,
        _ => G_DEB,
    };
    MotionModule::set_frame(boma, f, true);
    PREC[e] = f;
    ELAN[e] = 0.0;
    LANCE_AVANT[e] = false;
}

unsafe extern "C" fn lw_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    T[e] = 0;
    PAUVRE[e] = 0;
    physique(fighter);
    let m = if sol(boma) { "special_lw" } else { "special_air_lw" };
    MotionModule::change_motion(boma, Hash40::new(m), 0.0, 1.0, false, 0.0, false, false);
    if RELIQUE[e] > 0 && disponible(RELIQUE[e]) && (MANA[e] > 0 || RELIQUE[e] == 10) {
        MENU[e] = false;
        debut_relique(fighter, e);
    } else {
        RELIQUE[e] = 0;
        PHASE[e] = 0;
        MENU[e] = true;
        CURSEUR[e] = 10;
        PREC[e] = 0.0;
    }
    fighter.sub_shift_status_main(L2CValue::Ptr(lw_loop as *const () as _))
}

unsafe extern "C" fn lw_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    T[e] += 1;
    // sol <-> air : même image, animation de l'autre situation
    if StatusModule::is_situation_changed(boma) {
        physique(fighter);
        let f = MotionModule::frame(boma);
        if PHASE[e] == 2 && RELIQUE[e] == 10 && sol(boma) && f >= G_DEB + G_LONG_SOL {
            finir(fighter);
            return 0.into();
        }
        let m = if sol(boma) { "special_lw" } else { "special_air_lw" };
        MotionModule::change_motion_inherit_frame(boma, Hash40::new(m), -1.0, 1.0, 0.0, false, false);
    }
    let f = MotionModule::frame(boma);
    match PHASE[e] {
        0 => {
            // menu : Think en boucle, le stick pointe, B achète (ou ferme au neutre)
            if f >= M_FIN {
                MotionModule::set_frame(boma, 0.0, true);
            }
            CURSEUR[e] = curseur(boma);
            if PAUVRE[e] > 0 {
                PAUVRE[e] -= 1;
            }
            if T[e] > 2 && ControlModule::check_button_trigger(boma, *CONTROL_PAD_BUTTON_SPECIAL) {
                let c = CURSEUR[e];
                if c == 10 {
                    finir(fighter);
                    return 0.into();
                }
                let r = c + 1;
                if !disponible(r) {
                    // relique pas encore faite (étape 2)
                    PAUVRE[e] = 0;
                } else if OR[e] >= prix(c) {
                    OR[e] -= prix(c);
                    RELIQUE[e] = r;
                    MANA[e] = 100;
                    MENU[e] = false;
                    PHASE[e] = 1;
                    MotionModule::set_frame(boma, A_DEB, true);
                    effet(boma, "sys_flash", 12.0, 0.0, 0.8);
                } else {
                    PAUVRE[e] = 40;
                }
            }
        }
        1 => {
            if f >= A_FIN {
                finir(fighter);
                return 0.into();
            }
        }
        _ => {
            let prec = PREC[e];
            PREC[e] = f;
            match RELIQUE[e] {
                3 => {
                    // Morph Lance : -20 de magie à l'image 2, coup de lance à l'image 19, élan 1,6 au sol
                    let s = f - L_DEB;
                    let p = prec - L_DEB;
                    if p < 2.0 && s >= 2.0 {
                        MANA[e] -= 20;
                    }
                    let script = if sol(boma) { "game_sk_lance" } else { "game_sk_lanceair" };
                    coups(fighter, script, p, s);
                    if p < 19.0 && s >= 19.0 && sol(boma) {
                        ELAN[e] = 1.6;
                    }
                    if ELAN[e] > 0.01 {
                        let lr = PostureModule::lr(boma);
                        PostureModule::add_pos(boma, &Vector3f { x: ELAN[e] * lr, y: 0.0, z: 0.0 });
                        ELAN[e] *= 0.85;
                    }
                    if s >= L_LONG - 1.0 {
                        finir(fighter);
                        return 0.into();
                    }
                }
                7 => {
                    // Blade Charge (ShovelBurst) : toute la magie, gros coup à l'image 6
                    let s = f - B_DEB;
                    let p = prec - B_DEB;
                    if p < 1.0 && s >= 1.0 {
                        MANA[e] -= 100;
                        effet(boma, "sys_smash_flash", 8.0, 0.0, 1.2);
                    }
                    if p < 6.0 && s >= 6.0 {
                        effet(boma, "sys_attack_arc_d", 8.0, 6.0, 1.6);
                    }
                    let script = if sol(boma) { "game_sk_burst" } else { "game_sk_burstair" };
                    coups(fighter, script, p, s);
                    if s >= B_LONG - 1.0 {
                        finir(fighter);
                        return 0.into();
                    }
                }
                8 => {
                    // Healing Chalice (Drink) : toute la magie, soin de 14 % toutes les 10 images
                    let s = f - D_DEB;
                    let p = prec - D_DEB;
                    if T[e] == 1 {
                        MANA[e] -= 100;
                    }
                    for k in [10.0, 20.0, 30.0, 40.0] {
                        if p < k - 1.0 && s >= k - 1.0 {
                            let d = DamageModule::damage(boma, 0);
                            if d > 0.0 {
                                DamageModule::heal(boma, -(d.min(14.0)), 0);
                                effet(boma, "sys_recovery", 8.0, 0.0, 1.0);
                            }
                        }
                    }
                    if s >= D_LONG - 1.0 {
                        finir(fighter);
                        return 0.into();
                    }
                }
                _ => {
                    // Golden Armour : pose (ToadTaunt / AppealAirGoldArmor) avec armure 30 %
                    let s = f - G_DEB;
                    let long = if sol(boma) { G_LONG_SOL } else { G_LONG_AIR };
                    if s >= long - 1.0 {
                        finir(fighter);
                        return 0.into();
                    }
                }
            }
            if MANA[e] <= 0 && RELIQUE[e] != 10 {
                MANA[e] = 0;
            }
        }
    }
    0.into()
}

unsafe extern "C" fn lw_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    AttackModule::clear_all(boma);
    MENU[e] = false;
    PAUVRE[e] = 0;
    // magie épuisée : relique perdue (REX : 120B0100 V12000072)
    if RELIQUE[e] != 10 && MANA[e] <= 0 {
        RELIQUE[e] = 0;
    }
    sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_STOP, 0.0, 0.0);
    0.into()
}

unsafe extern "C" fn rien_status(_fighter: &mut L2CFighterCommon) -> L2CValue {
    0.into()
}

pub fn install(a: &mut Agent) {
    a.status(Init, *FIGHTER_STATUS_KIND_SPECIAL_LW, rien_status);
    a.status(Exec, *FIGHTER_STATUS_KIND_SPECIAL_LW, rien_status);
    a.status(Main, *FIGHTER_STATUS_KIND_SPECIAL_LW, lw_main);
    a.status(End, *FIGHTER_STATUS_KIND_SPECIAL_LW, lw_end);
    // bombe de Link enfant : scripts vides (le bas B est piloté ici)
    for s in ["game_speciallw", "game_specialairlw", "effect_speciallw", "effect_specialairlw",
              "sound_speciallw", "sound_specialairlw", "expression_speciallw", "expression_specialairlw"] {
        a.acmd(s, rien, Priority::Low);
    }
}
