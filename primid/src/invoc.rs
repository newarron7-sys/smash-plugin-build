//! Primid v0.37 : son bas B de REX (invocations de l'Émissaire subspatial).
//!
//! REX (FitPrimid.pac, action 0x115) : SummonStart (vitesse 2) -> SummonWait (B tenu, portail au-dessus de sa tête) ;
//! le stick choisit l'ennemi (au-delà de 0,75) -> SummonGo (interruptible à l'image 9) ; B lâché -> SummonEnd.
//! Recharges de REX : haut (Mite) 256 images, avant (Botron) 640 ; une seule invocation de chaque à la fois.
//! Cymul (arrière) et Boobas (bas) : modèles prêts, comportement à la version suivante.
//!
//! Les ennemis sont les modèles de REX mis sur des objets de Mario que Primid n'utilise pas :
//! - Mite -> v0.12 : dans le modèle de Primid (os pr_mite déplacé, 4 exemplaires face caméra) : la cape (v0.9 à v0.11)
//!   restait invisible en jeu ; mené depuis Primid (mite_pas) ;
//! - Botron -> J.E.T. (pump, statut WAIT) : vole au-dessus du terrain en portant son pot ;
//! - le pot de Botron : dans le modèle de Botron ; lâché au-dessus de l'adversaire, il est caché sur Botron et un pot
//!   (v0.9) : son os HachiN descend, il explose en feu au sol (plus d'objet à part : pumpwater plantait, Cappy refusé).
//! v0.3 : le pot était sur l'eau du J.E.T. (pumpwater) : plantage au chargement (tests A à D du joueur) ; directions
//!   corrigées d'après les icônes du portail de REX (haut Botron, avant Mite, arrière Boobas, bas Cymul).
//! Leurs statuts (Main / Exec) sont remplacés pour le costume de Primid seulement, et seulement pour les objets que le
//! bas B a créés (sinon : code d'origine, par exemple le tuyau de l'entrée en scène).

use smash::app::lua_bind::*;
use smash::app::*;
use smash::lib::lua_const::*;
use smash::lib::L2CValue;
use smash::lua2cpp::*;
use smash::phx::{Hash40, Vector2f, Vector3f};
use smash_script::macros::*;
use smash_script::*;
use smashline::*;

use crate::PR_COSTUMES;

// ------------------------------------------------------------------ réglages (REX)
// REX : haut 256 (Botron), bas 608 (Cymul), avant 640 (Mite), arrière 592 (Boobas)
const RECH_MITE: i32 = 640;
const RECH_BOTRON: i32 = 256;
const RECH_BOOBAS: i32 = 592;
const RECH_CYMUL: i32 = 608;
/// image de SummonGo à partir de laquelle il peut agir (REX : AllowInterrupt à 9)
const GO_LIBRE: f32 = 9.0;
/// B tenu au plus (images) avant qu'il range le portail tout seul
const ATTENTE_MAX: i32 = 180;

// Mite : animation unique (j00entry du tuyau) : course 0-24, élan 25-26, plongeon 27-28, attente 29-109
const MITE_COURSE: f32 = 25.0;
const MITE_ELAN: f32 = 25.0;
const MITE_PLONGE: f32 = 27.0;

static mut PH: [i32; 8] = [0; 8];
static mut T: [i32; 8] = [0; 8];
static mut PORTAIL: [u32; 8] = [0; 8];
/// recharges : 0 Mite, 1 Botron, 2 Boobas, 3 Cymul
static mut RECH: [[i32; 4]; 8] = [[0; 4]; 8];
/// v0.16 : le stick est revenu au centre pendant le bas B (sinon le bas tenu pour faire le bas B choisirait Cymul)
static mut NEUTRE: [bool; 8] = [false; 8];

/// objets créés par le bas B : 1 = à réclamer (créé à l'instant), sinon n° de l'objet
static mut MITE_ATTEND: [bool; 8] = [false; 8];
static mut MITE_ID: [u32; 8] = [0; 8];
static mut BOT_ATTEND: [bool; 8] = [false; 8];
static mut BOT_ID: [u32; 8] = [0; 8];
static mut POT_ATTEND: [bool; 8] = [false; 8];
static mut POT_ID: [u32; 8] = [0; 8];
static mut POT_LR: [f32; 8] = [1.0; 8];

// état des objets (par perso)
static mut M: [[f32; 8]; 8] = [[0.0; 8]; 8]; // x, y, vx, vy, lr, phase, temps, attente avant le prochain plongeon
static mut B: [[f32; 6]; 8] = [[0.0; 6]; 8]; // x, y, lr, temps, lâché (0/1), temps depuis le lâcher
static mut P: [[f32; 4]; 8] = [[0.0; 4]; 8]; // x, y, vy, explosé (temps)

fn e_of(boma: *mut BattleObjectModuleAccessor) -> usize {
    unsafe { (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7) }
}

unsafe fn boma_of(id: u32) -> Option<*mut BattleObjectModuleAccessor> {
    if id == 0 || id == *BATTLE_OBJECT_ID_INVALID as u32 {
        return None;
    }
    let b = sv_battle_object::module_accessor(id);
    if b.is_null() { None } else { Some(b) }
}

/// propriétaire (Primid) d'un objet : (son module, n° de perso)
unsafe fn owner(w: *mut BattleObjectModuleAccessor) -> Option<(*mut BattleObjectModuleAccessor, usize)> {
    let o = boma_of(WorkModule::get_int(w, *WEAPON_INSTANCE_WORK_ID_INT_LINK_OWNER) as u32)?;
    Some((o, e_of(o)))
}

unsafe fn id_of(weapon: &mut L2CWeaponCommon) -> u32 {
    (*weapon.battle_object).battle_object_id
}

unsafe fn pos(b: *mut BattleObjectModuleAccessor) -> (f32, f32) {
    (PostureModule::pos_x(b), PostureModule::pos_y(b))
}

/// sol sous (x, y) à moins de `bas` : sa hauteur
unsafe fn sol(w: *mut BattleObjectModuleAccessor, x: f32, y: f32, bas: f32) -> Option<f32> {
    let de = Vector2f { x, y: y + 6.0 };
    let dir = Vector2f { x: 0.0, y: -(bas + 6.0) };
    let mut hit = Vector2f { x: 0.0, y: 0.0 };
    if GroundModule::ray_check_hit_pos(w, &de, &dir, &mut hit, true) { Some(hit.y) } else { None }
}

/// adversaire le plus proche (hors propriétaire) : (dx, dy) depuis (x, y)
unsafe fn adversaire(moi: usize, x: f32, y: f32) -> Option<(f32, f32)> {
    let mut best: Option<(f32, f32)> = None;
    for n in 0..8 {
        if n == moi {
            continue;
        }
        let id = smash::app::Fighter::get_id_from_entry_id(n as i32);
        let Some(o) = boma_of(id) else { continue };
        if smash::app::utility::get_category(&mut *o) != *BATTLE_OBJECT_CATEGORY_FIGHTER {
            continue;
        }
        let st = StatusModule::status_kind(o);
        if st == *FIGHTER_STATUS_KIND_DEAD || st == *FIGHTER_STATUS_KIND_STANDBY {
            continue;
        }
        let (ox, oy) = pos(o);
        let (dx, dy) = (ox - x, oy - y);
        if best.map_or(true, |(bx, by)| dx * dx + dy * dy < bx * bx + by * by) {
            best = Some((dx, dy));
        }
    }
    best
}

/// objet libéré de Mario (contrainte de modèle puis lien, dans cet ordre : méthode de Chief), à plat, tourné
unsafe fn liberer(w: *mut BattleObjectModuleAccessor, lr: f32) {
    if LinkModule::is_model_constraint(w) {
        LinkModule::remove_model_constraint(w, true);
    }
    LinkModule::unlink_all(w);
    orienter(w, lr);
}

/// v0.5 : Mite est plat (papier, plan XY de son modèle) : tourné de 90° comme un perso, il était vu par la tranche
/// (invisible ; son coup touchait). Les invocations sont donc montrées face à la caméra : pas de rotation (côté droit)
/// ou demi-tour (côté gauche), refait à chaque image (le jeu ne les tourne plus selon lr)
unsafe fn orienter(w: *mut BattleObjectModuleAccessor, lr: f32) {
    PostureModule::set_lr(w, 1.0);
    PostureModule::set_rot(w, &Vector3f { x: 0.0, y: if lr < 0.0 { 180.0 } else { 0.0 }, z: 0.0 }, 0);
}

/// v0.4 : Mite invisible en v0.3 (son coup touchait) : modèle et visibilité forcés à chaque image
unsafe fn visible(w: *mut BattleObjectModuleAccessor) {
    ModelModule::set_visibility(w, true);
    VisibilityModule::set_whole(w, true);
}

unsafe fn placer(w: *mut BattleObjectModuleAccessor, x: f32, y: f32) {
    PostureModule::set_pos(w, &Vector3f { x, y, z: 0.0 });
}

unsafe fn eff(w: *mut BattleObjectModuleAccessor, name: &str, x: f32, y: f32, sc: f32) {
    EffectModule::req(w, Hash40::new(name), &Vector3f { x, y, z: 0.0 }, &Vector3f { x: 0.0, y: 0.0, z: 0.0 }, sc, 0, 0, false, 0);
}

unsafe fn tuer(weapon: &mut L2CWeaponCommon) {
    notify_event_msc_cmd!(weapon, Hash40::new_raw(0x199c462b5d));
}

