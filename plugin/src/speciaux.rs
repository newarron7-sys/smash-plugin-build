//! Spéciaux de Master Chief (REX, FitChief.pac : actions 0x113 à 0x115 ; vidéos du joueur, noms des coups affichés).
//!
//! Dans REX, ses objets sont des objets de Brawl créés par son module (ft_chief.rel), pas des modèles à lui :
//! - side B « Plasma Grenade » (Garnade toss) : la bombe collante de Brawl (objet « Gooey Bomb »), prise en main à
//!   l'image 3, lancée à l'image 19 (commande 1F010300 : lancer d'objet, vitesse 1,877, angle visé au stick ; flèche
//!   verte de visée) ; ~20 % (vidéo). Ultimate a la même bombe collante (ITEM_KIND_CHEWING) : lueur bleue électrique
//!   ajoutée dessus (vidéo : « boule bleue électrique »).
//! - down B « Fusion Coil » : posée à l'image 9 (drapeau V12000044) : la caisse explosive de Brawl (on la porte, elle
//!   explose quand on la frappe : ~10-19 %) = ITEM_KIND_POWDERBOX d'Ultimate ; une à la fois. Demandé : elle explose
//!   au premier contact (la caisse d'Ultimate encaisse plusieurs coups) : explosion faite par le plugin.
//! - up B « Gravity Lift » (vidéo de REX) : jeté au sol DEVANT lui à l'image 13 (V12000043), s'ouvre à plat (étoile
//!   violette) ; colonne de lumière (demandé : le jet cyan choisi sur photo, vers le haut, dans un halo blanc, comme
//!   le Chief d'un autre moddeur) ; Chief n'est pas propulsé en le posant : il doit marcher
//!   dedans, il s'accroupit puis monte très haut (~5 fois sa taille) et retombe ; il peut y retourner. En l'air :
//!   lancé en diagonale vers le bas. S'il est déjà posé : « NotReady » (drapeau V22000010 de REX).
//!   (essai de sys_direction2 pour les effets bleus : ce sont des traits blancs fins, retiré)
//! Statuts de Snake remplacés (Nikita, Cypher, C4 jamais créés) ; animations de REX : retarget_exo.py chief_speciaux.

use smash::app::lua_bind::*;
use smash::app::sv_animcmd::frame;
use smash::app::*;
use smash::hash40;
use smash::lib::lua_const::*;
use smash::lib::L2CValue;
use smash::lua2cpp::*;
use smash::phx::{Hash40, Vector2f, Vector3f};
use smash_script::macros::*;
use smash_script::*;
use smashline::*;

const BLEU: (f32, f32, f32) = (0.35, 0.7, 1.0);
const VIOLET: (f32, f32, f32) = (0.75, 0.35, 1.0);

unsafe fn slot(boma: *mut BattleObjectModuleAccessor) -> usize {
    (WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize).min(7)
}

/// sol / air pendant un spécial : physique et animation suivent (il peut tomber d'un bord ou atterrir)
unsafe fn situation(fighter: &mut L2CFighterCommon, sol_mot: &str, air_mot: &str, init: bool) {
    let boma = fighter.module_accessor;
    let sol = StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND;
    if !init && !StatusModule::is_situation_changed(boma) {
        return;
    }
    if sol {
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_GROUND_STOP);
        GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_GROUND_CLIFF_STOP));
    } else {
        // en l'air : il tombe normalement (contrôle gardé), comme dans REX
        KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_FALL);
        GroundModule::correct(boma, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
    }
    let m = Hash40::new(if sol { sol_mot } else { air_mot });
    if init {
        MotionModule::change_motion(boma, m, 0.0, 1.0, false, 0.0, false, false);
    } else {
        MotionModule::change_motion_inherit_frame(boma, m, -1.0, 1.0, 0.0, false, false);
    }
}

unsafe fn finir(fighter: &mut L2CFighterCommon) -> bool {
    let boma = fighter.module_accessor;
    if !MotionModule::is_end(boma) {
        return false;
    }
    if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND {
        fighter.change_status(FIGHTER_STATUS_KIND_WAIT.into(), false.into());
    } else {
        fighter.change_status(FIGHTER_STATUS_KIND_FALL.into(), false.into());
    }
    true
}

unsafe extern "C" fn rien_status(_fighter: &mut L2CFighterCommon) -> L2CValue {
    0.into()
}

// ------------------------------------------------------------------ side B : Plasma Grenade
const G_PRISE: f32 = 3.0;
const G_LANCER: f32 = 19.0;
static mut G_ETAT: [u8; 8] = [0; 8]; // 0 rien, 1 en main, 2 lancée
static mut G_ID: [u32; 8] = [0; 8];
static mut G_T: [i32; 8] = [0; 8];

