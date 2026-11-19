//! Dust Knuckles (vidéo de REX : tilt avant et avant aérien = jusqu'à 3 élans aux poings en appuyant de nouveau sur A,
//! arrière aérien aux poings). L'animation contient les 3 élans à la suite (AttackS3S / S3S1 / S3S2,
//! AttackAirF / FLoop1 / FLoop2) : sans nouvel appui, le coup s'arrête à la fin de l'élan en cours ; avec un appui
//! pendant l'élan, il passe à l'élan suivant dès que ses coups sont finis.
//! Les poings ne sont visibles que dans ces coups (la pelle est alors cachée), comme dans REX (changement de modèle
//! 0B000200 2 2 au début de chaque élan). En l'air, il avance tout droit (distance de REX, table AVANCE_AIR), sans monter ni descendre.

use smash::app::lua_bind::*;
use smash::app::*;
use smash::hash40;
use smash::lib::lua_const::*;
use smash::lua2cpp::*;
use smash::phx::{Hash40, Vector3f};
use smash_script::*;

/// image (dans l'élan) à partir de laquelle on peut passer au suivant : ses coups sont finis (effacés à 14-15)
const SUITE_DES: f32 = 16.0;
/// appui accepté à partir de cette image de l'élan
const APPUI_DES: f32 = 6.0;

static mut DEMANDE: [bool; 8] = [false; 8];
static mut ELAN: [usize; 8] = [0; 8];
static mut AIR_PREC: [(u64, f32); 8] = [(0, 0.0); 8];
static mut AIR_ACTIF: [bool; 8] = [false; 8];

unsafe fn id_of(boma: *mut BattleObjectModuleAccessor) -> usize {
    (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7)
}

fn table(t: &[f32], f: f32) -> f32 {
    if t.is_empty() {
        return 0.0;
    }
    let f = f.max(0.0);
    let i = f.floor() as usize;
    if i + 1 >= t.len() {
        return t[t.len() - 1];
    }
    let k = f - i as f32;
    t[i] * (1.0 - k) + t[i + 1] * k
}

pub unsafe fn dust_knuckles(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let id = id_of(boma);
    let m = MotionModule::motion_kind(boma);
    let sol = m == hash40("attack_s3_s");
    let air = m == hash40("attack_air_f");
    let arriere = m == hash40("attack_air_b");
    let poings = sol || air || arriere;
    ModelModule::set_mesh_visibility(boma, Hash40::new("dustknuckles"), poings);
    ModelModule::set_mesh_visibility(boma, Hash40::new("shovelr"), !poings);

    // déplacement en l'air des élans : tout droit devant, sans garder l'élan vertical (vidéo du joueur : il montait
    // ou descendait avec son saut) ; mouvement « par l'animation » (MOTION_AIR : ni gravité ni contrôle, animation sur
    // place) + avancée de REX vers l'avant seulement (table AVANCE_AIR, sans sa composante verticale)
    if air && StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR {
        let f = MotionModule::frame(boma);
        let (pm, pf) = AIR_PREC[id];
        AIR_PREC[id] = (m, f);
        if !AIR_ACTIF[id] {
            AIR_ACTIF[id] = true;
            KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_MOTION_AIR);
        }
        // v6 encore : il gardait l'élan du saut (le statut aérien remet ses énergies) -> à chaque image : toutes les
        // vitesses à zéro, gravité et contrôle suspendus ; seul le déplacement vers l'avant ci-dessous reste
        KineticModule::clear_speed_all(boma);
        KineticModule::suspend_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
        KineticModule::suspend_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_CONTROL);
        if let Some((_, tz, _)) = crate::AVANCE_AIR.iter().find(|(n, _, _)| *n == "attack_air_f") {
            if pm == m && f >= pf {
                let dz = table(tz, f) - table(tz, pf);
                let lr = PostureModule::lr(boma);
                PostureModule::add_pos(boma, &Vector3f { x: dz * lr, y: 0.0, z: 0.0 });
            }
        }
    } else {
        AIR_PREC[id] = (0, 0.0);
        if AIR_ACTIF[id] {
            AIR_ACTIF[id] = false;
            KineticModule::resume_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
            KineticModule::resume_energy(boma, *FIGHTER_KINETIC_ENERGY_ID_CONTROL);
        }
    }

    // enchaînement des élans
    if !(sol || air) {
        DEMANDE[id] = false;
        ELAN[id] = 0;
        return;
    }
    let script = if sol { "game_attacks3" } else { "game_attackairf" };
    let d = crate::debuts(script);
    let f = MotionModule::frame(boma);
    let e = if f >= d[2] && d[2] > 0.0 { 2 } else if f >= d[1] && d[1] > 0.0 { 1 } else { 0 };
    if e != ELAN[id] {
        ELAN[id] = e;
        DEMANDE[id] = false;
    }
    if e >= 2 {
        return;
    }
    let debut = d[e];
    let fin = d[e + 1];
    if f >= debut + APPUI_DES && ControlModule::check_button_trigger(boma, *CONTROL_PAD_BUTTON_ATTACK) {
        DEMANDE[id] = true;
    }
    if DEMANDE[id] && f >= debut + SUITE_DES {
        DEMANDE[id] = false;
        MotionModule::set_frame_sync_anim_cmd(boma, fin, true, true, false);
        return;
    }
    if !DEMANDE[id] && f >= fin - 1.0 {
        // pas de nouvel appui : fin du coup à la fin de cet élan
        let st = if StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR { *FIGHTER_STATUS_KIND_FALL } else { *FIGHTER_STATUS_KIND_WAIT };
        StatusModule::change_status_request_from_script(boma, st, false);
    }
}