// ------------------------------------------------------------------ bas B de Primid (statut SPECIAL_LW de Mario)

unsafe fn motion(fighter: &mut L2CFighterCommon, sol_: &str, air: &str, rate: f32, garder: bool) {
    let boma = fighter.module_accessor;
    let m = if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND { sol_ } else { air };
    if MotionModule::motion_kind(boma) == smash::hash40(m) {
        return;
    }
    if garder {
        MotionModule::change_motion_inherit_frame(boma, Hash40::new(m), -1.0, rate, 0.0, false, false);
    } else {
        MotionModule::change_motion(boma, Hash40::new(m), 0.0, rate, false, 0.0, false, false);
    }
}

unsafe fn portail(fighter: &mut L2CFighterCommon, on: bool) {
    PORTAIL[e_of(fighter.module_accessor)] = on as u32;
}

/// v0.4 : portail et icônes de REX (objets pr_* du modèle de Primid), affichés seulement pendant le bas B :
/// icône agrandie pour la direction pointée, cachée pendant sa recharge
unsafe fn portail_afficher(boma: *mut BattleObjectModuleAccessor, e: usize) {
    let on = PORTAIL[e] != 0 && StatusModule::status_kind(boma) == *FIGHTER_STATUS_KIND_SPECIAL_LW;
    let lr = PostureModule::lr(boma);
    let sx = ControlModule::get_stick_x(boma) * lr;
    let sy = ControlModule::get_stick_y(boma);
    let vis = |n: &str, v: bool| ModelModule::set_mesh_visibility(boma, Hash40::new(n), v);
    vis("pr_portail", on);
    // (nom, pointée, prête) : haut Botron, avant Mite, arrière Boobas, bas Cymul (ces deux-là : à venir)
    for (n, pointe, pret) in [("haut", sy > 0.5, RECH[e][1] <= 0), ("avant", sx > 0.5, RECH[e][0] <= 0),
                              ("arriere", sx < -0.5, RECH[e][2] <= 0), ("bas", sy < -0.5, RECH[e][3] <= 0)] {
        vis(&format!("pr_ic_{}", n), on && pret && !pointe);
        vis(&format!("pr_ic_{}_g", n), on && pret && pointe);
    }
}

unsafe extern "C" fn lw_rien(_fighter: &mut L2CFighterCommon) -> L2CValue {
    0.into()
}

unsafe extern "C" fn lw_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = e_of(boma);
    PH[e] = 0;
    T[e] = 0;
    NEUTRE[e] = false;
    // REX : SummonStart joué à vitesse 2
    motion(fighter, "special_lw_start", "special_air_lw_start", 2.0, false);
    if StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR {
        // en l'air : il descend doucement (REX : gravité réduite pendant l'invocation)
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_AIR_STOP);
    }
    fighter.sub_shift_status_main(L2CValue::Ptr(lw_loop as *const () as _))
}

unsafe fn finir(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND {
        fighter.change_status(FIGHTER_STATUS_KIND_WAIT.into(), false.into());
    } else {
        fighter.change_status(FIGHTER_STATUS_KIND_FALL.into(), false.into());
    }
}

unsafe extern "C" fn lw_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = e_of(boma);
    T[e] += 1;
    if StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR {
        // chute lente
        let vy = KineticModule::get_sum_speed_y(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN);
        if vy < -0.8 {
            KineticModule::add_speed(boma, &Vector3f { x: 0.0, y: (-0.8 - vy).min(0.2), z: 0.0 });
        }
    }
    match PH[e] {
        0 => {
            motion(fighter, "special_lw_start", "special_air_lw_start", 2.0, true);
            portail(fighter, MotionModule::frame(boma) > 20.0);
            if MotionModule::is_end(boma) {
                PH[e] = 1;
                T[e] = 0;
                MotionModule::change_motion(boma, Hash40::new(if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND { "special_lw_hold" } else { "special_air_lw_hold" }), 0.0, 1.0, false, 0.0, false, false);
            }
        }
        1 => {
            motion(fighter, "special_lw_hold", "special_air_lw_hold", 1.0, true);
            if MotionModule::is_end(boma) {
                MotionModule::set_frame(boma, 0.0, true);
            }
            portail(fighter, true);
            let lr = PostureModule::lr(boma);
            let sx = ControlModule::get_stick_x(boma) * lr;
            let sy = ControlModule::get_stick_y(boma);
            let mut go = 0;
            if sx.abs() < 0.3 && sy.abs() < 0.3 {
                NEUTRE[e] = true;
            }
            // icônes du portail de REX : haut Botron, avant Mite, arrière Boobas, bas Cymul
            if sy > 0.75 && RECH[e][1] <= 0 {
                go = 2;
            } else if sx > 0.75 && RECH[e][0] <= 0 {
                go = 1;
            } else if sx < -0.75 && RECH[e][2] <= 0 {
                go = 3;
            } else if sy < -0.75 && NEUTRE[e] && RECH[e][3] <= 0 {
                go = 4;
            }
            if go > 0 {
                match go {
                    1 => inv_creer(boma, e, INV_MITE),
                    2 => botron_creer(boma, e),
                    3 => inv_creer(boma, e, INV_BOOBAS),
                    _ => inv_creer(boma, e, INV_CYMUL),
                }
                PH[e] = 2;
                T[e] = 0;
                MotionModule::change_motion(boma, Hash40::new(if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND { "special_lw_light" } else { "special_air_lw_light" }), 0.0, 1.0, false, 0.0, false, false);
                PLAY_SE(fighter, Hash40::new("se_common_swing_09"));
            } else if !ControlModule::check_button_on(boma, *CONTROL_PAD_BUTTON_SPECIAL) || T[e] > ATTENTE_MAX {
                PH[e] = 3;
                MotionModule::change_motion(boma, Hash40::new(if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND { "special_lw_heavy" } else { "special_air_lw_heavy" }), 0.0, 1.0, false, 0.0, false, false);
            }
        }
        2 => {
            motion(fighter, "special_lw_light", "special_air_lw_light", 1.0, true);
            portail(fighter, MotionModule::frame(boma) < 20.0);
            if MotionModule::frame(boma) >= GO_LIBRE {
                // REX : il peut agir à partir de l'image 9 (l'invocation continue seule)
                if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND {
                    if fighter.sub_wait_ground_check_common(false.into()).get_bool() {
                        return 1.into();
                    }
                } else if fighter.sub_air_check_fall_common().get_bool() {
                    return 1.into();
                }
            }
            if MotionModule::is_end(boma) {
                finir(fighter);
                return 1.into();
            }
        }
        _ => {
            motion(fighter, "special_lw_heavy", "special_air_lw_heavy", 1.0, true);
            portail(fighter, false);
            if MotionModule::is_end(boma) {
                finir(fighter);
                return 1.into();
            }
        }
    }
    0.into()
}

unsafe extern "C" fn lw_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    portail(fighter, false);
    0.into()
}

/// chaque image (depuis lib.rs) : recharges
unsafe fn diag(boma: *mut BattleObjectModuleAccessor, x: f32, y: f32, ok: bool) {
    let h = EffectModule::req(boma, Hash40::new("sys_flash"), &Vector3f { x, y, z: 0.0 }, &Vector3f { x: 0.0, y: 0.0, z: 0.0 }, 1.6, 0, 0, false, 0) as u32;
    if h != 0 {
        if ok { EffectModule::set_rgb(boma, h, 0.1, 1.0, 0.1); } else { EffectModule::set_rgb(boma, h, 1.0, 0.0, 0.0); }
    }
}

/// v0.8 (diagnostic) : depuis Primid, toutes les 8 images, un éclat à la position de Mite / Botron tant que leur
/// boucle tourne (leur temps avance) : blanc pour Mite, jaune pour Botron
static mut DIAG_T: [[f32; 2]; 8] = [[0.0; 2]; 8];

// ------------------------------------------------------------------ v0.18 : armes de REX
/// coups où Primid tient son épée laser (PSA de REX : traînée d'épée sur AttackS3Hi, AttackS3Lw, AttackHi4,
/// AttackAirN, AttackAirB, ThrowF ; vidéos du joueur : attaque rapide, tilts avant, smash haut chargé)
/// v0.34 (le joueur) : pas d'épée à l'attaque rapide (REX : la plume de la tête et l'ombre du dos)
const COUPS_EPEE: [&str; 10] = ["attack_s3_hi", "attack_s3_s", "attack_s3_lw", "attack_hi4", "attack_hi4_hold",
                                 "attack_hi4_start", "attack_air_n", "attack_air_b", "throw_f", "landing_air_n"];

/// v0.33 : traînée d'épée = système de traînées du jeu (AFTER_IMAGE, comme Zero sur Roy) avec des textures que le jeu
/// charge toujours : celles des objets (effect/system/item/trail). Mario n'a pas de dossier trail : ses textures à lui
/// (v0.28, trail_c00) ne sont pas chargées (taches), et le ruban du modèle (v0.32) ne s'affichait pas. Textures de la
/// Killing Edge (tex_item_killsword1/2) remplacées par des vertes dans le mod (objet rare ; seule sa traînée change).
static mut TRAINEE_ON: [bool; 8] = [false; 8];