unsafe extern "C" fn s_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let e = slot(fighter.module_accessor);
    G_ETAT[e] = 0;
    situation(fighter, "special_s_start", "special_air_s_start", true);
    fighter.sub_shift_status_main(L2CValue::Ptr(s_loop as *const () as _))
}

unsafe extern "C" fn s_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    situation(fighter, "special_s_start", "special_air_s_start", false);
    let f = MotionModule::frame(boma);
    // vitesses de l'animation de REX (FSM) : visée ralentie (0,5) tant que B est tenu, puis 1,15 et 1,63 après le lancer
    let rate = if f < 10.0 {
        1.0
    } else if f < G_LANCER {
        if ControlModule::check_button_on(boma, *CONTROL_PAD_BUTTON_SPECIAL) { 0.5 } else { 1.0 }
    } else if f < 22.0 {
        1.15
    } else {
        1.63
    };
    MotionModule::set_rate(boma, rate);
    if G_ETAT[e] == 0 && f >= G_PRISE && !ItemModule::is_have_item(boma, 0) {
        ItemModule::have_item(boma, ItemKind(*ITEM_KIND_CHEWING), 0, 0, false, false);
        G_ETAT[e] = 1;
    }
    if G_ETAT[e] == 1 && f >= G_LANCER {
        // angle visé au stick (REX : V11000030, flèche verte) : 35° par défaut, de 5° (bas) à 75° (haut)
        let sy = ControlModule::get_stick_y(boma);
        let angle = (35.0 + 40.0 * sy).clamp(5.0, 75.0);
        let id = ItemModule::get_have_item_id(boma, 0) as u32;
        ItemModule::throw_item(boma, angle, 1.877, 1.0, 0, true, 0.0);
        if ItemModule::is_have_item(boma, 0) {
            ItemModule::drop_item(boma, angle, 1.877, 0);
        }
        G_ID[e] = id;
        G_T[e] = 0;
        G_ETAT[e] = 2;
    }
    finir(fighter);
    0.into()
}

unsafe extern "C" fn s_end(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    // interrompu avant le lancer : la bombe ne reste pas dans sa main
    if G_ETAT[e] == 1 && ItemModule::is_have_item(boma, 0) {
        ItemModule::remove_item(boma, 0);
    }
    G_ETAT[e] = 0;
    0.into()
}

/// lueur bleue électrique sur la bombe lancée (tant qu'elle existe) ; quand elle disparaît (elle a explosé),
/// explosion affichée à sa dernière position (le modèle de Chief n'a pas l'os « Have » où le jeu met la sienne)
static mut G_POS: [(f32, f32); 8] = [(0.0, 0.0); 8];

unsafe fn lueur_grenade(boma: *mut BattleObjectModuleAccessor) {
    let e = slot(boma);
    let id = G_ID[e];
    if id == 0 {
        return;
    }
    let actif = sv_battle_object::is_active(id);
    let ib = if actif { sv_battle_object::module_accessor(id) } else { std::ptr::null_mut() };
    if ib.is_null() || G_T[e] > 1500 {
        if G_T[e] > 2 && G_T[e] <= 1500 {
            let p = Vector3f { x: G_POS[e].0, y: G_POS[e].1, z: 0.0 };
            let z = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
            EffectModule::req(boma, Hash40::new("sys_bomb_a"), &p, &z, 0.9, 0, 0, false, 0);
        }
        G_ID[e] = 0;
        return;
    }
    let p = PostureModule::pos(ib);
    G_POS[e] = ((*p).x, (*p).y);
    G_T[e] += 1;
    if G_T[e] % 6 != 1 {
        return;
    }
    let z = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    let h = EffectModule::req_follow(ib, Hash40::new("sys_hit_elec"), Hash40::new("top"), &z, &z, 0.3, true, 0, 0, 0, 0, 0, false, false);
    EffectModule::set_rgb(ib, h as u32, BLEU.0, BLEU.1, BLEU.2);
}

// ------------------------------------------------------------------ down B : Fusion Coil
const C_POSE: f32 = 9.0;
static mut C_FAIT: [bool; 8] = [false; 8];
static mut C_ID: [u32; 8] = [0; 8];

unsafe extern "C" fn lw_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let e = slot(fighter.module_accessor);
    C_FAIT[e] = false;
    situation(fighter, "special_lw_start", "special_air_lw_start", true);
    fighter.sub_shift_status_main(L2CValue::Ptr(lw_loop as *const () as _))
}

