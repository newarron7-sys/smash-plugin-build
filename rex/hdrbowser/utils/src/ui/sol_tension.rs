//! Jauge de Tension de Sol Badguy (copie de la jauge du Potemkin, panes « *_sol »).
//! Pilotée par le plugin « sol_badguy » via UiManager__set_sol_tension_info.
//! Animée exactement comme celle du Potemkin (mêmes timings que ses fichiers .bflan).
use super::*;

#[derive(Default, Copy, Clone)]
pub struct SolTension {
    root: u64,
    meter: u64,
    gear1: u64,
    gear2: u64,
    fire1: [u64; 4],
    fire2: [u64; 4],
    full_width: f32,
    height: f32,
    gear_rot: f32,
    percent: f32,
    /// 0 = normal, 1 = Tension pleine, 2 = Dragon Install, 3 = épuisé
    state: i32,
    frame: u32,
    is_enabled: bool,
}

/// nn::ui2d::Pane : rotation (degrés) à +0x3C, drapeaux à +0x58 (bit 4 = matrice à recalculer)
fn set_rotation_z(pane: u64, deg: f32) {
    if !is_pane_valid(pane) {
        return;
    }
    unsafe {
        let internal = *(pane as *const u64);
        *(internal as *mut f32).add(0x44 / 4) = deg;
        *(internal as *mut u8).add(0x58) |= 0x10;
    }
}

/// nn::ui2d::Pane : échelle X à +0x48
fn set_scale_x(pane: u64, x: f32) {
    if !is_pane_valid(pane) {
        return;
    }
    unsafe {
        let internal = *(pane as *const u64);
        *(internal as *mut f32).add(0x48 / 4) = x;
        *(internal as *mut u8).add(0x58) |= 0x10;
    }
}

fn set_color(pane: u64, c: [f32; 4]) {
    if is_pane_valid(pane) {
        set_vertex_colors(pane, c, c, c, c);
    }
}

impl SolTension {
    pub fn new(layout_data: u64) -> Self {
        let get = |n: &str| get_pane_from_layout(layout_data, n).unwrap_or(0);
        let root = get("sol_tension\0");
        let meter = get("tension_meter_sol\0");
        let (w, h) = if is_pane_valid(meter) { get_width_height(meter) } else { (240.0, 25.0) };
        Self {
            root,
            meter,
            gear1: get("gear1_sol\0"),
            gear2: get("gear2_sol\0"),
            fire1: [get("fire1_sol_0\0"), get("fire1_sol_1\0"), get("fire1_sol_2\0"), get("fire1_sol_3\0")],
            fire2: [get("fire2_sol_0\0"), get("fire2_sol_1\0"), get("fire2_sol_2\0"), get("fire2_sol_3\0")],
            full_width: w,
            height: h,
            gear_rot: 0.0,
            percent: 0.0,
            state: 0,
            frame: 0,
            is_enabled: false,
        }
    }

    pub fn set_info(&mut self, percent: f32, state: i32) {
        self.percent = percent.clamp(0.0, 1.0);
        self.state = state;
    }
}

impl UiObject for SolTension {
    fn update(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        let f = self.frame as f32;
        set_pane_visible(self.root, true);

        // barre : remplie en étirant la largeur (échelle X), comme l'animation « tension_meter » du Potemkin
        let p = self.percent.max(0.001);
        set_scale_x(self.meter, p);
        let bar = match self.state {
            2 => { let t = (f * 0.25).sin() * 0.5 + 0.5; [1.0, 0.15 + 0.25 * t, 0.05, 1.0] }
            3 => [0.5, 0.5, 0.5, 1.0],
            _ => [1.0, 1.0, 1.0, 1.0],
        };
        set_color(self.meter, bar);

        // engrenages : 360° en 100 images (animations gear1 / gear2 du Potemkin)
        self.gear_rot = (self.gear_rot + 3.6) % 360.0;
        set_rotation_z(self.gear1, self.gear_rot);
        set_rotation_z(self.gear2, self.gear_rot);

        // flammes : 4 images qui changent toutes les 3 frames, boucle de 21 frames
        // (animations fire1 / fire2 du Potemkin : 0,1,2,3,0,1,3)
        let idx = match (self.frame % 21) / 3 {
            0 => 0, 1 => 1, 2 => 2, 3 => 3, 4 => 0, 5 => 1, _ => 3,
        };
        for k in 0..4 {
            if is_pane_valid(self.fire1[k]) { set_pane_visible(self.fire1[k], k == idx); }
            if is_pane_valid(self.fire2[k]) { set_pane_visible(self.fire2[k], k == idx); }
        }
    }

    fn is_valid(&self) -> bool {
        is_pane_valid(self.root) && is_pane_valid(self.meter)
    }

    fn set_enable(&mut self, enable: bool) {
        if !enable && self.is_valid() {
            set_pane_visible(self.root, false);
        }
        self.is_enabled = enable;
    }

    fn is_enabled(&self) -> bool {
        self.is_enabled
    }
}