unsafe fn trainee(fighter: &mut L2CFighterCommon, epee: bool, gauche: bool) {
    let boma = fighter.module_accessor;
    let e = e_of(boma);
    ModelModule::set_mesh_visibility(boma, Hash40::new("pr_trail"), false);
    let actif = epee && (0..7).any(|i| AttackModule::is_attack(boma, i, false));
    if actif && !TRAINEE_ON[e] {
        let os = Hash40::new(if gauche { "havel" } else { "haver" });
        AFTER_IMAGE4_ON_arg29(fighter, Hash40::new("tex_item_killsword1"), Hash40::new("tex_item_killsword2"), 10,
                              os, 0.0, 2.4, 0.0, os, 0.0, 13.6, 0.0, true, Hash40::new("null"),
                              os, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0, *EFFECT_AXIS_Y, 0,
                              *TRAIL_BLEND_ALPHA, 101, *TRAIL_CULL_NONE, 1.3, 0.2);
        TRAINEE_ON[e] = true;
        // v0.35 (le joueur : « rajoute les sfx d'épée du jeu de base ») : son du sabre laser (objet Beam Sword) à
        // chaque coup d'épée ; grand coup pour le smash haut
        let mk = MotionModule::motion_kind(boma);
        let fort = mk == smash::hash40("attack_hi4") || mk == smash::hash40("throw_f");
        PLAY_SE(fighter, Hash40::new(if fort { "se_item_beamsword_l" } else { "se_item_beamsword_m" }));
    } else if !actif && TRAINEE_ON[e] {
        AFTER_IMAGE_OFF(fighter, 4);
        TRAINEE_ON[e] = false;
    }
}

unsafe fn armes(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let mk = MotionModule::motion_kind(boma);
    let epee = COUPS_EPEE.iter().any(|n| mk == smash::hash40(n));
    // v0.23 : tourné à gauche, le jeu échange les animations des deux bras : épée dans la main droite (copie en miroir)
    let gauche = PostureModule::lr(boma) > 0.0;
    ModelModule::set_mesh_visibility(boma, Hash40::new("pr_epee"), epee && gauche);
    ModelModule::set_mesh_visibility(boma, Hash40::new("pr_epee_r"), epee && !gauche);
    // v0.32 : vraie traînée (le joueur : « toujours des éclats bleus ») : ruban pr_trail du modèle (trail_primid.py),
    // ses 9 colonnes (pied, pointe) posées sur les positions de la lame 0 à 8 images plus tôt
    trainee(fighter, epee, gauche);
    // v0.25 : l'Auroros du haut B (REX : il attrape Primid et l'emporte ; article retiré à l'image 67)
    let haut_b = mk == smash::hash40("special_hi") || mk == smash::hash40("special_air_hi");
    let fh = MotionModule::frame(boma);
    ModelModule::set_mesh_visibility(boma, Hash40::new("pr_aurore"), haut_b && fh >= 2.0 && fh < 67.0);
    if haut_b && fh >= 2.0 && fh < 3.0 {
        // éclat blanc à son arrivée (vidéo de REX)
        let (x, y) = pos(boma);
        EffectModule::req(boma, Hash40::new("sys_smash_flash"), &Vector3f { x, y: y + 20.0, z: 0.0 },
            &Vector3f { x: 0.0, y: 0.0, z: 0.0 }, 1.2, 0, 0, false, 0);
    }
    if haut_b && fh >= 66.0 && fh < 67.0 {
        let (x, y) = pos(boma);
        EffectModule::req(boma, Hash40::new("sys_erace_smoke"), &Vector3f { x, y: y + 20.0, z: 0.0 },
            &Vector3f { x: 0.0, y: 0.0, z: 0.0 }, 1.0, 0, 0, false, 0);
    }
    // le boomerang : pour le côté B de REX (à venir)
    // smash avant de REX : il crache du feu (Firebreath). v0.34 (le joueur : « le feu sort trop tard et ne reste pas
    // assez longtemps ») : REX = 6 tours de coups toutes les 6 images (boucle du PSA, lue maintenant par gen.py) ;
    // feu dès la fin de la charge (image 20) jusqu'à la fin du dernier coup (61) + 3
    let f = MotionModule::frame(boma);
    let lr = PostureModule::lr(boma);
    let (x, y) = pos(boma);
    if [smash::hash40("attack_s4_s"), smash::hash40("attack_s4_hi"), smash::hash40("attack_s4_lw")].contains(&mk)
        && f >= 20.0 && f < 64.0 {
        if (f as i32) % 2 == 0 {
            for k in 0..3 {
                let kf = k as f32;
                EffectModule::req(boma, Hash40::new("sys_fireflower_shot"),
                    &Vector3f { x: x + lr * (6.0 + 6.5 * kf), y: y + 11.0 - 1.2 * kf, z: 0.0 },
                    &Vector3f { x: 0.0, y: 90.0 * lr, z: 0.0 }, 1.2 + 0.25 * kf, 0, 0, false, 0);
            }
        }
    }
    // v0.34 : smash bas de REX = coup de poing de feu : le poing (main gauche de REX, LHandN ; main droite quand le
    // jeu échange les bras, tourné à gauche) dans les flammes pendant la charge et jusqu'à la fin du dernier coup (26)
    let lw4 = mk == smash::hash40("attack_lw4") && f < 27.0;
    if lw4 || mk == smash::hash40("attack_lw4_hold") {
        if (f as i32) % 2 == 0 {
            let main = Hash40::new(if lr > 0.0 { "handl" } else { "handr" });
            EffectModule::req_on_joint(boma, Hash40::new("sys_fireflower_shot"), main,
                &Vector3f { x: 0.0, y: 0.0, z: 0.0 }, &Vector3f { x: 0.0, y: 0.0, z: 0.0 }, 0.75,
                &Vector3f { x: 0.0, y: 0.0, z: 0.0 }, &Vector3f { x: 0.0, y: 0.0, z: 0.0 }, false, 0, 0, 0);
        }
    }
    // v0.34 : tilt haut = poison (fumée violette de REX) pendant le coup (11 à 21 de REX) ; attaque rapide : la plume et
    // l'ombre frappent (8 à 35)
    if mk == smash::hash40("attack_hi3") && f >= 9.0 && f < 23.0 && (f as i32) % 2 == 0 {
        fumee_violette(boma, x + lr * 1.0, y + 10.0 + (f - 9.0) * 0.6, 0.7);
        fumee_violette(boma, x + lr * 2.0, y + 17.0, 0.9);
    }
    if mk == smash::hash40("attack_dash") && f >= 7.0 && f < 35.0 && (f as i32) % 3 == 0 {
        fumee_violette(boma, x - lr * 2.0, y + 9.0, 0.6);
        fumee_violette(boma, x + lr * 1.5, y + 15.0, 0.5);
    }
    // v0.34 : l'ombre du dos (Kagemushi de Brawl : grand carré noir ici, retiré du modèle) -> fumée violette qui s'en
    // échappe comme dans REX
    if (f as i32) % 9 == 0 && StatusModule::status_kind(boma) != *FIGHTER_STATUS_KIND_DEAD
        && VisibilityModule::is_visible(boma) {
        fumee_violette(boma, x - lr * 2.6, y + 8.5, 0.3);
    }
}

/// v0.35 : éclat d'ombre sur l'os qui frappe (coups normaux) : fumée violette + petit éclat violet
pub unsafe fn eclat_ombre(boma: *mut BattleObjectModuleAccessor, os: &str, x: f32, y: f32, z: f32, ech: f32) {
    let o = Hash40::new(os);
    for (nom, k) in [("sys_erace_smoke", 1.0f32), ("sys_smash_flash_s", 0.6)] {
        let h = EffectModule::req_on_joint(boma, Hash40::new(nom), o, &Vector3f { x, y, z },
            &Vector3f { x: 0.0, y: 0.0, z: 0.0 }, ech * k, &Vector3f { x: 0.0, y: 0.0, z: 0.0 },
            &Vector3f { x: 0.0, y: 0.0, z: 0.0 }, false, 0, 0, 0) as u32;
        if h != 0 {
            EffectModule::set_rgb(boma, h, 0.5, 0.08, 0.7);
        }
    }
}

/// fumée / éclat violets (poison, ombre de REX)
unsafe fn fumee_violette(boma: *mut BattleObjectModuleAccessor, x: f32, y: f32, ech: f32) {
    let h = EffectModule::req(boma, Hash40::new("sys_erace_smoke"), &Vector3f { x, y, z: 0.0 },
                              &Vector3f { x: 0.0, y: 0.0, z: 0.0 }, ech, 0, 0, false, 0) as u32;
    if h != 0 {
        EffectModule::set_rgb(boma, h, 0.45, 0.05, 0.65);
    }
}

/// v0.36 : coup du pot de Botron, porté par Primid : actif (0 non, 1 chute, 2 explosion), x, y, dégâts, taille, kbg, bkb
static mut POT: [[f32; 7]; 8] = [[0.0; 7]; 8];
static mut POT_AVANT: [f32; 8] = [0.0; 8];
const ID_POT: u64 = 5;