unsafe extern "C" fn lw_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    situation(fighter, "special_lw_start", "special_air_lw_start", false);
    if !C_FAIT[e] && MotionModule::frame(boma) >= C_POSE {
        C_FAIT[e] = true;
        // une seule à la fois (REX) ; pas s'il tient déjà un objet
        let existe = C_ID[e] != 0 && sv_battle_object::is_active(C_ID[e]);
        if !existe && !ItemModule::is_have_item(boma, 0) {
            ItemModule::have_item(boma, ItemKind(*ITEM_KIND_POWDERBOX), 0, 0, false, false);
            C_ID[e] = ItemModule::get_have_item_id(boma, 0) as u32;
            ItemModule::drop_item(boma, 0.0, 0.0, 0);
        }
    }
    finir(fighter);
    0.into()
}

/// gestionnaire d'objets du jeu (singleton) : pour retirer la caisse qui explose
static mut ITEM_MGR: usize = 0;
unsafe fn item_manager() -> *mut smash::app::ItemManager {
    if ITEM_MGR == 0 {
        let mut a: usize = 0;
        skyline::nn::ro::LookupSymbol(&mut a, "_ZN3lib9SingletonIN3app11ItemManagerEE9instance_E\0".as_ptr() as _);
        ITEM_MGR = a;
    }
    if ITEM_MGR == 0 { std::ptr::null_mut() } else { *(ITEM_MGR as *const *mut smash::app::ItemManager) }
}

const C_RAYON: f32 = 10.0;
static mut C_BOUM: [i32; 8] = [0; 8];

/// demandé : la Fusion Coil explose au premier contact : un adversaire qui la touche (ou la frappe), elle-même frappée
/// (dégâts reçus), ou lancée par Chief quand elle touche le sol / un mur / quelqu'un. Pas tant que Chief la tient.
/// Explosion : la caisse est retirée, effet + son d'explosion du jeu, coup de Chief (crédité à Chief) à sa place
unsafe fn coil_contact(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    if C_BOUM[e] > 0 {
        C_BOUM[e] -= 1;
        if C_BOUM[e] == 0 {
            AttackModule::clear(boma, 7, false);
        }
    }
    let id = C_ID[e];
    if id == 0 {
        return;
    }
    if !sv_battle_object::is_active(id) {
        C_ID[e] = 0;
        return;
    }
    let ib = sv_battle_object::module_accessor(id);
    if ib.is_null() {
        C_ID[e] = 0;
        return;
    }
    let st = StatusModule::status_kind(ib);
    if st == *ITEM_STATUS_KIND_HAVE {
        return;
    }
    let cp = PostureModule::pos(ib);
    let (cx, cy) = ((*cp).x, (*cp).y);
    // la caisse touche quelqu'un (lancée) ou se fait frapper : explosion à cette image même (vidéo : elle explosait
    // après le contact, en retombant au sol)
    let mut boum = DamageModule::damage(ib, 0) > 0.0
        || AttackModule::is_infliction_status(ib, *COLLISION_KIND_MASK_HIT | *COLLISION_KIND_MASK_SHIELD)
        || AttackModule::is_infliction(ib, *COLLISION_KIND_MASK_HIT | *COLLISION_KIND_MASK_SHIELD);
    if (st == *ITEM_STATUS_KIND_THROW || st == *ITEM_STATUS_KIND_THROWN)
        && GroundModule::is_touch(ib, *GROUND_TOUCH_FLAG_ALL as u32) {
        boum = true;
    }
    if !boum {
        let moi = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID);
        let n = smash::app::Fighter::get_fighter_entry_count();
        for i in 0..n.min(8) {
            if i == moi {
                continue;
            }
            let fid = smash::app::Fighter::get_id_from_entry_id(i);
            if !sv_battle_object::is_active(fid) {
                continue;
            }
            let fb = sv_battle_object::module_accessor(fid);
            if fb.is_null() {
                continue;
            }
            let p = PostureModule::pos(fb);
            // le perso (pieds à p.y, ~16 de haut) recouvre la caisse
            if ((*p).x - cx).abs() < C_RAYON && (*p).y < cy + 9.0 && (*p).y + 16.0 > cy - 1.0 {
                boum = true;
                break;
            }
        }
    }
    if !boum {
        return;
    }
    C_ID[e] = 0;
    let mgr = item_manager();
    if !mgr.is_null() {
        smash::app::lua_bind::ItemManager::remove_item_from_id(mgr, id);
    }
    let pos = Vector3f { x: cx, y: cy + 4.0, z: 0.0 };
    let z = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    EffectModule::req(boma, Hash40::new("sys_bomb_a"), &pos, &z, 1.3, 0, 0, false, 0);
    PLAY_SE(fighter, Hash40::new("se_common_bomb_l"));
    // coup d'explosion (REX ~10-19 %) posé sur « top » de Chief, décalé jusqu'à la caisse (z = vers l'avant)
    let fp = PostureModule::pos(boma);
    let lr = PostureModule::lr(boma);
    let dz = (cx - (*fp).x) * lr;
    let dy = cy + 4.0 - (*fp).y;
    ATTACK(fighter, 7, 0, Hash40::new("top"), 16.0, 361, 70, 0, 50, 12.0, 0.0, dy, dz, None, None, None, 0.6, 1.0,
           *ATTACK_SETOFF_KIND_OFF, *ATTACK_LR_CHECK_POS, false, 0, 0.0, 0, false, false, false, false, true,
           *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false,
           Hash40::new("collision_attr_fire"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_FIRE, *ATTACK_REGION_BOMB);
    C_BOUM[e] = 3;
}

