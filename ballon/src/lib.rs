#![feature(proc_macro_hygiene)]
#![allow(unused, non_snake_case, static_mut_refs, unused_unsafe, unused_must_use)]

//! Hammer Express (portage du stage Brawl « Lethal League », 2e variante du Petit Champ de bataille) : le mode
//! Lethal League recréé d'après la vidéo du mod Brawl :
//! - la balle (Ballon de foot d'Ultimate) attend immobile en l'air au centre ;
//! - frappée par une attaque, elle part tout droit, sans gravité ni frottement, et rebondit sur le sol, les murs
//!   et le plafond (±75, 0, 81) sans jamais s'arrêter ; chaque coup la fait accélérer (et la fige un instant) ;
//! - si elle touche un combattant qui ne la frappe pas (n'importe lequel, comme dans le mod Brawl : pas de
//!   couleur ni de protection pour celui qui l'a frappée), il est éliminé d'un coup (sauf bouclier : elle
//!   rebondit) ; la balle revient alors au centre.
//! Seulement sur Hammer Express : Petit Champ de bataille dont les murs invisibles sont à x ±75.
//! - v0.3 (mécaniques du vrai jeu) : coups normaux = droit,
//!   stick haut = vers le haut, stick bas = vers le bas ; smash (coup chargé) = beaucoup plus rapide et arrêt sur
//!   image plus court ; amorti (jab) = la balle monte doucement puis retombe (gravité), garde sa vitesse et la
//!   prochaine frappe est plus forte ; pendant l'amorti elle n'élimine personne.
//! - v0.5 (comme le mod Brawl) : le combattant touché est envoyé juste derrière la zone de K.O. dans la direction
//!   de la balle : c'est le vrai K.O. du jeu (rayon, son) qui se joue. Pas d'effet ajouté (ni rayon, ni traînée).
//! - v0.6 : balle du mod Brawl (modèle + texture, dans item/soccerball) et 2× plus grosse (comme dans Brawl).
//! - v0.10 : le joueur 1 (identifiant 0) était ignoré : il ne touchait jamais la balle et n'était jamais éliminé.
//! - v0.9 : la balle ne bougeait pas quand on la tapait -> détection des coups par le plugin (hitbox active + zone devant le combattant).
//! - v0.8 : crash (récursion infinie : changer le statut du combattant relançait ballon_frame) -> verrou anti ré-entrée.
//! - v0.7 : correctif : le combattant qui « donne » la balle restait bloqué (objet lourd tenu) -> éjection + debout.
//! v0.1 : balle d'Ultimate normale au centre ; v0.2 : comportement Lethal League

use smash::app::lua_bind::*;
use smash::app::*;
use smash::lib::lua_const::*;
use smash::lua2cpp::*;
use smash::phx::{Vector2f, Vector3f};
use smashline::*;

const DELAI: i32 = 90; // avant de (re)faire apparaître la balle
const CENTRE: (f32, f32) = (0.0, 45.0);
/// la balle de Brawl est beaucoup plus grosse que celle d'Ultimate : l'objet est agrandi (modèle ET zones de coup)
const TAILLE: f32 = 2.0;
const R: f32 = 2.75 * TAILLE + 0.3; // rayon de la balle (modèle 5,5 de diamètre × TAILLE)
const MUR: f32 = 75.0;
const SOL: f32 = 0.0;
const PLAFOND: f32 = 81.4; // plafond de la boîte de Brawl (pour la balle seulement)
const V0: f32 = 1.6; // vitesse après le premier coup (unités / image)
const ACCEL: f32 = 1.22; // multiplicateur à chaque coup
const VMAX: f32 = 11.0;
const CORPS: f32 = 7.5; // rayon de contact d'un combattant (autour du milieu du corps)

static mut EST_HE: i32 = -1;
static mut ATTENTE: i32 = DELAI;
static mut CHEF: i32 = -1;
static mut SANS_CHEF: i32 = 0;

static mut BALLE: u32 = 0;
static mut POS: (f32, f32) = CENTRE;
static mut DIR: (f32, f32) = (0.0, 0.0);
static mut VIT: f32 = 0.0;
static mut PROPRIO: i32 = -1;
static mut FIGE: i32 = 0;
static mut GRACE: i32 = 0; // images sans contact mortel après un coup / un rebond sur bouclier
/// zone de K.O. du Petit Champ de bataille (lvd)
const KO_G: f32 = -240.0;
const KO_D: f32 = 240.0;
const KO_H: f32 = 180.0;
const KO_B: f32 = -140.0;