unsafe fn pot_frappe(fighter: &mut L2CFighterCommon, e: usize) {
    let boma = fighter.module_accessor;
    if !ArticleModule::is_exist(boma, *FIGHTER_MARIO_GENERATE_ARTICLE_PUMP) {
        POT[e][0] = 0.0;
    }
    let [on, x, y, dmg, taille, kbg, bkb] = POT[e];
    if on != POT_AVANT[e] {
        // changement (début, explosion, fin) : nouveau coup (peut retoucher)
        AttackModule::clear(boma, ID_POT as i32, false);
        POT_AVANT[e] = on;
    }
    if on < 0.5 {
        return;
    }
    let (px, py) = pos(boma);
    let lr = PostureModule::lr(boma);
    macros::ATTACK(fighter, ID_POT, 0, Hash40::new("top"), dmg, 70, kbg as i32, 0, bkb as i32, taille, 0.0, y - py,
                   (x - px) * lr, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0,
                   0, true, true, false, false, false, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL,
                   *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_fire"), *ATTACK_SOUND_LEVEL_L,
                   *COLLISION_SOUND_ATTR_BOMB, *ATTACK_REGION_OBJECT);
}

pub unsafe fn update(fighter: &mut L2CFighterCommon) {
    let e = e_of(fighter.module_accessor);
    pump_attente(fighter.module_accessor, e);
    pot_frappe(fighter, e);
    inv_pas(fighter, e);
    armes(fighter);
    portail_afficher(fighter.module_accessor, e);
    for k in 0..4 {
        if RECH[e][k] > 0 {
            RECH[e][k] -= 1;
        }
    }
}

// ------------------------------------------------------------------ Mite (cape de Mario, v0.9)
// v0.8 (diagnostic du joueur : éclat rouge) : le jeu refuse de recréer le tuyau d'entrée (dokan) pendant le match.
// La cape (mantle, côté B de Mario, créée pendant le match) le remplace ; son mouvement « special_s » porte les
// animations de Mite (d01specials.nuanmb).

unsafe fn article_boma(boma: *mut BattleObjectModuleAccessor, kind: i32) -> Option<*mut BattleObjectModuleAccessor> {
    if !ArticleModule::is_exist(boma, kind) {
        return None;
    }
    let a = ArticleModule::get_article(boma, kind);
    if a.is_null() {
        return None;
    }
    boma_of(smash::app::lua_bind::Article::get_battle_object_id(a) as u32)
}

/// ce tuyau est-il le Mite du bas B ? (réclamé à sa première image)
unsafe fn est_mite(w: *mut BattleObjectModuleAccessor, id: u32) -> Option<usize> {
    if let Some(e) = (0..8).find(|&k| MITE_ATTEND[k]) {
        MITE_ATTEND[e] = false;
        MITE_ID[e] = id;
        return Some(e);
    }
    if let Some(e) = (0..8).find(|&k| MITE_ID[k] == id && id != 0) {
        return Some(e);
    }
    let (_, e) = owner(w)?;
    if MITE_ATTEND[e] {
        MITE_ATTEND[e] = false;
        MITE_ID[e] = id;
    }
    if MITE_ID[e] == id { Some(e) } else { None }
}

unsafe extern "C" fn mite_main(weapon: &mut L2CWeaponCommon) -> L2CValue {
    let w = weapon.module_accessor;
    let Some(e) = est_mite(w, id_of(weapon)) else {
        // cape qui n'est pas Mite (côté B de Mario, pas encore remplacé) : retirée (pas de code d'origine appelé)
        tuer(weapon);
        return 0.into();
    };
    liberer(w, M[e][4]);
    placer(w, M[e][0], M[e][1]);
    visible(w);
    MotionModule::change_motion(w, Hash40::new("special_s"), 0.0, 0.0, false, 0.0, false, false);
    // v0.11 : la cape ne fait plus rien d'elle-même ; Mite est entièrement mené depuis Primid (mite_pas, comme
    // l'ascenseur de Chief, méthode validée) : v0.9 / v0.10 : sa boucle d'objet ne tournait jamais sur la cape
    weapon.fastshift(L2CValue::Ptr(mite_passif as *const () as _))
}

unsafe extern "C" fn mite_exec(weapon: &mut L2CWeaponCommon) -> L2CValue {
    let w = weapon.module_accessor;
    let _ = est_mite(w, id_of(weapon));
    0.into()
}

unsafe extern "C" fn mite_passif(_weapon: &mut L2CWeaponCommon) -> L2CValue {
    0.into()
}

/// v0.37 (le joueur : elles buguaient au demi-tour et aux prises, Botron non) : Mite, Boobas et Cymul ne sont plus dans
/// le modèle de Primid (os Throw, que le jeu tourne avec lui et prend pour tenir l'adversaire) mais dans le J.E.T., comme
/// Botron (tools/invocs_pump.py : objets pr_<nom><pose> sur l'os racine). Mario n'a qu'un J.E.T. : une seule invocation
/// à la fois en tout (choix du joueur). Leur comportement et leurs coups restent menés depuis Primid (inv_pas, coups
/// de Primid : le J.E.T. disparaît quand son propre coup touche) ; le J.E.T. ne fait que les montrer (inv_loop).
/// ce que montre le J.E.T. : 0 Botron, sinon l'invocation (INV_MITE...)
static mut PUMP_MODE: [u8; 8] = [0; 8];
/// J.E.T. à créer à l'image suivante (l'ancien est en train d'être retiré) : mode + 1, 0 = rien
static mut PUMP_ATTENTE: [u8; 8] = [0; 8];
static mut PUMP_ESSAIS: [i32; 8] = [0; 8];
/// pose à montrer : (invocation, pose, sens)
static mut VUE: [(u8, usize, f32); 8] = [(0, 0, 1.0); 8];

unsafe fn pump_generer(boma: *mut BattleObjectModuleAccessor, e: usize) {
    BOT_ATTEND[e] = true;
    BOT_ID[e] = 0;
    ArticleModule::generate_article(boma, *FIGHTER_MARIO_GENERATE_ARTICLE_PUMP, false, -1);
    if let Some(w) = article_boma(boma, *FIGHTER_MARIO_GENERATE_ARTICLE_PUMP) {
        if BOT_ATTEND[e] {
            StatusModule::change_status_force(w, *WEAPON_MARIO_PUMP_STATUS_KIND_WAIT, false);
        }
    }
    PUMP_ATTENTE[e] = 0;
}

/// nouvelle invocation sur le J.E.T. : l'ancien (Botron ou une autre invocation) est retiré d'abord
unsafe fn pump_creer(boma: *mut BattleObjectModuleAccessor, e: usize, mode: u8) {
    PUMP_MODE[e] = mode;
    POT[e][0] = 0.0;
    VUE[e] = (mode, 0, 1.0);
    if ArticleModule::is_exist(boma, *FIGHTER_MARIO_GENERATE_ARTICLE_PUMP) {
        // l'ancien n'est plus réclamé (sa boucle se retire elle-même) ; le nouveau à l'image suivante
        BOT_ATTEND[e] = false;
        BOT_ID[e] = 0;
        ArticleModule::remove_exist(boma, *FIGHTER_MARIO_GENERATE_ARTICLE_PUMP, ArticleOperationTarget(*ARTICLE_OPE_TARGET_ALL));
        PUMP_ATTENTE[e] = mode + 1;
        PUMP_ESSAIS[e] = 0;
    } else {
        pump_generer(boma, e);
    }
}

/// chaque image (Primid) : création différée
unsafe fn pump_attente(boma: *mut BattleObjectModuleAccessor, e: usize) {
    if PUMP_ATTENTE[e] == 0 {
        return;
    }
    PUMP_ESSAIS[e] += 1;
    if !ArticleModule::is_exist(boma, *FIGHTER_MARIO_GENERATE_ARTICLE_PUMP) || PUMP_ESSAIS[e] > 6 {
        if ArticleModule::is_exist(boma, *FIGHTER_MARIO_GENERATE_ARTICLE_PUMP) {
            ArticleModule::remove_exist(boma, *FIGHTER_MARIO_GENERATE_ARTICLE_PUMP, ArticleOperationTarget(*ARTICLE_OPE_TARGET_ALL));
        }
        pump_generer(boma, e);
    }
}

/// le J.E.T. de cette invocation existe-t-il (ou va-t-il être créé) ?
unsafe fn pump_la(boma: *mut BattleObjectModuleAccessor, e: usize) -> bool {
    PUMP_ATTENTE[e] != 0 || ArticleModule::is_exist(boma, *FIGHTER_MARIO_GENERATE_ARTICLE_PUMP)
}

/// objets du J.E.T. : Botron (polygon0 à 9) et les poses des invocations (pr_<nom><pose>)
/// v0.40 (le joueur : Mite n'est pas dans le bon sens) : l'objet n'est plus tourné ; deux exemplaires par pose
/// (pr_<nom><pose> : s = +1, pr_<nom><pose>r : s = -1 ; Cymul, symétrique : un seul)
unsafe fn pump_afficher(w: *mut BattleObjectModuleAccessor, botron: bool, quoi: Option<(u8, usize, f32)>) {
    for k in 0..10 {
        ModelModule::set_mesh_visibility(w, Hash40::new(&format!("polygon{}", k)), botron);
    }
    for (k, (nom, np, sym)) in INV_OBJ.iter().enumerate() {
        for p in 0..*np {
            for (suf, sens) in [("", 1.0f32), ("r", -1.0)] {
                if *sym && sens < 0.0 {
                    continue;
                }
                let on = quoi.map_or(false, |(kind, pose, s)| {
                    kind as usize == k + 1 && pose.min(np - 1) == p && (*sym || (s >= 0.0) == (sens > 0.0))
                });
                ModelModule::set_mesh_visibility(w, Hash40::new(&format!("pr_{}{}{}", nom, p, suf)), on);
            }
        }
    }
}