// ------------------------------------------------------------------ up B : Gravity Lift
const L_POSE: f32 = 13.0;
const L_DUREE: i32 = 600;           // « reste en place un bon moment » : 10 s
const L_VOL: i32 = 14;              // lancé : images jusqu'au sol
const L_DEVANT: f32 = 11.0;         // vidéo de REX : posé ~0,6 fois sa taille devant lui
const L_LARGE: f32 = 5.0;           // demi-largeur de la colonne (vidéo : ~ la moitié de sa taille)
const L_HAUT: f32 = 110.0;          // zone où il est « dans la colonne » (vidéo de REX : jusqu'en haut de l'écran)
const L_HALO: f32 = 45.0;           // hauteur du halo blanc (vidéo de l'autre Chief : ~2 fois sa taille)
const L_ELAN: f32 = 3.4;            // vidéo de REX : ~5 fois sa taille ; 3,9 l'envoyait hors de l'écran (vidéo v9)
static mut L_ETAT: [u8; 8] = [0; 8];    // 0 rien, 1 en vol, 2 posé
static mut L_POS: [(f32, f32); 8] = [(0.0, 0.0); 8];
static mut L_DE: [(f32, f32); 8] = [(0.0, 0.0); 8];
static mut L_T: [i32; 8] = [0; 8];
static mut L_RECH: [i32; 8] = [0; 8];
static mut L_ELAN_ATT: [bool; 8] = [false; 8];
/// armé quand Chief est hors de la colonne : il faut y entrer pour être propulsé (pas en le posant, pas en
/// retombant dedans)
static mut L_ARME: [bool; 8] = [false; 8];
static mut L_FAISCEAU: [u32; 8] = [0; 8];
static mut L_FAISCEAU_T: [i32; 8] = [0; 8];
static mut L_TIRS: [u8; 8] = [0; 8];

unsafe fn faisceau_fin(boma: *mut BattleObjectModuleAccessor) {
    let e = slot(boma);
    if L_FAISCEAU[e] != 0 {
        EffectModule::kill(boma, L_FAISCEAU[e], false, true);
        L_FAISCEAU[e] = 0;
    }
}
static mut HI_FAIT: [bool; 8] = [false; 8];
static mut HI_VOL: [i32; 8] = [L_VOL; 8];
static mut HI_PAS_PRET: [bool; 8] = [false; 8];

unsafe extern "C" fn hi_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    let e = slot(fighter.module_accessor);
    HI_FAIT[e] = false;
    HI_PAS_PRET[e] = L_ETAT[e] != 0;
    if HI_PAS_PRET[e] {
        situation(fighter, "special_air_hi_hang", "special_air_hi_cut", true);
    } else {
        situation(fighter, "special_hi_start", "special_air_hi_start", true);
    }
    fighter.sub_shift_status_main(L2CValue::Ptr(hi_loop as *const () as _))
}

unsafe extern "C" fn hi_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    if HI_PAS_PRET[e] {
        situation(fighter, "special_air_hi_hang", "special_air_hi_cut", false);
    } else {
        situation(fighter, "special_hi_start", "special_air_hi_start", false);
    }
    if !HI_PAS_PRET[e] && !HI_FAIT[e] && MotionModule::frame(boma) >= L_POSE {
        HI_FAIT[e] = true;
        let p = PostureModule::pos(boma);
        let (x, y) = ((*p).x, (*p).y);
        let lr = PostureModule::lr(boma);
        let mut hit = Vector2f { x: 0.0, y: 0.0 };
        if StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND {
            // jeté au sol devant lui (vidéo de REX) ; au bord (pas de sol devant) : à ses pieds
            let de = Vector2f { x: x + L_DEVANT * lr, y: y + 8.0 };
            let dir = Vector2f { x: 0.0, y: -20.0 };
            let (px, py) = if GroundModule::ray_check_hit_pos(boma, &de, &dir, &mut hit, true) { (hit.x, hit.y) } else { (x, y) };
            L_DE[e] = (x + 4.0 * lr, y + 10.0);
            L_POS[e] = (px, py);
            L_ETAT[e] = 1;
            L_T[e] = 6;
            HI_VOL[e] = 6;
        } else {
            // lancé en diagonale vers le bas : il se pose là où la diagonale touche le sol
            let de = Vector2f { x: x + 3.0 * lr, y: y + 6.0 };
            let dir = Vector2f { x: 60.0 * lr, y: -90.0 };
            if GroundModule::ray_check_hit_pos(boma, &de, &dir, &mut hit, true) {
                L_DE[e] = (de.x, de.y);
                L_POS[e] = (hit.x, hit.y);
                L_ETAT[e] = 1;
                L_T[e] = L_VOL;
                HI_VOL[e] = L_VOL;
            }
        }
    }
    finir(fighter);
    0.into()
}

