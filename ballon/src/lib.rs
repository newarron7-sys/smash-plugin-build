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
//! - v0.22 : la balle ne clignote / disparaît plus : toujours en statut FALL, sans collision décor.
//! - v0.21 : la balle tourne nettement sur elle-même quand elle se déplace.
//! - v0.20 : freeze plus long à la frappe (10 + 3,5 × vitesse images, max 55).
//! - v0.19 : effets de coup sur la balle : étincelle selon l'attribut, son d'impact, secousse de l'écran.
//! - v0.18 : petite fumée blanche quand la balle rebondit sur le sol, un mur ou le plafond.
//! - v0.17 : la balle ne disparaît plus (durée de vie remise à sa valeur de départ à chaque image).
//! - v0.16 : élimination visible : dégâts de feu, gel, vol en tournoyant à travers murs et sol jusqu'au K.O.
//! - v0.15 : arrêt sur image du joueur qui frappe + balle (plus long) ; le lanceur ne peut pas être éliminé par sa balle.
//! - v0.14 : direction = angle réel de la hitbox qui touche (comme sur un adversaire), détection par la vraie hitbox.
//! - v0.13 : détection du stage : le booléen du rayon n'est pas fiable -> seulement les positions des murs (±74.2).
//! - v0.12 : journal de diagnostic sd:/ballon_log.txt.
//! - v0.11 : apparition par n'importe quel combattant au sol et libre (la balle n'apparaissait pas si P1 bougeait).
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

/// v0.12 : journal de diagnostic sur la carte SD (sd:/ballon_log.txt)
static mut NB_LOG: u32 = 0;
fn log(m: &str) {
    unsafe {
        if NB_LOG > 400 { return; }
        NB_LOG += 1;
    }
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open("sd:/ballon_log.txt") {
        let _ = writeln!(f, "{}", m);
    }
}
static mut IMAGE: u32 = 0;

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
/// v0.16 : éjection visible comme dans Brawl : dégâts de feu, gel, puis le combattant vole en tournoyant dans la
/// direction de la balle, traverse les murs et le sol et retombe jusqu'à la zone de K.O.
#[derive(Clone, Copy)]
struct Vol { actif: bool, gel: i32, t: i32, x: f32, y: f32, vx: f32, vy: f32 }
static mut VOLS: [Vol; 8] = [Vol { actif: false, gel: 0, t: 0, x: 0.0, y: 0.0, vx: 0.0, vy: 0.0 }; 8];
const V_EJ: f32 = 5.5;
const GRAV_EJ: f32 = 0.2;

unsafe fn feu(o: *mut BattleObjectModuleAccessor, sc: f32) {
    let z = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    EffectModule::req_follow(o, smash::phx::Hash40::new("sys_damage_fire"), smash::phx::Hash40::new("hip"), &z, &z, sc, true, 0, 0, 0, 0, 0, false, false);
}