/// boucle du J.E.T. quand il porte Mite, Boobas ou Cymul : placé et tourné d'après Primid (inv_pas)
unsafe extern "C" fn inv_loop(weapon: &mut L2CWeaponCommon) -> L2CValue {
    let w = weapon.module_accessor;
    let Some(e) = est_botron(w, id_of(weapon)) else {
        tuer(weapon);
        return 0.into();
    };
    if INV[e] == 0 || PUMP_MODE[e] == 0 {
        BOT_ID[e] = 0;
        tuer(weapon);
        return 0.into();
    }
    WorkModule::set_int(w, 9999, *WEAPON_INSTANCE_WORK_ID_INT_LIFE);
    WorkModule::set_int(w, 9999, *WEAPON_INSTANCE_WORK_ID_INT_INIT_LIFE);
    let (kind, pose, s) = VUE[e];
    placer(w, M[e][0], M[e][1]);
    orienter(w, 1.0);
    visible(w);
    pump_afficher(w, false, Some((kind, pose, s)));
    0.into()
}

// ------------------------------------------------------------------ v0.16 : Mite, Boobas, Cymul sur l'os Throw
// v0.16 à v0.36 : dessinées dans le modèle de Primid (os Throw). v0.37 : dans le J.E.T. (voir pump_creer) ; une seule
// invocation à la fois en tout (Botron compris).

const INV_MITE: u8 = 1;
const INV_BOOBAS: u8 = 2;
const INV_CYMUL: u8 = 3;
/// invocation présente (0 = aucune)
static mut INV: [u8; 8] = [0; 8];
/// minuteries propres à chaque invocation (Boobas : vent ; Cymul : rotation)
static mut Q: [[f32; 4]; 8] = [[0.0; 4]; 8];

/// (nom des objets, nombre de poses, symétrique)
const INV_OBJ: [(&str, usize, bool); 3] = [("mite", 9, false), ("boobas", 6, false), ("cymul", 8, true)];

unsafe fn inv_creer(boma: *mut BattleObjectModuleAccessor, e: usize, kind: u8) {
    if INV[e] != 0 {
        // une seule à la fois : l'ancienne disparaît
        eff(boma, "sys_erace_smoke", M[e][0], M[e][1] + 3.0, 0.8);
        AttackModule::clear(boma, 7, false);
    }
    botron_partir(boma, e);
    let lr = PostureModule::lr(boma);
    let (x, y) = pos(boma);
    // REX : elles sortent du portail, au-dessus de sa tête
    M[e] = [x + 2.0 * lr, y + 22.0, 0.0, 0.0, lr, 0.0, 0.0, 0.0];
    Q[e] = [0.0; 4];
    INV[e] = kind;
    match kind {
        INV_MITE => RECH[e][0] = RECH_MITE,
        INV_BOOBAS => RECH[e][2] = RECH_BOOBAS,
        _ => RECH[e][3] = RECH_CYMUL,
    }
    eff(boma, "sys_erace_smoke", M[e][0], M[e][1], 0.6);
    pump_creer(boma, e, kind);
}

/// v0.37 : Botron présent quand on invoque autre chose : il part (fumée) ; son pot ne frappe plus
unsafe fn botron_partir(boma: *mut BattleObjectModuleAccessor, e: usize) {
    if PUMP_MODE[e] == 0 && ArticleModule::is_exist(boma, *FIGHTER_MARIO_GENERATE_ARTICLE_PUMP) {
        eff(boma, "sys_erace_smoke", B[e][0], B[e][1], 1.0);
    }
    POT[e][0] = 0.0;
    AttackModule::clear(boma, ID_POT as i32, false);
}

unsafe fn inv_fin(fighter: &mut L2CFighterCommon, e: usize) {
    let boma = fighter.module_accessor;
    eff(boma, "sys_erace_smoke", M[e][0], M[e][1] + 3.0, 0.8);
    AttackModule::clear(boma, 7, false);
    INV[e] = 0;
    // le J.E.T. qui la montrait est retiré (sa boucle le fait aussi d'elle-même)
    if PUMP_MODE[e] != 0 && PUMP_ATTENTE[e] == 0 && ArticleModule::is_exist(boma, *FIGHTER_MARIO_GENERATE_ARTICLE_PUMP) {
        ArticleModule::remove_exist(boma, *FIGHTER_MARIO_GENERATE_ARTICLE_PUMP, ArticleOperationTarget(*ARTICLE_OPE_TARGET_ALL));
    }
    if PUMP_MODE[e] != 0 {
        PUMP_ATTENTE[e] = 0;
        BOT_ATTEND[e] = false;
        BOT_ID[e] = 0;
    }
}

/// coup posé à l'endroit de l'invocation (coup de Primid, id 7)
unsafe fn inv_coup(fighter: &mut L2CFighterCommon, x: f32, y: f32, dmg: f32, angle: u64, kbg: i32, bkb: i32, taille: f32, dy: f32) {
    let boma = fighter.module_accessor;
    let (px, py) = pos(boma);
    let plr = PostureModule::lr(boma);
    ATTACK(fighter, 7, 0, Hash40::new("top"), dmg, angle, kbg, 0, bkb, taille, 0.0, y - py + dy, (x - px) * plr, None, None, None,
           0.8, 1.0, *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, true, true, false, false, false,
           *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
           Hash40::new("collision_attr_normal"), *ATTACK_SOUND_LEVEL_S, *COLLISION_SOUND_ATTR_KICK, *ATTACK_REGION_OBJECT);
}

unsafe fn touche(boma: *mut BattleObjectModuleAccessor) -> bool {
    AttackModule::is_infliction(boma, *COLLISION_KIND_MASK_HIT) || AttackModule::is_infliction(boma, *COLLISION_KIND_MASK_SHIELD)
}

/// adversaire le plus proche : (son module, dx, dy)
unsafe fn adversaire_obj(moi: usize, x: f32, y: f32) -> Option<(*mut BattleObjectModuleAccessor, f32, f32)> {
    let mut best: Option<(*mut BattleObjectModuleAccessor, f32, f32)> = None;
    for n in 0..8 {
        if n == moi {
            continue;
        }
        let id = smash::app::Fighter::get_id_from_entry_id(n as i32);
        let Some(o) = boma_of(id) else { continue };
        if smash::app::utility::get_category(&mut *o) != *BATTLE_OBJECT_CATEGORY_FIGHTER {
            continue;
        }
        let st = StatusModule::status_kind(o);
        if st == *FIGHTER_STATUS_KIND_DEAD || st == *FIGHTER_STATUS_KIND_STANDBY {
            continue;
        }
        let (ox, oy) = pos(o);
        let (dx, dy) = (ox - x, oy - y);
        if best.map_or(true, |(_, bx, by)| dx * dx + dy * dy < bx * bx + by * by) {
            best = Some((o, dx, dy));
        }
    }
    best
}

/// chute (gravité g), posée au sol : vrai si elle vient de toucher le sol
unsafe fn chute(boma: *mut BattleObjectModuleAccessor, x: f32, y: &mut f32, vy: &mut f32, g: f32) -> bool {
    *vy = (*vy - g).max(-3.0);
    *y += *vy;
    if *vy <= 0.0 {
        if let Some(s) = sol(boma, x, *y, (-*vy).max(1.0)) {
            *y = s;
            *vy = 0.0;
            return true;
        }
    }
    false
}

const INV_VIE: f32 = 720.0;

unsafe fn inv_pas(fighter: &mut L2CFighterCommon, e: usize) {
    let boma = fighter.module_accessor;
    if INV[e] == 0 {
        return;
    }
    M[e][6] += 1.0;
    let (x, y, t) = (M[e][0], M[e][1], M[e][6]);
    // le J.E.T. retiré par le jeu (après sa création) : l'invocation s'arrête aussi
    let perdu = t > 10.0 && !pump_la(boma, e);
    if perdu || t > INV_VIE || y < -300.0 || StatusModule::status_kind(boma) == *FIGHTER_STATUS_KIND_DEAD {
        inv_fin(fighter, e);
        return;
    }
    let (pose, s) = match INV[e] {
        INV_MITE => mite_pas(fighter, e),
        INV_BOOBAS => boobas_pas(fighter, e),
        _ => cymul_pas(fighter, e),
    };
    if INV[e] == 0 {
        return;
    }
    VUE[e] = (INV[e], pose, s);
}