unsafe fn eff_monde(boma: *mut BattleObjectModuleAccessor, eff: &str, x: f32, y: f32, rx: f32, sc: f32, c: Option<(f32, f32, f32)>) {
    let p = Vector3f { x, y, z: 0.0 };
    let r = Vector3f { x: rx, y: 0.0, z: 0.0 };
    let h = EffectModule::req(boma, Hash40::new(eff), &p, &r, sc, 0, 0, false, 0) as u32;
    if let (true, Some(c)) = (h != 0, c) {
        EffectModule::set_rgb(boma, h, c.0, c.1, c.2);
    }
}

/// le modèle de REX de l'ascenseur est mis sur la C4 de Snake (dossier du costume c140 seulement).
///
/// Vidéos v8 à v10 : l'ascenseur s'affichait toujours au centre du terrain (l'origine du monde), la colonne et la zone
/// de propulsion au bon endroit, devant Chief. Code du jeu relu (statuts de la C4, lua2cpp_snake ; scripts du bas B
/// de Snake) ; deux causes possibles, je ne peux pas savoir laquelle sans tester : les deux sont corrigées.
/// - créée, la C4 est au statut START, tenue par une « contrainte de modèle » sur la main de Snake ; le jeu la libère
///   avec ArticleModule::shoot (script du bas B : `is_constraint_article` puis `shoot`) ou `detach_constraint`. Les
///   versions d'avant coupaient le lien (unlink_all) sans retirer la contrainte ;
/// - au statut ESTABLISH_GROUND (utilisé avant), la C4 se replace toute seule à chaque image sur la position de son
///   os « rot » (`joint_global_position(rot)` puis `PostureModule::set_pos`), cherche un perso où se coller (SEARCH)
///   puis le sol sous elle ; notre modèle n'avait pas d'os « rot » : position lue (0, 0) = l'origine du monde.
/// Maintenant : elle reste au statut START (aucune logique dans ce statut : ni déplacement, ni recherche, ni effet,
/// ni son), libérée de la main comme le fait le jeu, et replacée à chaque image au point de pose (par l'objet et par
/// ArticleModule). Le modèle a aussi les os « rot » et « top » (objets_ult.py) : si un statut de la C4 les lit, il
/// trouve la position du modèle et plus l'origine.
/// Jamais déclenchée (bas B remplacé), retirée à la fin (sa minuterie de 1600 images ne peut pas arriver : 600)
unsafe fn c4_boma(boma: *mut BattleObjectModuleAccessor) -> Option<*mut BattleObjectModuleAccessor> {
    if !ArticleModule::is_exist(boma, *FIGHTER_SNAKE_GENERATE_ARTICLE_C4) {
        return None;
    }
    let a = ArticleModule::get_article(boma, *FIGHTER_SNAKE_GENERATE_ARTICLE_C4);
    if a.is_null() {
        return None;
    }
    let id = smash::app::lua_bind::Article::get_battle_object_id(a) as u32;
    let b = sv_battle_object::module_accessor(id);
    if b.is_null() { None } else { Some(b) }
}

unsafe fn c4_poser(fighter: &mut L2CFighterCommon, x: f32, y: f32) {
    let boma = fighter.module_accessor;
    if !ArticleModule::is_exist(boma, *FIGHTER_SNAKE_GENERATE_ARTICLE_C4) {
        ArticleModule::generate_article(boma, *FIGHTER_SNAKE_GENERATE_ARTICLE_C4, false, -1);
    }
    L_TIRS[slot(boma)] = 0;
    c4_tenir(fighter, x, y);
}

