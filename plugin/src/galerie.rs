//! Rangée d'effets candidats (outil de réglage, provisoire) : je ne vois pas les effets du jeu. Le joueur a choisi sur
//! photo un effet du défilé d'avant (dôme cyan à cœur clair posé au sol) : je l'identifie comme le jet de la
//! ceinture-fusée (armes::JET), sans certitude. Une provocation (taunt) affiche donc côte à côte, devant Chief, les 4
//! effets du défilé qui peuvent donner cette image, numérotés par des éclats rouges au-dessus (1, 2, 3 ou 4 éclats) :
//! si les jets du bas aérien / de l'ascenseur ne sont pas l'effet voulu, le joueur donne le bon numéro.
//! Même hauteur et même taille que dans le défilé (origine 5 au-dessus du sol, taille 1) pour comparer à la photo.

use smash::app::lua_bind::*;
use smash::app::*;
use smash::lib::lua_const::*;
use smash::lua2cpp::*;
use smash::phx::{Hash40, Vector3f};

pub const LISTE: &[&str] = &["sys_rocketbelt_jet", "sys_rocketbelt_burner", "sys_dolphinbomb_jet", "sys_bossgalaga_beam"];
const DUREE: i32 = 240;
const ECART: f32 = 9.0;
static mut T: [i32; 8] = [-1; 8];
static mut H: [[u32; 4]; 8] = [[0; 4]; 8];
static mut POS: [(f32, f32, f32); 8] = [(0.0, 0.0, 1.0); 8];

unsafe fn fin(boma: *mut BattleObjectModuleAccessor, e: usize) {
    for k in 0..LISTE.len() {
        if H[e][k] != 0 {
            EffectModule::kill(boma, H[e][k], true, true);
            H[e][k] = 0;
        }
    }
    T[e] = -1;
}

pub unsafe fn chaque_image(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let e = (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7);
    if StatusModule::status_kind(boma) == *FIGHTER_STATUS_KIND_ENTRY {
        T[e] = -1;
        H[e] = [0; 4];
        return;
    }
    if T[e] < 0 {
        if StatusModule::status_kind(boma) == *FIGHTER_STATUS_KIND_APPEAL && MotionModule::frame(boma) <= 1.0 {
            let p = PostureModule::pos(boma);
            POS[e] = ((*p).x, (*p).y, PostureModule::lr(boma));
            T[e] = 0;
        } else {
            return;
        }
    }
    let t = T[e];
    T[e] += 1;
    if t >= DUREE {
        fin(boma, e);
        return;
    }
    let z = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    let (x0, y0, lr) = POS[e];
    for k in 0..LISTE.len() {
        let x = x0 + (12.0 + ECART * k as f32) * lr;
        // l'effet : créé une fois ; recréé toutes les 45 images s'il s'est éteint (effets courts)
        if t % 45 == 0 && (H[e][k] == 0 || !EffectModule::is_exist_effect(boma, H[e][k])) {
            let p = Vector3f { x, y: y0 + 5.0, z: 0.0 };
            H[e][k] = EffectModule::req(boma, Hash40::new(LISTE[k]), &p, &z, 1.0, 0, 0, false, 0) as u32;
        }
        // son numéro : k + 1 éclats rouges empilés au-dessus
        if t % 30 == 0 {
            for n in 0..=k {
                let m = Vector3f { x, y: y0 + 22.0 + 4.0 * n as f32, z: 0.0 };
                let h = EffectModule::req(boma, Hash40::new("sys_smash_flash_s"), &m, &z, 0.7, 0, 0, false, 0) as u32;
                if h != 0 {
                    EffectModule::set_rgb(boma, h, 1.0, 0.1, 0.1);
                }
            }
        }
    }
}