// ---------------------------------------------------------------- Mite
/// v0.6 (vidéo de REX du joueur) : Mite tombe du portail, court vers l'adversaire le plus proche, fonce sur lui
/// (plongeon), rebondit après le coup et recommence, pendant une douzaine de secondes.
/// v0.16 (remarques du joueur sur la v0.14) : il courait à l'envers (son devant est le -z de son animation) et il était
/// bien trop agressif : course plus lente, 1,5 s entre deux plongeons, il s'éloigne après un coup avant de revenir.
/// v0.17 (test du joueur : après un coup, il ne faisait plus que suivre l'adversaire : collé à lui, il n'était
/// jamais à distance de plongeon) : il se tient à distance (entre 12 et 18), recule s'il est trop près, et replonge
/// dès que l'attente est finie.
/// Phases : 0 chute du portail, 1 à distance de l'adversaire, 2 plongeon, 3 rebond après un coup.
/// Poses : 0-5 course, 6 plongeon, 7 repli, 8 debout. Renvoie (pose, sens d'affichage).
unsafe fn mite_pas(fighter: &mut L2CFighterCommon, e: usize) -> (usize, f32) {
    let boma = fighter.module_accessor;
    let [mut x, mut y, mut vx, mut vy, mut lr, mut ph, t, mut att] = M[e];
    if att > 0.0 {
        att -= 1.0;
    }
    let mut court = false;
    if ph < 0.5 {
        if chute(boma, x, &mut y, &mut vy, 0.15) {
            ph = 1.0;
        }
    } else if ph < 1.5 {
        let cible = adversaire(e, x, y);
        let mut v = 0.0;
        if let Some((dx, dy)) = cible {
            let sens = if dx.abs() > 1.0 { dx.signum() } else { lr };
            if dx.abs() > 18.0 {
                // il s'approche
                lr = sens;
                v = 1.0 * sens;
            } else if dx.abs() < 12.0 && att > 0.0 {
                // trop près pendant l'attente : il recule (il court dos à l'adversaire)
                lr = -sens;
                v = -0.9 * sens;
            } else {
                lr = sens;
            }
            if att <= 0.0 && vy == 0.0 && dx.abs() < 24.0 && dx.abs() > 6.0 && dy.abs() < 14.0 {
                // plongeon rasant, comme dans la vidéo de REX (corps à l'horizontale)
                lr = sens;
                ph = 2.0;
                vx = 2.4 * lr;
                vy = 0.7;
                v = 0.0;
                AttackModule::clear(boma, 7, false);
            }
        }
        x += v;
        court = v != 0.0;
        if ph < 1.5 {
            match sol(boma, x, y, 14.0) {
                Some(g) => { y = g; vy = 0.0; }
                None => { vy = (vy - 0.15).max(-3.0); y += vy; }
            }
        }
    } else if ph < 2.5 {
        x += vx;
        vy = (vy - 0.07).max(-3.0);
        y += vy;
        inv_coup(fighter, x, y, 0.8, 50, 30, 60, 3.5, 3.0);
        // traînée rouge du plongeon (vidéo de REX), posée sur l'os Throw (= Mite) ; +Z de l'os = devant Primid
        if (t as i32) % 3 == 0 {
            // v0.17 : posé à l'endroit de Mite dans le jeu (sur l'os Throw, il restait sur Primid)
            let h = EffectModule::req(boma, Hash40::new("sys_attack_speedline"), &Vector3f { x, y: y + 4.0, z: 0.0 },
                &Vector3f { x: 0.0, y: 90.0 * lr, z: 0.0 }, 0.7, 0, 0, false, 0) as u32;
            if h != 0 {
                EffectModule::set_rgb(boma, h, 1.0, 0.15, 0.15);
            }
        }
        if touche(boma) {
            AttackModule::clear(boma, 7, false);
            ph = 3.0;
            vx = -0.8 * lr;
            vy = 1.6;
            att = 90.0;
        } else if vy < 0.0 {
            if let Some(g) = sol(boma, x, y, (-vy).max(1.0)) {
                AttackModule::clear(boma, 7, false);
                y = g;
                vy = 0.0;
                ph = 1.0;
                att = 60.0;
            }
        }
    } else {
        // rebond après un coup
        x += vx;
        if chute(boma, x, &mut y, &mut vy, 0.14) {
            ph = 1.0;
        }
    }
    M[e] = [x, y, vx, vy, lr, ph, t, att];
    let pose = if ph < 1.5 && ph > 0.5 && vy == 0.0 {
        if court { ((t as i32 / 5) % 6) as usize } else { 8 }
    } else if ph > 1.5 && ph < 2.5 {
        6
    } else {
        7
    };
    (pose, -lr)
}

// ---------------------------------------------------------------- Boobas
/// REX (ItmPrimid01) : animations Jump, Fall, Landing, WindStart, Wind, WindEnd ; ennemi de l'Émissaire subspatial
/// (SmashWiki) : il sautille ou attend, et souffle du vent quand un perso s'approche (environ trois largeurs de Donkey
/// Kong), qui le pousse ; toucher sa tête quand il commence à souffler fait mal.
/// Phases : 0 chute du portail, 1 sautille vers l'adversaire, 2 début du vent, 3 vent, 4 fin du vent.
/// Poses : 0 saut, 1 chute, 2 atterrissage, 3 début du vent, 4-5 vent.
const BOOBAS_PORTEE: f32 = 40.0;

unsafe fn boobas_pas(fighter: &mut L2CFighterCommon, e: usize) -> (usize, f32) {
    let boma = fighter.module_accessor;
    let [mut x, mut y, mut vx, mut vy, mut lr, mut ph, t, mut att] = M[e];
    // Q : 0 temps dans la phase, 1 images d'atterrissage restantes
    Q[e][0] += 1.0;
    let tp = Q[e][0];
    if att > 0.0 {
        att -= 1.0;
    }
    let cible = adversaire_obj(e, x, y);
    let mut pose = 2;
    if ph < 0.5 {
        pose = 1;
        if chute(boma, x, &mut y, &mut vy, 0.12) {
            ph = 1.0;
            Q[e] = [0.0, 10.0, 0.0, 0.0];
        }
    } else if ph < 1.5 {
        // sautille vers l'adversaire, souffle quand il est assez près (devant lui)
        let au_sol = vy == 0.0 && sol(boma, x, y, 2.0).is_some();
        if au_sol {
            vx = 0.0;
            if Q[e][1] > 0.0 {
                Q[e][1] -= 1.0;
                pose = 2;
            } else {
                pose = 2;
                if let Some((_, dx, dy)) = cible {
                    if dx.abs() > 2.0 {
                        lr = dx.signum();
                    }
                    if att <= 0.0 && dx.abs() < BOOBAS_PORTEE && dy.abs() < 16.0 {
                        ph = 2.0;
                        Q[e][0] = 0.0;
                    } else if tp > 40.0 {
                        // petit saut vers lui
                        vx = 0.9 * lr;
                        vy = 2.0;
                        Q[e][0] = 0.0;
                    }
                }
            }
        }
        if ph < 1.5 && (!au_sol || vy > 0.0) {
            x += vx;
            pose = if vy > 0.0 { 0 } else { 1 };
            if chute(boma, x, &mut y, &mut vy, 0.12) {
                Q[e][1] = 8.0;
                vx = 0.0;
            }
        }
    } else if ph < 2.5 {
        // début du vent : coup de tête (trompette) au moment où il souffle
        pose = 3;
        if tp > 8.0 && tp < 14.0 {
            inv_coup(fighter, x + 7.0 * lr, y, 6.0, 45, 60, 35, 4.0, 11.0);
        }
        if tp == 14.0 {
            AttackModule::clear(boma, 7, false);
        }
        if tp > 20.0 {
            ph = 3.0;
            Q[e][0] = 0.0;
        }
    } else if ph < 3.5 {
        // vent : pousse l'adversaire devant lui (plus fort de près)
        pose = 4 + ((t as i32 / 6) % 2) as usize;
        if let Some((o, dx, dy)) = cible {
            if dx * lr > 0.0 && dx.abs() < BOOBAS_PORTEE + 10.0 && dy.abs() < 18.0 {
                let f = 0.35 + 0.55 * (1.0 - dx.abs() / (BOOBAS_PORTEE + 10.0));
                PostureModule::add_pos(o, &Vector3f { x: f * lr, y: 0.0, z: 0.0 });
            }
        }
        if (t as i32) % 6 == 0 {
            // souffle (traînées blanches devant sa trompette) ; +Z de l'os Throw = devant Primid
            EffectModule::req(boma, Hash40::new("sys_attack_speedline"), &Vector3f { x: x + 12.0 * lr, y: y + 10.0, z: 0.0 },
                &Vector3f { x: 0.0, y: 90.0 * lr, z: 0.0 }, 0.8, 0, 0, false, 0);
        }
        if tp > 120.0 {
            ph = 4.0;
            Q[e][0] = 0.0;
        }
    } else {
        pose = 3;
        if tp > 20.0 {
            ph = 1.0;
            Q[e][0] = 0.0;
            att = 90.0;
        }
    }
    M[e] = [x, y, vx, vy, lr, ph, t, att];
    // v0.17 : le joueur : il apparaissait dans le mauvais sens
    (pose, -lr)
}

// ---------------------------------------------------------------- Cymul
/// REX (ItmPrimid00) : animations Start, Float, ItmBumper ; ennemi de l'Émissaire subspatial (SmashWiki) : il se
/// déplace en volant ; quand un perso s'approche, il sort ses réflecteurs et tourne : petits coups répétés (toutes les
/// 8 images, ils le gardent près de lui), puis un dernier coup qui l'envoie vers le haut.
/// v0.19 (vidéo de REX du joueur) : il déploie ses lames (animation ItmBumper : réflecteurs sortis, rotation de 18° par
/// image) et tourne ; plus gros (x1,2) ; il passait sous le sol (son origine est en son centre).
/// Phases : 0 sortie du portail, 1 vole vers l'adversaire, 2 tourne, 3 repos.
/// Poses : 0-1 vol, 2 lames sorties, 3-7 rotation (un quart de tour, les 4 lames se répètent).
const CYMUL_BAS: f32 = 10.5;
/// vitesse vers (ax, ay), lissée (il flotte)
fn vise(x: f32, y: f32, vx: &mut f32, vy: &mut f32, ax: f32, ay: f32, v: f32) {
    let (dx, dy) = (ax - x, ay - y);
    let d = (dx * dx + dy * dy).sqrt().max(0.001);
    let k = (v / d).min(1.0);
    *vx = *vx * 0.85 + dx * k * 0.15;
    *vy = *vy * 0.85 + dy * k * 0.15;
}