/// chaque image tant qu'il est posé : libérée de la main, au point de pose, effets / sons de C4 coupés.
/// Renvoie la position où le modèle est vraiment dessiné (son os « rot »), si on peut la lire
unsafe fn c4_tenir(fighter: &mut L2CFighterCommon, x: f32, y: f32) -> Option<(f32, f32)> {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    if !ArticleModule::is_exist(boma, *FIGHTER_SNAKE_GENERATE_ARTICLE_C4) {
        return None;
    }
    // la méthode du jeu (3 essais au plus)
    if L_TIRS[e] < 3 && FighterSpecializer_Snake::is_constraint_article(
        fighter.battle_object as *mut Fighter, *FIGHTER_SNAKE_GENERATE_ARTICLE_C4, ArticleOperationTarget(*ARTICLE_OPE_TARGET_LAST)) {
        L_TIRS[e] += 1;
        ArticleModule::shoot(boma, *FIGHTER_SNAKE_GENERATE_ARTICLE_C4, ArticleOperationTarget(*ARTICLE_OPE_TARGET_ALL), false);
    }
    let mut vu = None;
    if let Some(cb) = c4_boma(boma) {
        // et directement sur l'objet : la contrainte d'abord, le lien ensuite (dans cet ordre)
        if LinkModule::is_model_constraint(cb) {
            LinkModule::remove_model_constraint(cb, true);
            LinkModule::unlink_all(cb);
        }
        PostureModule::set_pos(cb, &Vector3f { x, y, z: 0.0 });
        EffectModule::kill_all(cb, 0, true, false);
        SoundModule::stop_all_sound(cb);
        // os « rot » : sur l'origine du modèle (« ChiefRoot » ne l'est pas : os de référence décalé de 5 vers le bas)
        let mut v = Vector3f { x: f32::NAN, y: f32::NAN, z: 0.0 };
        ModelModule::joint_global_position(cb, Hash40::new("rot"), &mut v, true);
        if v.x.is_finite() && v.y.is_finite() {
            vu = Some((v.x, v.y));
        }
    }
    ArticleModule::set_pos(boma, *FIGHTER_SNAKE_GENERATE_ARTICLE_C4, Vector3f { x, y, z: 0.0 });
    vu
}

unsafe fn c4_retirer(boma: *mut BattleObjectModuleAccessor) {
    if ArticleModule::is_exist(boma, *FIGHTER_SNAKE_GENERATE_ARTICLE_C4) {
        ArticleModule::remove_exist(boma, *FIGHTER_SNAKE_GENERATE_ARTICLE_C4, ArticleOperationTarget(*ARTICLE_OPE_TARGET_ALL));
    }
}