unsafe fn eliminer(e: i32, o: *mut BattleObjectModuleAccessor) {
    let p = PostureModule::pos(o);
    let (x, y) = ((*p).x, (*p).y);
    DamageModule::add_damage(o, 10.0 + VIT * 2.0, 0);
    let gel = (((6.0 + VIT * 2.5) * 1.3) as i32).clamp(10, 45);
    let rot = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    EffectModule::req(o, smash::phx::Hash40::new("sys_hit_fire"), &Vector3f { x: POS.0, y: POS.1, z: 0.0 }, &rot, 1.6, 0, 0, false, 0);
    feu(o, 1.2);
    StatusModule::change_status_force(o, *FIGHTER_STATUS_KIND_DAMAGE_FLY_ROLL, false);
    GroundModule::correct(o, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
    GroundModule::set_collidable(o, false);
    StopModule::set_hit_stop_frame(o, gel, false);
    let (dx, dy) = if DIR.0.abs() + DIR.1.abs() < 0.01 { (1.0, 0.0) } else { DIR };
    VOLS[e as usize] = Vol { actif: true, gel, t: 0, x, y: y.max(SOL + 1.0), vx: dx * V_EJ, vy: dy * V_EJ + 1.0 };
    log(&format!("elimination : joueur {} dir ({:.2},{:.2}) gel {}", e, dx, dy, gel));
}

unsafe fn vols() {
    for e in 0..8 {
        let v = VOLS[e];
        if !v.actif { continue; }
        let Some(o) = boma_de(Fighter::get_id_from_entry_id(e as i32)) else { VOLS[e].actif = false; continue };
        let st = StatusModule::status_kind(o);
        if st == *FIGHTER_STATUS_KIND_DEAD || st == *FIGHTER_STATUS_KIND_REBIRTH || st == *FIGHTER_STATUS_KIND_STANDBY {
            GroundModule::set_collidable(o, true);
            VOLS[e].actif = false;
            continue;
        }
        let mut v = v;
        if v.gel > 0 {
            v.gel -= 1;
        } else {
            v.vy -= GRAV_EJ;
            v.x += v.vx;
            v.y += v.vy;
            v.t += 1;
            if v.t % 12 == 0 { feu(o, 1.0); }
            if st != *FIGHTER_STATUS_KIND_DAMAGE_FLY_ROLL && st != *FIGHTER_STATUS_KIND_DAMAGE_FLY {
                StatusModule::change_status_force(o, *FIGHTER_STATUS_KIND_DAMAGE_FLY_ROLL, false);
            }
        }
        if v.t > 150 {
            // sécurité : toujours vivant -> directement hors de la zone de K.O. (ancienne méthode)
            GroundModule::set_collidable(o, true);
            VOLS[e].actif = false;
            ejecter(o, v.x, v.y, v.vx, v.vy);
            continue;
        }
        GroundModule::correct(o, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
        PostureModule::set_pos(o, &Vector3f { x: v.x, y: v.y, z: 0.0 });
        KineticModule::clear_speed_all(o);
        VOLS[e] = v;
    }
}

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
static mut VIE0: i32 = -1; // v0.17 : durée de vie de la balle à son apparition, remise chaque image (plus de clignotement / disparition)
static mut ROT: f32 = 0.0; // v0.21 : rotation visuelle de la balle (degrés)
static mut EN_COURS: bool = false; // v0.8 : anti ré-entrée (changer le statut relance la boucle du combattant -> récursion infinie)
static mut BONUS: f32 = 1.0; // frappe après un amorti : plus forte

/// comment le combattant a frappé : (direction, multiplicateur de vitesse, arrêt sur image ×, amorti)
/// v0.14 : la hitbox active la plus proche de la balle -> (distance - portée, angle du coup en degrés)
unsafe fn hitbox_touche(o: *mut BattleObjectModuleAccessor) -> Option<(f32, i32, u64)> {
    let mut best: Option<(f32, i32, u64)> = None;
    for i in 0..8 {
        if !AttackModule::is_attack(o, i, false) {
            continue;
        }
        let ad = AttackModule::attack_data(o, i, false);
        if ad.is_null() {
            continue;
        }
        let ad = *ad;
        let mut jp = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
        ModelModule::joint_global_position(o, smash::phx::Hash40 { hash: ad.node }, &mut jp, true);
        let ol = (ad.x * ad.x + ad.y * ad.y + ad.z * ad.z).sqrt();
        let portee = ad.size.max(2.0) + ol * 0.8 + R + 1.5;
        let d = ((POS.0 - jp.x).powi(2) + (POS.1 - jp.y).powi(2)).sqrt() - portee;
        if d < 0.0 && best.map_or(true, |b| d < b.0) {
            best = Some((d, ad.vector, ad.attr));
        }
    }
    best
}

/// v0.14 : direction comme sur un vrai adversaire : l'angle du coup (0 = devant, 90 = haut, 270 = bas)
unsafe fn direction_coup(o: *mut BattleObjectModuleAccessor, angle: i32) -> (f32, f32) {
    let p = PostureModule::pos(o);
    let mut lr = PostureModule::lr(o);
    if (POS.0 - (*p).x) * lr < -2.0 {
        lr = -lr; // balle derrière le combattant (coup arrière) : vers l'arrière
    }
    let a = match angle {
        361 => 40,                                 // angle « Sakurai »
        a if (0..360).contains(&a) => a,
        _ => {
            // angles spéciaux (360+) : à l'opposé du combattant
            let (dx, dy) = (POS.0 - (*p).x, POS.1 - ((*p).y + 10.0));
            let n = (dx * dx + dy * dy).sqrt().max(0.001);
            return (dx / n, dy / n);
        }
    } as f32;
    let r = a.to_radians();
    (r.cos() * lr, r.sin())
}

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
    // v0.13 : le booléen renvoyé n'est pas fiable (droite=false alors que le mur est touché à x=74.2) :
    // on ne regarde que la position touchée (initialisée hors stage pour repérer un vrai « rien touché »)
    let de = Vector2f { x: 0.0, y: 25.0 };
    let mut hd = Vector2f { x: 999.0, y: 0.0 };
    let droite = GroundModule::ray_check_hit_pos(boma, &de, &Vector2f { x: 100.0, y: 0.0 }, &mut hd, true);
    let mut hg = Vector2f { x: 999.0, y: 0.0 };
    let gauche = GroundModule::ray_check_hit_pos(boma, &de, &Vector2f { x: -100.0, y: 0.0 }, &mut hg, true);
    log(&format!("detection stage : droite={} x={:.1} gauche={} x={:.1}", droite, hd.x, gauche, hg.x));
    (hd.x - MUR).abs() < 8.0 && (hg.x + MUR).abs() < 8.0
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
    // v0.11 : n'importe quel combattant au sol et libre (pas seulement le joueur 1 immobile)
    let st = StatusModule::status_kind(boma);
    let libre = [*FIGHTER_STATUS_KIND_WAIT, *FIGHTER_STATUS_KIND_WALK, *FIGHTER_STATUS_KIND_DASH, *FIGHTER_STATUS_KIND_RUN,
        *FIGHTER_STATUS_KIND_RUN_BRAKE, *FIGHTER_STATUS_KIND_TURN, *FIGHTER_STATUS_KIND_TURN_DASH, *FIGHTER_STATUS_KIND_TURN_RUN,
        *FIGHTER_STATUS_KIND_SQUAT, *FIGHTER_STATUS_KIND_SQUAT_WAIT, *FIGHTER_STATUS_KIND_SQUAT_RV, *FIGHTER_STATUS_KIND_LANDING];
    if ItemModule::is_have_item(boma, 0) || !libre.contains(&st)
        || StatusModule::situation_kind(boma) != *SITUATION_KIND_GROUND {
        return;
    }
    ItemModule::have_item(boma, ItemKind(*ITEM_KIND_SOCCERBALL), 0, 0, false, false);
    let tient = ItemModule::is_have_item(boma, 0);
    let id0 = ItemModule::get_have_item_id(boma, 0);
    log(&format!("apparition : statut={} tient={} id={:#x} actif={}", st, tient, id0, boma_de(id0 as u32).is_some()));
    if !tient {
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
        VIE0 = WorkModule::get_int(ib, *ITEM_INSTANCE_WORK_INT_LIFE_TIME);
        log(&format!("vie balle : {} / max {}", VIE0, WorkModule::get_int(ib, *ITEM_INSTANCE_WORK_INT_LIFE_TIME_MAX)));
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

/// v0.19 : effets d'un vrai coup : étincelle selon l'attribut du coup, son d'impact, secousse de l'écran
unsafe fn impact(o: *mut BattleObjectModuleAccessor, ib: *mut BattleObjectModuleAccessor, attr: u64) {
    use smash::phx::Hash40;
    let a = |n: &str| Hash40::new(n).hash == attr;
    let niv = if VIT < 3.0 { "s" } else if VIT < 6.0 { "m" } else { "l" };
    let (fx, se) = if a("collision_attr_fire") {
        ("sys_hit_fire", "se_common_fire_m_damage".to_string())
    } else if a("collision_attr_elec") {
        ("sys_hit_elec", format!("se_common_electric_hit_{}", niv))
    } else if a("collision_attr_cutup") {
        ("sys_hit_cut", format!("se_common_sword_hit_{}", niv))
    } else if a("collision_attr_ice") {
        ("sys_hit_ice", format!("se_common_kick_hit_{}", niv))
    } else if a("collision_attr_magic") {
        ("sys_hit_magic", format!("se_common_magic_hit_{}", niv))
    } else if a("collision_attr_purple") {
        ("sys_hit_purple", format!("se_common_magic_hit_{}", niv))
    } else if VIT >= 6.0 {
        ("sys_hit_normal_l", format!("se_common_kick_hit_{}", niv))
    } else {
        ("sys_hit_normal", format!("se_common_kick_hit_{}", niv))
    };
    let rot = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    let sc = (0.7 + VIT * 0.08).min(1.6);
    EffectModule::req(ib, Hash40::new(fx), &Vector3f { x: POS.0, y: POS.1, z: 0.0 }, &rot, sc, 0, 0, false, 0);
    SoundModule::play_se(o, Hash40::new(&se), true, false, false, false, enSEType(0));
    if VIT >= 9.5 {
        CameraModule::req_quake(o, *CAMERA_QUAKE_KIND_L);
    } else if VIT >= 7.0 {
        CameraModule::req_quake(o, *CAMERA_QUAKE_KIND_M);
    } else if VIT >= 4.0 {
        CameraModule::req_quake(o, *CAMERA_QUAKE_KIND_S);
    }
}

unsafe fn fumee(ib: *mut BattleObjectModuleAccessor, x: f32, y: f32, rz: f32) {
    let rot = Vector3f { x: 0.0, y: 0.0, z: rz };
    EffectModule::req(ib, smash::phx::Hash40::new("sys_landing_smoke_s"), &Vector3f { x, y, z: 0.0 }, &rot, 0.9, 0, 0, false, 0);
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
            let Some((_, angle, attr)) = hitbox_touche(o) else { continue };
            DEJA[e as usize] = true;
            PROPRIO = e;
            let dir = direction_coup(o, angle);
            let st2 = StatusModule::status_kind(o);
            let smash = st2 == *FIGHTER_STATUS_KIND_ATTACK_S4 || st2 == *FIGHTER_STATUS_KIND_ATTACK_HI4 || st2 == *FIGHTER_STATUS_KIND_ATTACK_LW4;
            let (mult, fige, amorti) = if smash { (1.7, 1.2, false) } else { (1.0, 1.0, false) };
            log(&format!("frappe : joueur {} statut {} angle {} -> dir ({:.2},{:.2})", e, st2, angle, dir.0, dir.1));
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
            // v0.15 : arrêt sur image de la balle ET du joueur qui frappe, plus long quand elle va vite
            FIGE = (((10.0 + VIT * 3.5) * fige) as i32).min(55); // v0.20 : freeze plus long
            StopModule::set_hit_stop_frame(o, FIGE, false);
            impact(o, ib, attr);
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
        if y < SOL + R { y = SOL + R; if VY_AMORTI < -0.6 { fumee(ib, x, SOL, 0.0); } VY_AMORTI = VY_AMORTI.abs() * 0.5; }
        if y > PLAFOND - R { y = PLAFOND - R; VY_AMORTI = -VY_AMORTI.abs(); }
        POS = (x, y);
    } else if VIT > 0.0 {
        // avance en ligne droite, rebonds parfaits
        let mut x = POS.0 + DIR.0 * VIT;
        let mut y = POS.1 + DIR.1 * VIT;
        // v0.18 : petite fumée blanche à chaque rebond (mur, sol, plafond)
        if x > MUR - R { x = 2.0 * (MUR - R) - x; DIR.0 = -DIR.0.abs(); fumee(ib, MUR, y, 90.0); }
        if x < -MUR + R { x = 2.0 * (-MUR + R) - x; DIR.0 = DIR.0.abs(); fumee(ib, -MUR, y, -90.0); }
        if y < SOL + R { y = 2.0 * (SOL + R) - y; DIR.1 = DIR.1.abs(); fumee(ib, x, SOL, 0.0); }
        if y > PLAFOND - R { y = 2.0 * (PLAFOND - R) - y; DIR.1 = -DIR.1.abs(); fumee(ib, x, PLAFOND, 180.0); }
        POS = (x, y);
    }
    PostureModule::set_pos(ib, &Vector3f { x: POS.0, y: POS.1, z: 0.0 });
    PostureModule::set_scale(ib, TAILLE, false);
    // v0.21 : la balle tourne sur elle-même quand elle bouge, plus vite quand elle va vite
    if FIGE == 0 && (VIT > 0.0 || AMORTI) {
        let sens = if DIR.0 < -0.01 { 1.0 } else { -1.0 };
        let v = if AMORTI { 6.0 } else { VIT };
        ROT = (ROT + sens * (10.0 + v * 4.0).min(45.0)) % 360.0;
    }
    PostureModule::set_rot(ib, &Vector3f { x: 0.0, y: 0.0, z: ROT }, 0);
    // v0.22 : la balle passait au sol dans un autre statut (4) où elle ne suivait plus nos positions, puis
    // clignotait et disparaissait : on la garde toujours « en chute » et sans collision avec le décor
    if StatusModule::status_kind(ib) != *ITEM_STATUS_KIND_FALL {
        StatusModule::change_status_force(ib, *ITEM_STATUS_KIND_FALL, false);
    }
    GroundModule::set_collidable(ib, false);
    if VIE0 >= 0 {
        if IMAGE % 600 == 0 {
            log(&format!("vie balle avant remise : {}", WorkModule::get_int(ib, *ITEM_INSTANCE_WORK_INT_LIFE_TIME)));
        }
        WorkModule::set_int(ib, VIE0, *ITEM_INSTANCE_WORK_INT_LIFE_TIME);
    }
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
        if e == PROPRIO || VOLS[e as usize].actif {
            continue; // v0.15 : sa propre balle ne l'élimine pas
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
        let _ = (ox, oy);
        eliminer(e, o);
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
        for v in VOLS.iter_mut() { v.actif = false; }
        return;
    }
    if CHEF < 0 {
        CHEF = entree;
    }
    // apparition : le premier combattant libre s'en charge (avant : seulement le chef, et seulement immobile)
    if EST_HE == 1 && ATTENTE <= 0 && balle_boma().is_none() {
        apparaitre(boma);
        if balle_boma().is_some() {
            return;
        }
    }
    if entree != CHEF {
        SANS_CHEF += 1;
        if SANS_CHEF > 240 { CHEF = entree; } else { return; }
    }
    SANS_CHEF = 0;
    if EST_HE < 0 {
        EST_HE = hammer_express(boma) as i32;
        log(&format!("stage {} -> Hammer Express = {} (chef = joueur {})", stage::get_stage_id(), EST_HE, CHEF));
    }
    if EST_HE == 0 {
        return;
    }
    IMAGE += 1;
    if IMAGE % 120 == 0 {
        let pb = match balle_boma() { Some(ib) => { let p = PostureModule::pos(ib); format!("({:.1},{:.1}) statut {}", (*p).x, (*p).y, StatusModule::status_kind(ib)) }, None => "aucune".into() };
        log(&format!("image {} : balle id={:#x} {} attente={} vit={:.2}", IMAGE, BALLE, pb, ATTENTE, VIT));
    }
    vols();
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
    log("=== plugin ballon v0.22 chargé ===");
    Agent::new("fighter").on_line(Main, ballon_frame).install();
}