unsafe fn cymul_pas(fighter: &mut L2CFighterCommon, e: usize) -> (usize, f32) {
    let boma = fighter.module_accessor;
    let [mut x, mut y, mut vx, mut vy, lr, mut ph, t, mut att] = M[e];
    Q[e][0] += 1.0;
    let tp = Q[e][0];
    if att > 0.0 {
        att -= 1.0;
    }
    let cible = adversaire_obj(e, x, y);
    let mut pose = if ((t as i32 / 30) % 2) == 0 { 0 } else { 1 };
    if ph < 0.5 {
        // sort du portail et descend un peu
        vise(x, y, &mut vx, &mut vy, x, y - 8.0, 0.6);
        if tp > 25.0 {
            ph = 1.0;
            Q[e][0] = 0.0;
        }
    } else if ph < 1.5 || ph > 2.5 {
        if let Some((_, dx, dy)) = cible {
            let (ax, ay) = (x + dx, y + dy + 9.0);
            vise(x, y, &mut vx, &mut vy, ax, ay, 0.75);
            if ph < 1.5 && att <= 0.0 && dx.abs() < 18.0 && (dy + 9.0).abs() < 14.0 {
                ph = 2.0;
                Q[e][0] = 0.0;
                vx = 0.0;
                vy = 0.0;
                AttackModule::clear(boma, 7, false);
            }
        } else {
            vise(x, y, &mut vx, &mut vy, x, y, 0.0);
        }
        if ph > 2.5 && tp > 70.0 {
            ph = 1.0;
            Q[e][0] = 0.0;
        }
    } else {
        // rotation, lames sorties : coups répétés puis coup final vers le haut
        vx *= 0.8;
        vy *= 0.8;
        let ti = tp as i32;
        pose = if ti < 4 || ti > 56 { 2 } else { 3 + (ti % 5) as usize };
        // v0.39 (vidéo de REX du joueur : « rajoute l'effet quand il tourne ») : traînées blanches autour de lui,
        // dans le plan des lames (à plat, couché vers la caméra), qui font le tour
        if ti >= 4 && ti <= 56 && ti % 3 == 0 {
            for k in 0..2 {
                let tour = ((ti * 50 + k * 180) % 360) as f32;
                let h = EffectModule::req(boma, Hash40::new("sys_attack_arc"), &Vector3f { x, y: y - 1.0, z: 0.0 },
                    &Vector3f { x: 75.0, y: tour, z: 0.0 }, 1.25, 0, 0, false, 0) as u32;
                if h != 0 {
                    EffectModule::set_rgb(boma, h, 1.0, 1.0, 1.0);
                }
            }
        }
        if ti >= 4 && ti < 50 {
            if ti % 8 == 4 {
                AttackModule::clear(boma, 7, false);
                inv_coup(fighter, x, y, 1.2, 80, 20, 25, 11.0, 0.0);
            } else if ti % 8 == 7 {
                AttackModule::clear(boma, 7, false);
            }
        } else if ti == 52 {
            AttackModule::clear(boma, 7, false);
            inv_coup(fighter, x, y, 2.5, 85, 110, 55, 12.0, 0.0);
        } else if ti == 56 {
            AttackModule::clear(boma, 7, false);
        } else if ti > 60 {
            ph = 3.0;
            Q[e][0] = 0.0;
            att = 60.0;
        }
    }
    x += vx;
    y += vy;
    // jamais dans le sol (son origine est en son centre)
    if let Some(g) = sol(boma, x, y, CYMUL_BAS + 4.0) {
        if y < g + CYMUL_BAS {
            y = g + CYMUL_BAS;
            vy = vy.max(0.0);
        }
    }
    M[e] = [x, y, vx, vy, lr, ph, t, att];
    (pose, 1.0)
}

// ------------------------------------------------------------------ Botron (J.E.T. de Mario) et son pot (eau du J.E.T.)

unsafe fn botron_creer(boma: *mut BattleObjectModuleAccessor, e: usize) {
    if INV[e] != 0 {
        // v0.37 : une seule invocation à la fois : celle qui est là disparaît
        eff(boma, "sys_erace_smoke", M[e][0], M[e][1] + 3.0, 0.8);
        AttackModule::clear(boma, 7, false);
        INV[e] = 0;
    }
    let lr = PostureModule::lr(boma);
    let (x, y) = pos(boma);
    B[e] = [x + 6.0 * lr, y + 20.0, lr, 0.0, 0.0, 0.0];
    pump_creer(boma, e, 0);
    RECH[e][1] = RECH_BOTRON;
}

unsafe fn est_botron(_w: *mut BattleObjectModuleAccessor, id: u32) -> Option<usize> {
    if let Some(e) = (0..8).find(|&k| BOT_ATTEND[k]) {
        BOT_ATTEND[e] = false;
        BOT_ID[e] = id;
        return Some(e);
    }
    let e = (0..8).find(|&k| BOT_ID[k] == id && id != 0)?;
    if BOT_ATTEND[e] {
        BOT_ATTEND[e] = false;
        BOT_ID[e] = id;
    }
    if BOT_ID[e] == id { Some(e) } else { None }
}

unsafe extern "C" fn pump_main(weapon: &mut L2CWeaponCommon) -> L2CValue {
    let w = weapon.module_accessor;
    let Some(e) = est_botron(w, id_of(weapon)) else {
        // J.E.T. qui n'est pas Botron (Primid n'en a pas d'autre) : retiré
        tuer(weapon);
        return 0.into();
    };
    P[e] = [0.0, 0.0, 0.0, 0.0];
    POT[e][0] = 0.0;
    BOT_ATT[e] = 0.0;
    EffectModule::kill_all(w, 0, true, false);
    if PUMP_MODE[e] != 0 {
        // v0.37 : Mite, Boobas ou Cymul
        if INV[e] == 0 {
            BOT_ID[e] = 0;
            tuer(weapon);
            return 0.into();
        }
        liberer(w, 1.0);
        placer(w, M[e][0], M[e][1]);
        visible(w);
        MotionModule::change_motion(w, Hash40::new("wait"), 0.0, 0.0, false, 0.0, false, false);
        pump_afficher(w, false, Some(VUE[e]));
        return weapon.fastshift(L2CValue::Ptr(inv_loop as *const () as _));
    }
    pump_afficher(w, true, None);
    liberer(w, B[e][2]);
    placer(w, B[e][0], B[e][1]);
    visible(w);
    MotionModule::change_motion(w, Hash40::new("start"), 0.0, 1.0, false, 0.0, false, false);
    EffectModule::kill_all(w, 0, true, false);
    pot_porte(w, true);
    weapon.fastshift(L2CValue::Ptr(botron_loop as *const () as _))
}

unsafe extern "C" fn pump_touche(_w: &mut L2CWeaponCommon, _a: &L2CValue, _b: &L2CValue) -> L2CValue {
    0.into()
}

unsafe extern "C" fn pump_degats(_w: &mut L2CWeaponCommon, _a: &L2CValue) -> L2CValue {
    0.into()
}

unsafe extern "C" fn pump_rien(_w: &mut L2CWeaponCommon) -> L2CValue {
    0.into()
}

unsafe extern "C" fn pump_exec(weapon: &mut L2CWeaponCommon) -> L2CValue {
    let w = weapon.module_accessor;
    let _ = est_botron(w, id_of(weapon));
    0.into()
}