/// l'ascenseur : vol jusqu'au sol, colonne de lumière, propulsion de Chief quand il marche dedans
unsafe fn ascenseur(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let e = slot(boma);
    if L_RECH[e] > 0 {
        L_RECH[e] -= 1;
    }
    match L_ETAT[e] {
        1 => {
            // la capsule part de sa main vers le point de pose (éclat violet le long du trajet)
            L_T[e] -= 1;
            let t = 1.0 - (L_T[e].max(0) as f32) / (HI_VOL[e].max(1) as f32);
            let x = L_DE[e].0 + (L_POS[e].0 - L_DE[e].0) * t;
            let y = L_DE[e].1 + (L_POS[e].1 - L_DE[e].1) * t;
            eff_monde(boma, "sys_smash_flash_s", x, y, 0.0, 0.5, Some(VIOLET));
            if L_T[e] <= 0 {
                L_ETAT[e] = 2;
                L_T[e] = L_DUREE;
                L_FAISCEAU_T[e] = 6;
                let (bx, by) = L_POS[e];
                c4_poser(fighter, bx, by);
                eff_monde(boma, "sys_smash_flash", bx, by + 1.0, 0.0, 0.8, Some(VIOLET));
                // posé sur lui (au bord) : pas propulsé tant qu'il n'en est pas sorti
                let p = PostureModule::pos(boma);
                L_ARME[e] = ((*p).x - bx).abs() >= L_LARGE;
            }
        }
        2 => {
            L_T[e] -= 1;
            if L_T[e] <= 0 {
                L_ETAT[e] = 0;
                c4_retirer(boma);
                faisceau_fin(boma);
                return;
            }
            let (bx, by) = L_POS[e];
            let vu = c4_tenir(fighter, bx, by);
            // contrôle (rien si tout va bien) : si le modèle est dessiné ailleurs qu'au point de pose, éclat rouge là
            // où il est, pour le voir sur une vidéo
            if let Some((vx, vy)) = vu {
                if L_T[e] < L_DUREE - 6 && L_T[e] % 20 == 0 && ((vx - bx).abs() > 2.0 || (vy - by).abs() > 2.0) {
                    eff_monde(boma, "sys_smash_flash_s", vx, vy + 1.0, 0.0, 0.8, Some((1.0, 0.1, 0.1)));
                }
            }
            // faisceau (demandé : l'effet bleu de la photo, « comme dans la vidéo » de l'autre Chief : pointe cyan qui
            // monte de l'ascenseur dans un halo blanc) : le jet de la ceinture-fusée, retourné vers le haut (dans le
            // jeu il part vers le bas) et étiré en hauteur ; un seul, recréé s'il s'éteint (6 images au moins entre deux)
            L_FAISCEAU_T[e] += 1;
            if (L_FAISCEAU[e] == 0 || !EffectModule::is_exist_effect(boma, L_FAISCEAU[e])) && L_FAISCEAU_T[e] >= 6 {
                L_FAISCEAU_T[e] = 0;
                let p = Vector3f { x: bx, y: by + 1.0, z: 0.0 };
                let r = Vector3f { x: 180.0, y: 0.0, z: 0.0 };
                L_FAISCEAU[e] = EffectModule::req(boma, Hash40::new(crate::armes::JET), &p, &r, 1.0, 0, 0, false, 0) as u32;
                if L_FAISCEAU[e] != 0 {
                    EffectModule::set_scale(boma, L_FAISCEAU[e], &Vector3f { x: 1.4, y: 2.6, z: 1.4 });
                }
            }
            // halo : traits lumineux blanc bleuté de part et d'autre (plus léger qu'en v10)
            if L_T[e] % 6 == 0 {
                for (k, dx) in [-2.5f32, 2.5].iter().enumerate() {
                    let mut h = 5.0 + 7.0 * k as f32;
                    while h < L_HALO {
                        eff_monde(boma, "sys_attack_line", bx + dx, by + h, -90.0, 1.6, Some((0.85, 0.95, 1.0)));
                        h += 14.0;
                    }
                }
            }
            if L_T[e] % 10 == 0 {
                eff_monde(boma, "sys_smash_flash_s", bx, by + 2.0, 0.0, 1.0, Some((0.85, 0.95, 1.0)));
            }
            // Chief entre dans la colonne au sol (en marchant, en courant...) : accroupi puis propulsé (vidéo de REX)
            let p = PostureModule::pos(boma);
            let (x, y) = ((*p).x, (*p).y);
            let dedans = (x - bx).abs() < L_LARGE && y >= by - 2.0 && y < by + L_HAUT;
            if !dedans {
                L_ARME[e] = true;
            }
            let sol = StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND;
            if dedans && sol && L_ARME[e] && L_RECH[e] == 0 && (y - by).abs() < 3.0 {
                let st = StatusModule::status_kind(boma);
                let libre = [*FIGHTER_STATUS_KIND_WAIT, *FIGHTER_STATUS_KIND_WALK, *FIGHTER_STATUS_KIND_DASH,
                             *FIGHTER_STATUS_KIND_RUN, *FIGHTER_STATUS_KIND_RUN_BRAKE, *FIGHTER_STATUS_KIND_TURN,
                             *FIGHTER_STATUS_KIND_TURN_DASH, *FIGHTER_STATUS_KIND_TURN_RUN, *FIGHTER_STATUS_KIND_SQUAT,
                             *FIGHTER_STATUS_KIND_SQUAT_WAIT, *FIGHTER_STATUS_KIND_SQUAT_RV,
                             *FIGHTER_STATUS_KIND_LANDING, *FIGHTER_STATUS_KIND_LANDING_LIGHT];
                if libre.contains(&st) {
                    L_ARME[e] = false;
                    L_RECH[e] = 40;
                    L_ELAN_ATT[e] = true;
                    StatusModule::change_status_request_from_script(boma, *FIGHTER_STATUS_KIND_JUMP_SQUAT, true);
                    eff_monde(boma, "sys_smash_flash", bx, by + 3.0, 0.0, 1.0, Some((0.85, 0.95, 1.0)));
                    PLAY_SE(fighter, Hash40::new("se_snake_special_h01"));
                }
            }
        }
        _ => {}
    }
    // élan appliqué une fois en l'air (après l'accroupissement)
    if L_ELAN_ATT[e] && StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR
        && StatusModule::status_kind(boma) != *FIGHTER_STATUS_KIND_JUMP_SQUAT {
        L_ELAN_ATT[e] = false;
        sv_kinetic_energy!(set_speed, fighter, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY, L_ELAN);
    } else if L_ELAN_ATT[e] && L_RECH[e] < 25 {
        L_ELAN_ATT[e] = false;
    }
}

pub unsafe fn chaque_image(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    // début de match : rien ne reste du match précédent (états gardés par n° de joueur)
    if StatusModule::status_kind(boma) == *FIGHTER_STATUS_KIND_ENTRY {
        let e = slot(boma);
        G_ID[e] = 0;
        G_ETAT[e] = 0;
        C_ID[e] = 0;
        L_ETAT[e] = 0;
        L_ELAN_ATT[e] = false;
        L_ARME[e] = false;
        C_BOUM[e] = 0;
        c4_retirer(boma);
        L_FAISCEAU[e] = 0;
        L_TIRS[e] = 0;
        return;
    }
    lueur_grenade(fighter.module_accessor);
    coil_contact(fighter);
    ascenseur(fighter);
}