/// envoie le combattant juste derrière la zone de K.O., dans la direction de la balle : le jeu fait le vrai K.O.
unsafe fn ejecter(o: *mut BattleObjectModuleAccessor, x: f32, y: f32, dx: f32, dy: f32) {
    let mut t = 1.0e9f32;
    if dx > 0.001 { t = t.min((KO_D - x) / dx); }
    if dx < -0.001 { t = t.min((KO_G - x) / dx); }
    if dy > 0.001 { t = t.min((KO_H - y) / dy); }
    if dy < -0.001 { t = t.min((KO_B - y) / dy); }
    if t > 1.0e8 { t = 0.0; }
    let (px, py) = (x + dx * (t + 15.0), y + dy * (t + 15.0));
    StatusModule::change_status_force(o, *FIGHTER_STATUS_KIND_FALL, false);
    GroundModule::correct(o, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
    PostureModule::set_pos(o, &Vector3f { x: px, y: py, z: 0.0 });
    KineticModule::clear_speed_all(o);
}

static mut AMORTI: bool = false; // amorti : trajectoire en cloche, pas mortelle
static mut VY_AMORTI: f32 = 0.0;
static mut DEJA: [bool; 8] = [false; 8]; // ce combattant a déjà renvoyé la balle avec ce coup
const PORTEE: f32 = 16.0; // portée d'un coup autour du point devant le combattant
static mut EN_COURS: bool = false; // v0.8 : anti ré-entrée (changer le statut relance la boucle du combattant -> récursion infinie)
static mut BONUS: f32 = 1.0; // frappe après un amorti : plus forte

/// comment le combattant a frappé : (direction, multiplicateur de vitesse, arrêt sur image ×, amorti)
unsafe fn frappe(o: *mut BattleObjectModuleAccessor, d: (f32, f32)) -> ((f32, f32), f32, f32, bool) {
    let st = StatusModule::status_kind(o);
    let lr = PostureModule::lr(o);
    let sy = ControlModule::get_stick_y(o);
    let air = StatusModule::situation_kind(o) == *SITUATION_KIND_AIR;
    if st == *FIGHTER_STATUS_KIND_ATTACK {
        return ((0.0, 1.0), 1.0, 0.5, true); // jab = amorti
    }
    let smash = st == *FIGHTER_STATUS_KIND_ATTACK_S4 || st == *FIGHTER_STATUS_KIND_ATTACK_HI4 || st == *FIGHTER_STATUS_KIND_ATTACK_LW4
        || (air && sy < -0.5);
    let (mut ax, mut ay) = (lr, 0.0f32);
    if sy > 0.5 { ay = 1.0; } else if sy < -0.5 { ay = -1.0; }
    if st == *FIGHTER_STATUS_KIND_ATTACK_HI4 || st == *FIGHTER_STATUS_KIND_ATTACK_HI3 { ay = 1.0; }
    let n = (ax * ax + ay * ay).sqrt();
    let dir = if n > 0.0 { (ax / n, ay / n) } else { d };
    if smash { (dir, 1.7, 0.5, false) } else { (dir, 1.0, 1.0, false) }
}

unsafe fn boma_de(id: u32) -> Option<*mut BattleObjectModuleAccessor> {
    // v0.10 : l'identifiant 0 est valide (c'est le joueur 1 !) : avant il était ignoré -> P1 ne frappait jamais
    // la balle et ne pouvait pas être éliminé
    if id == *BATTLE_OBJECT_ID_INVALID as u32 || !sv_battle_object::is_active(id) {
        return None;
    }
    let b = sv_battle_object::module_accessor(id);
    if b.is_null() { None } else { Some(b) }
}

unsafe fn hammer_express(boma: *mut BattleObjectModuleAccessor) -> bool {
    if stage::get_stage_id() != *StageID::BattleField_S {
        return false;
    }
    let mut hit = Vector2f { x: 0.0, y: 0.0 };
    let de = Vector2f { x: 0.0, y: 25.0 };
    let droite = GroundModule::ray_check_hit_pos(boma, &de, &Vector2f { x: 100.0, y: 0.0 }, &mut hit, true);
    let x_d = hit.x;
    let gauche = GroundModule::ray_check_hit_pos(boma, &de, &Vector2f { x: -100.0, y: 0.0 }, &mut hit, true);
    droite && gauche && (x_d - MUR).abs() < 8.0 && (hit.x + MUR).abs() < 8.0
}

unsafe fn balle_boma() -> Option<*mut BattleObjectModuleAccessor> {
    if BALLE == 0 { None } else { boma_de(BALLE) }
}

unsafe fn remettre_au_centre() {
    POS = CENTRE;
    DIR = (0.0, 0.0);
    VIT = 0.0;
    PROPRIO = -1;
    FIGE = 0;
    GRACE = 0;
    AMORTI = false;
    VY_AMORTI = 0.0;
    BONUS = 1.0;
}

unsafe fn apparaitre(boma: *mut BattleObjectModuleAccessor) {
    // v0.7 : le combattant ne doit JAMAIS rester avec la balle (c'est un objet lourd : il restait bloqué à la porter)
    // -> seulement quand il est debout sans rien faire, puis la balle est éjectée de sa main et il est remis debout
    if ItemModule::is_have_item(boma, 0) || StatusModule::status_kind(boma) != *FIGHTER_STATUS_KIND_WAIT
        || StatusModule::situation_kind(boma) != *SITUATION_KIND_GROUND {
        return;
    }
    ItemModule::have_item(boma, ItemKind(*ITEM_KIND_SOCCERBALL), 0, 0, false, false);
    if !ItemModule::is_have_item(boma, 0) {
        return;
    }
    let id = ItemModule::get_have_item_id(boma, 0) as u32;
    ItemModule::eject_have_item(boma, 0, false, false);
    if ItemModule::is_have_item(boma, 0) {
        ItemModule::drop_item(boma, 0.0, 0.0, 0);
    }
    StatusModule::change_status_force(boma, *FIGHTER_STATUS_KIND_WAIT, false);
    BALLE = id;
    remettre_au_centre();
    if let Some(ib) = boma_de(id) {
        StatusModule::change_status_force(ib, *ITEM_STATUS_KIND_FALL, false);
        PostureModule::set_pos(ib, &Vector3f { x: POS.0, y: POS.1, z: 0.0 });
        KineticModule::clear_speed_all(ib);
    }
}

unsafe fn milieu(o: *mut BattleObjectModuleAccessor) -> (f32, f32) {
    let p = PostureModule::pos(o);
    ((*p).x, (*p).y + 8.0)
}

/// le combattant le plus proche de la balle (celui qui vient de la frapper)
unsafe fn plus_proche(x: f32, y: f32) -> i32 {
    let mut best = (-1, 1.0e9f32);
    for e in 0..8 {
        let Some(o) = boma_de(Fighter::get_id_from_entry_id(e)) else { continue };
        let st = StatusModule::status_kind(o);
        if st == *FIGHTER_STATUS_KIND_DEAD || st == *FIGHTER_STATUS_KIND_STANDBY {
            continue;
        }
        let (ox, oy) = milieu(o);
        let d = (ox - x).powi(2) + (oy - y).powi(2);
        if d < best.1 {
            best = (e, d);
        }
    }
    best.0
}

unsafe fn physique() {
    let Some(ib) = balle_boma() else { return };
    // tenue par quelqu'un (ne devrait plus arriver) : on la libère au lieu de la déplacer
    if StatusModule::status_kind(ib) == *ITEM_STATUS_KIND_HAVE {
        StatusModule::change_status_force(ib, *ITEM_STATUS_KIND_FALL, false);
        PostureModule::set_pos(ib, &Vector3f { x: POS.0, y: POS.1, z: 0.0 });
        return;
    }
    // v0.9 : détection des coups par nous-mêmes (le jeu ne bouge pas la balle quand on la tape) :
    // un combattant dont un coup est actif (hitbox) et dont la zone de frappe, devant lui, touche la balle
    if FIGE == 0 {
        for e in 0..8 {
            let Some(o) = boma_de(Fighter::get_id_from_entry_id(e)) else { continue };
            let st = StatusModule::status_kind(o);
            if st == *FIGHTER_STATUS_KIND_DEAD || st == *FIGHTER_STATUS_KIND_STANDBY || st == *FIGHTER_STATUS_KIND_REBIRTH {
                continue;
            }
            let att = (0..8).any(|i| AttackModule::is_attack(o, i, false));
            if !att {
                DEJA[e as usize] = false;
                continue;
            }
            if DEJA[e as usize] {
                continue; // un seul renvoi par coup
            }
            let lr = PostureModule::lr(o);
            let p = PostureModule::pos(o);
            let (cx, cy) = ((*p).x + lr * 7.0, (*p).y + 10.0);
            let (dx, dy) = (POS.0 - cx, POS.1 - cy);
            let d = (dx * dx + dy * dy).sqrt();
            if d > R + PORTEE {
                continue;
            }
            DEJA[e as usize] = true;
            PROPRIO = e;
            let n = d.max(0.001);
            let (dir, mult, fige, amorti) = frappe(o, (dx / n, dy / n));
            if VIT <= 0.0 { VIT = V0; }
            if amorti {
                AMORTI = true;                    // garde sa vitesse ; la prochaine frappe ira plus vite
                VY_AMORTI = 2.2;
                BONUS = 1.35;
                DIR = dir;
            } else {
                AMORTI = false;
                DIR = dir;
                VIT = (VIT * ACCEL * mult * BONUS).min(VMAX);
                BONUS = 1.0;
            }
            FIGE = ((3.0 + VIT * 1.5) * fige) as i32; // arrêt sur image, plus long quand elle va vite
            GRACE = 6;
            break;
        }
    }
    if FIGE > 0 {
        FIGE -= 1;
    } else if AMORTI {
        // amorti : monte doucement puis retombe, rebondit sur les murs
        VY_AMORTI -= 0.09;
        let mut x = POS.0;
        let mut y = POS.1 + VY_AMORTI;
        if y < SOL + R { y = SOL + R; VY_AMORTI = VY_AMORTI.abs() * 0.5; }
        if y > PLAFOND - R { y = PLAFOND - R; VY_AMORTI = -VY_AMORTI.abs(); }
        POS = (x, y);
    } else if VIT > 0.0 {
        // avance en ligne droite, rebonds parfaits
        let mut x = POS.0 + DIR.0 * VIT;
        let mut y = POS.1 + DIR.1 * VIT;
        if x > MUR - R { x = 2.0 * (MUR - R) - x; DIR.0 = -DIR.0.abs(); }
        if x < -MUR + R { x = 2.0 * (-MUR + R) - x; DIR.0 = DIR.0.abs(); }
        if y < SOL + R { y = 2.0 * (SOL + R) - y; DIR.1 = DIR.1.abs(); }
        if y > PLAFOND - R { y = 2.0 * (PLAFOND - R) - y; DIR.1 = -DIR.1.abs(); }
        POS = (x, y);
    }
    PostureModule::set_pos(ib, &Vector3f { x: POS.0, y: POS.1, z: 0.0 });
    PostureModule::set_scale(ib, TAILLE, false);
    KineticModule::clear_speed_all(ib);
    AttackModule::clear_all(ib); // la balle ne blesse pas d'elle-même : c'est nous qui gérons le contact
    if GRACE > 0 {
        GRACE -= 1;
        return;
    }
    if VIT <= 0.0 || FIGE > 0 || AMORTI {
        return;
    }
    // contact avec un adversaire
    for e in 0..8 {
        let Some(o) = boma_de(Fighter::get_id_from_entry_id(e)) else { continue };
        let st = StatusModule::status_kind(o);
        if st == *FIGHTER_STATUS_KIND_DEAD || st == *FIGHTER_STATUS_KIND_STANDBY || st == *FIGHTER_STATUS_KIND_REBIRTH {
            continue;
        }
        let (ox, oy) = milieu(o);
        let dd = ((ox - POS.0).powi(2) + (oy - POS.1).powi(2)).sqrt();
        if dd > R + CORPS {
            continue;
        }
        if AttackModule::is_attack(o, 0, false) || AttackModule::is_attack(o, 1, false) {
            continue; // il est en train de frapper : le coup sera détecté à l'image suivante
        }
        if st == *FIGHTER_STATUS_KIND_GUARD || st == *FIGHTER_STATUS_KIND_GUARD_ON || st == *FIGHTER_STATUS_KIND_GUARD_DAMAGE {
            DIR = (-DIR.0, DIR.1); // bouclier : elle rebondit
            GRACE = 10;
            continue;
        }
        // éliminé d'un coup : vrai K.O. du jeu dans la direction de la balle
        ejecter(o, ox, oy - 8.0, DIR.0, DIR.1);
        remettre_au_centre();
        return;
    }
}

unsafe extern "C" fn ballon_frame(fighter: &mut L2CFighterCommon) {
    if EN_COURS {
        return;
    }
    EN_COURS = true;
    ballon_frame_(fighter);
    EN_COURS = false;
}

unsafe fn ballon_frame_(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let entree = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID);
    if !sv_information::is_ready_go() {
        EST_HE = -1;
        ATTENTE = DELAI;
        CHEF = -1;
        BALLE = 0;
        return;
    }
    if CHEF < 0 {
        CHEF = entree;
    }
    if entree != CHEF {
        SANS_CHEF += 1;
        if SANS_CHEF > 240 { CHEF = entree; } else { return; }
    }
    SANS_CHEF = 0;
    if EST_HE < 0 {
        EST_HE = hammer_express(boma) as i32;
    }
    if EST_HE == 0 {
        return;
    }
    if balle_boma().is_some() {
        ATTENTE = DELAI;
        physique();
        return;
    }
    if ATTENTE > 0 {
        ATTENTE -= 1;
        return;
    }
    let st = StatusModule::status_kind(boma);
    if st == *FIGHTER_STATUS_KIND_DEAD || st == *FIGHTER_STATUS_KIND_REBIRTH || st == *FIGHTER_STATUS_KIND_STANDBY {
        return;
    }
    apparaitre(boma);
}

#[skyline::main(name = "ballon")]
pub fn main() {
    Agent::new("fighter").on_line(Main, ballon_frame).install();
}