/// v0.9 : le pot reste dans le modèle de Botron ; lâché, son os (HachiN, sous Hip à -6) descend (set_joint_translate)
/// et frappe depuis cet os ; Botron reste sur place jusqu'à l'explosion. (v0.3 : pot sur l'eau du J.E.T. = plantage ;
/// v0.4 : sur Cappy, que le jeu ne crée sans doute pas non plus pendant le match.)
#[allow(dead_code)]
unsafe fn pot_coup(weapon: &mut L2CWeaponCommon, dmg: f32, taille: f32, kbg: i32, bkb: i32) {
    ATTACK(weapon, 1, 0, Hash40::new("hachin"), dmg, 70, kbg, 0, bkb, taille, 0.0, -4.0, 0.0, None, None, None, 1.0, 1.0,
           *ATTACK_SETOFF_KIND_ON, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, true, true, false, false, false,
           *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
           Hash40::new("collision_attr_fire"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_BOMB, *ATTACK_REGION_OBJECT);
    // v0.35f (le joueur : il disparaît toujours pile quand le pot touche) : un coup de type BOMBE fait « exploser »
    // l'objet qui le porte (le jeu retire le projectile, comme une bombe) : Botron disparaissait avec. Type OBJET.
}

/// v0.38 (le joueur : « comme Mite, après une attaque il attend un peu avant de réattaquer ») : images d'attente de
/// Botron après chaque pot (il se tient à distance de l'adversaire avant de revenir au-dessus de lui)
static mut BOT_ATT: [f32; 8] = [0.0; 8];
const BOT_PAUSE: f32 = 90.0;

unsafe fn pot_os(w: *mut BattleObjectModuleAccessor, chute: f32) {
    ModelModule::set_joint_translate(w, Hash40::new("hachin"), &Vector3f { x: 0.0, y: -6.0 - chute, z: 0.0 }, false, false);
}

unsafe extern "C" fn botron_loop(weapon: &mut L2CWeaponCommon) -> L2CValue {
    let w = weapon.module_accessor;
    let Some(e) = est_botron(w, id_of(weapon)) else {
        tuer(weapon);
        return 0.into();
    };
    if PUMP_MODE[e] != 0 {
        // v0.37 : une autre invocation a pris le J.E.T.
        tuer(weapon);
        return 0.into();
    }
    let [mut x, mut y, lr, mut t, mut lache, mut tl] = B[e];
    // v0.35 (le joueur : il disparaît juste après le 1er pot) : le J.E.T. a une durée de vie au bout de laquelle le jeu
    // le retire : remise à plein à chaque image (c'est la boucle qui décide quand il part)
    WorkModule::set_int(w, 9999, *WEAPON_INSTANCE_WORK_ID_INT_LIFE);
    WorkModule::set_int(w, 9999, *WEAPON_INSTANCE_WORK_ID_INT_INIT_LIFE);
    // pot : chute (sous Botron), vitesse, explosion (temps)
    // v0.34 (le joueur : « il jette plusieurs fois des projectiles et après il s'en va ») : 3 pots, puis il part
    let [mut chute, mut vy, mut boum, mut lances] = P[e];
    t += 1.0;
    let mk = MotionModule::motion_kind(w);
    if lache < 0.5 {
        // vol (REX : StartFloat puis FloatFly) vers l'avant, à hauteur constante au-dessus du sol
        let attend = BOT_ATT[e] > 0.0;
        if attend {
            // v0.38 : pause après un pot : il s'écarte (au moins 20 de l'adversaire) et plane un peu plus haut
            BOT_ATT[e] -= 1.0;
            t = 0.0;
            if let Some((dx, _)) = adversaire(e, x, y) {
                if dx.abs() < 20.0 {
                    let sens = if dx.abs() > 0.5 { -dx.signum() } else { -lr };
                    x += 0.8 * sens;
                }
            }
        } else {
            match adversaire(e, x, y) {
                // pots suivants : il se place au-dessus de l'adversaire
                Some((dx, _)) if lances > 0.5 => x += (dx * 0.08).clamp(-1.3, 1.3),
                _ => x += 1.1 * lr,
            }
        }
        if let Some(g) = sol(w, x, y, 80.0) {
            // v0.10 : juste au-dessus des têtes (v0.9 : tout en haut de l'écran)
            let cible = g + if attend { 30.0 } else { 22.0 };
            y += (cible - y).clamp(-0.6, 0.6);
        }
        if MotionModule::is_end(w) || (mk != smash::hash40("start") && mk != smash::hash40("wait")) {
            MotionModule::change_motion(w, Hash40::new("wait"), 0.0, 1.0, false, 0.0, false, false);
        }
        let au_dessus = adversaire(e, x, y).map_or(false, |(dx, dy)| dx.abs() < 7.0 && dy < 0.0 && dy > -70.0);
        if !attend && (au_dessus || t > (if lances > 0.5 { 70.0 } else { 150.0 })) {
            // il lâche son pot (REX : ShootStart + ShootFire)
            lache = 1.0;
            tl = 0.0;
            chute = 0.0;
            vy = 0.0;
            boum = 0.0;
            MotionModule::change_motion(w, Hash40::new("light"), 0.0, 1.0, false, 0.0, false, false);
        }
        pot_os(w, 0.0);
    } else {
        tl += 1.0;
        if mk == smash::hash40("light") && MotionModule::is_end(w) {
            MotionModule::change_motion(w, Hash40::new("wait"), 0.0, 1.0, false, 0.0, false, false);
        }
        if boum <= 0.0 {
            // le pot tombe (Botron reste sur place) et frappe en tombant
            vy = (vy + 0.22).min(4.5);
            chute += vy;
            pot_os(w, chute);
            let py = y - 6.0 - chute - 8.0;
            // v0.36 : le coup du pot est porté par Primid (le J.E.T. disparaissait dès que son propre coup touchait)
            POT[e] = [1.0, x, py + 4.0, 6.0, 5.0, 60.0, 40.0];
            let touche = adversaire(e, x, py + 4.0).map_or(false, |(dx, dy)| dx.abs() < 8.0 && dy > -24.0 && dy < 3.0);
            if touche || sol(w, x, py + 4.0, vy.max(1.0)).is_some() || chute > 150.0 {
                POT[e] = [2.0, x, py + 4.0, 13.0, 13.0, 80.0, 55.0];
                eff(w, "sys_bomb_a", x, py + 4.0, 0.9);
                eff(w, "sys_fireflower_shot", x, py + 6.0, 1.5);
                PLAY_SE(weapon, Hash40::new("se_common_bomb_l"));
                // le pot disparaît dans l'explosion
                pot_porte(w, false);
                boum = 1.0;
            }
        } else {
            boum += 1.0;
            pot_os(w, chute);
            if boum > 6.0 {
                POT[e][0] = 0.0;
            }
            if boum > 24.0 && lances < 2.0 {
                // pot suivant : il en reprend un et repart vers l'adversaire
                lances += 1.0;
                lache = 0.0;
                t = 0.0;
                BOT_ATT[e] = BOT_PAUSE;
                boum = 0.0;
                chute = 0.0;
                vy = 0.0;
                pot_porte(w, true);
                pot_os(w, 0.0);
                MotionModule::change_motion(w, Hash40::new("wait"), 0.0, 1.0, false, 0.0, false, false);
            } else if boum > 30.0 {
                // il remonte et s'en va (REX : ShootEnd)
                y += 1.2;
                x += 0.6 * lr;
            }
            if boum > 90.0 {
                eff(w, "sys_erace_smoke", x, y, 1.0);
                BOT_ID[e] = 0;
                tuer(weapon);
                return 0.into();
            }
        }
    }
    B[e] = [x, y, lr, t, lache, tl];
    P[e] = [chute, vy, boum, lances];
    placer(w, x, y);
    orienter(w, lr);
    visible(w);
    pump_afficher(w, true, None);
    pot_porte(w, lache < 0.5 || boum <= 0.0);
    0.into()
}

/// objets du pot dans le modèle de Botron (HachiM, FireM : polygon6 à 9)
unsafe fn pot_porte(w: *mut BattleObjectModuleAccessor, on: bool) {
    for m in ["polygon6", "polygon7", "polygon8", "polygon9"] {
        ModelModule::set_mesh_visibility(w, Hash40::new(m), on);
    }
}

unsafe extern "C" fn rien(_agent: &mut L2CAgentBase) {}

pub fn install(a: &mut Agent) {
    a.status(Main, *FIGHTER_STATUS_KIND_SPECIAL_LW, lw_main);
    a.status(Init, *FIGHTER_STATUS_KIND_SPECIAL_LW, lw_rien);
    a.status(Exec, *FIGHTER_STATUS_KIND_SPECIAL_LW, lw_rien);
    a.status(End, *FIGHTER_STATUS_KIND_SPECIAL_LW, lw_end);
    // scripts du J.E.T. de Mario sur ses animations : retirés (effets d'eau, son, voix)
    for m in ["speciallwstart", "specialairlwstart", "speciallwhold", "specialairlwhold", "speciallwlight",
              "specialairlwlight", "speciallwheavy", "specialairlwheavy"] {
        for k in ["game", "effect", "sound", "expression"] {
            a.acmd(&format!("{}_{}", k, m), rien, Priority::Low);
        }
    }

    // v0.9 : Mite = cape (DUMMY et TERM : le jeu peut la faire passer de l'un à l'autre)
    let mut d = Agent::new("mario_mantle");
    d.set_costume(PR_COSTUMES.to_vec());
    for st in [*WEAPON_MARIO_MANTLE_STATUS_KIND_DUMMY, *WEAPON_MARIO_MANTLE_STATUS_KIND_TERM] {
        d.status(Main, st, mite_main);
        d.status(Exec, st, mite_exec);
    }
    d.install();

    // v0.9 : Botron = J.E.T. sur tous ses statuts (v0.8 : créé mais sa boucle ne tournait pas : sans doute passé à un
    // autre statut par son propre code)
    let mut p = Agent::new("mario_pump");
    p.set_costume(PR_COSTUMES.to_vec());
    for st in [*WEAPON_MARIO_PUMP_STATUS_KIND_WAIT, *WEAPON_MARIO_PUMP_STATUS_KIND_SHOOT, *WEAPON_MARIO_PUMP_STATUS_KIND_END,
               *WEAPON_MARIO_PUMP_STATUS_KIND_TERM] {
        p.status(Main, st, pump_main);
        p.status(Exec, st, pump_exec);
        // v0.35d (le joueur : il disparaît toujours juste après le 1er pot ; la durée de vie n'y était pour rien) :
        // il disparaît quand son pot TOUCHE : le code du J.E.T. de Mario retire l'objet quand son coup touche
        // (CheckAttack) ; retiré, ainsi que ses autres fonctions de statut (fin, sortie, dégâts)
        p.status(CheckAttack, st, pump_touche);
        p.status(CheckDamage, st, pump_degats);
        p.status(End, st, pump_rien);
        p.status(Exit, st, pump_rien);
        p.status(Init, st, pump_rien);
    }
    for m in ["hold", "light", "heavy", "start", "end", "wait"] {
        for k in ["game", "effect", "sound"] {
            p.acmd(&format!("{}_{}", k, m), rien, Priority::Low);
        }
    }
    p.install();
}