// ------------------------------------------------------------------ scripts (effets / sons aux images de REX)
unsafe extern "C" fn rien(_agent: &mut L2CAgentBase) {}

unsafe extern "C" fn son_grenade(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 3.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_snake_special_s01"));
    }
    frame(agent.lua_state_agent, 19.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_snake_special_s02"));
    }
}

unsafe extern "C" fn eff_grenade(agent: &mut L2CAgentBase) {
    // flèche verte de visée (REX : effet 0x1FB0008) : éclat vert devant lui pendant la visée
    frame(agent.lua_state_agent, 4.0);
    if is_excute(agent) {
        EFFECT_FOLLOW(agent, Hash40::new("sys_smash_flash_s"), Hash40::new("haver"), 0, 0, 0, 0, 0, 0, 0.5, true);
        LAST_EFFECT_SET_COLOR(agent, BLEU.0, BLEU.1, BLEU.2);
    }
    frame(agent.lua_state_agent, 10.0);
    if is_excute(agent) {
        EFFECT(agent, Hash40::new("sys_attack_line"), Hash40::new("top"), 0, 14, 6, -35, 0, 0, 0.5, 0, 0, 0, 0, 0, 0, true);
        LAST_EFFECT_SET_COLOR(agent, 0.3, 1.0, 0.3);
    }
}

unsafe extern "C" fn son_coil(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 9.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_snake_special_l01"));
    }
    frame(agent.lua_state_agent, 20.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_snake_special_l02"));
    }
}

unsafe extern "C" fn son_lift(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 1.0);
    if is_excute(agent) {
        PLAY_SE(agent, Hash40::new("se_snake_special_h01"));
    }
}

unsafe extern "C" fn eff_lift(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 13.0);
    if is_excute(agent) {
        EFFECT_FOLLOW(agent, Hash40::new("sys_smash_flash"), Hash40::new("haver"), 0, 0, 0, 0, 0, 0, 0.6, true);
        LAST_EFFECT_SET_COLOR(agent, VIOLET.0, VIOLET.1, VIOLET.2);
    }
}

pub fn install(a: &mut Agent) {
    // scripts de Snake (Nikita, Cypher, C4) remplacés pour ces animations
    for m in ["specialsstart", "specialairsstart", "specialhistart", "specialairhistart", "specialairhicut",
              "specialairhihang", "speciallwstart", "specialairlwstart"] {
        for k in ["game_", "effect_", "sound_", "expression_"] {
            let n: &'static str = Box::leak(format!("{}{}", k, m).into_boxed_str());
            let f: unsafe extern "C" fn(&mut L2CAgentBase) = match (k, m) {
                ("sound_", "specialsstart") | ("sound_", "specialairsstart") => son_grenade,
                ("effect_", "specialsstart") | ("effect_", "specialairsstart") => eff_grenade,
                ("sound_", "speciallwstart") | ("sound_", "specialairlwstart") => son_coil,
                ("sound_", "specialhistart") | ("sound_", "specialairhistart") => son_lift,
                ("effect_", "specialhistart") | ("effect_", "specialairhistart") => eff_lift,
                _ => rien,
            };
            a.acmd(n, f, Priority::Low);
        }
    }
    for st in [*FIGHTER_STATUS_KIND_SPECIAL_S, *FIGHTER_STATUS_KIND_SPECIAL_HI, *FIGHTER_STATUS_KIND_SPECIAL_LW] {
        a.status(Init, st, rien_status);
        a.status(Exec, st, rien_status);
    }
    a.status(Main, *FIGHTER_STATUS_KIND_SPECIAL_S, s_main);
    a.status(End, *FIGHTER_STATUS_KIND_SPECIAL_S, s_end);
    a.status(Main, *FIGHTER_STATUS_KIND_SPECIAL_HI, hi_main);
    a.status(End, *FIGHTER_STATUS_KIND_SPECIAL_HI, rien_status);
    a.status(Main, *FIGHTER_STATUS_KIND_SPECIAL_LW, lw_main);
    // sécurité : si le jeu passait au bas B « déclencher la C4 » de Snake (C4 = l'ascenseur posé), c'est la Fusion
    // Coil qui se fait, rien n'explose
    a.status(Init, *FIGHTER_SNAKE_STATUS_KIND_SPECIAL_LW_EXPLODING, rien_status);
    a.status(Exec, *FIGHTER_SNAKE_STATUS_KIND_SPECIAL_LW_EXPLODING, rien_status);
    a.status(End, *FIGHTER_SNAKE_STATUS_KIND_SPECIAL_LW_EXPLODING, rien_status);
    a.status(Main, *FIGHTER_SNAKE_STATUS_KIND_SPECIAL_LW_EXPLODING, lw_main);
    a.status(End, *FIGHTER_STATUS_KIND_SPECIAL_LW, rien_status);
}
