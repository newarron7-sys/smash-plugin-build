#![feature(proc_macro_hygiene)]

//! Wario Land de HDR (case WarioWare, id 0x8f) fixe, pour Smash Ultimate 13.0.4.
//! Repris de HewDraw Remix (fighters/common/src/function_hooks/stage_hazards.rs : `stub` et `area_manager_process`,
//! offsets de la version 13.0.4) ; HewDraw Remix est sous licence AGPL-3.0. Sur la forme normale du terrain (alt 0,
//! libstage_alts.nro) : le code de WarioWare (micro-jeux, changements de décor) ne tourne pas et ses zones (type 0x1b)
//! sont vidées. La forme s01 (WarioWare d'origine) garde son code (si libstage_alts.nro est là).

extern "C" {
    #[link_name = "_ZN3app5stage12get_stage_idEv"]
    fn get_stage_id() -> u32;
}

/// v2 (16/11, joueur : « toujours pareil ») : la v1 importait get_current_stage_alt de libstage_alts.nro ; si ce
/// plugin n'est pas chargé, la v1 ne se charge pas du tout. Recherche à l'exécution : sans libstage_alts, forme 0.
static mut ALT: Option<extern "C" fn() -> usize> = None;
static mut ALT_CHERCHE: bool = false;

unsafe fn forme() -> usize {
    if !ALT_CHERCHE {
        ALT_CHERCHE = true;
        let mut addr = 0usize;
        let nom = b"get_current_stage_alt\0";
        if skyline::nn::ro::LookupSymbol(&mut addr, nom.as_ptr() as _) == 0 && addr != 0 {
            ALT = Some(core::mem::transmute(addr));
        }
    }
    match ALT {
        Some(f) => f(),
        None => 0,
    }
}

unsafe fn wario_land() -> bool {
    get_stage_id() == 0x8f && forme() == 0
}

#[skyline::hook(offset = 0x30F6E00)]
unsafe fn stub(arg: u64) {
    if wario_land() {
        return;
    }
    call_original!(arg);
}

#[skyline::hook(offset = 0x5209c0)]
unsafe fn area_manager_process(manager: *const u64) {
    let mut start = *manager.add(1);
    let end = *manager.add(2);
    while start != end {
        let current = *(start as *const u64);
        if *(current as *mut u8).add(0x20) == 0x1b && wario_land() {
            *(current as *mut bool).add(0x21) = false;
            for k in 0..8 {
                *((current + 0x40) as *mut f32).add(k) = 0.0;
            }
        }
        start += 0x8;
    }
    call_original!(manager)
}

#[skyline::main(name = "wario_land_fixe")]
pub fn main() {
    skyline::install_hooks!(stub, area_manager_process);
}
