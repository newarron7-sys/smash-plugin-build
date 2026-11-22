//! Diagnostic pour le mod Sol Badguy (StealthyCape131) sur la 13.0.4 : écrit sd:/sol_diag.txt avec les vérifications
//! que son plugin fait avant d'installer ses modifications du jeu (signature de version, instructions attendues).
use std::io::Write;

const SIG_A: [u64; 6] = [0xd00171481e230001, 0xd0017148bd4e0d02, 0xbd4599021e221821, 0xf9479d00b001e708, 0x1e3900221e220821, 0x14369d19320003e1];
const SIG_B: [u64; 6] = [0xf00171481e230001, 0xf0017148bd4e0d02, 0xbd4599021e221821, 0xf9479d00d001e708, 0x1e3900221e220821, 0x14369e2d320003e1];

unsafe fn rd64(base: usize, off: usize) -> u64 { std::ptr::read_unaligned((base + off) as *const u64) }
unsafe fn rd32(base: usize, off: usize) -> u32 { std::ptr::read_unaligned((base + off) as *const u32) }

#[skyline::main(name = "soldiag")]
pub fn main() {
    let mut s = String::new();
    unsafe {
        let t = skyline::hooks::getRegionAddress(skyline::hooks::Region::Text) as usize;
        let v: Vec<u64> = (0..6).map(|i| rd64(t, 0x1647180 + 8 * i)).collect();
        let ver = if v == SIG_A { "A (13.0.4 d'après les adresses)" } else if v == SIG_B { "B (13.0.5)" } else { "INCONNUE" };
        s += &format!("signature version a text+0x1647180 : {}\n  lu : {:x?}\n", ver, v);
        let a = rd32(t, 0x60bf38);
        let b = rd32(t, 0x60bf38 + 0xa8);
        s += &format!("text+0x60bf38 = {:08x} (attendu 2a1603e2) -> {}\n", a, if a == 0x2a1603e2 { "OK" } else { "DIFFERENT" });
        s += &format!("text+0x60bfe0 = {:08x} (attendu b9400e68) -> {}\n", b, if b == 0xb9400e68 { "OK" } else { "DIFFERENT" });
        let c = rd64(t, 0x4e53e0);
        s += &format!("text+0x4e53e0 = {:016x} (attendu f9400008f9401c00 ou trampoline d61f022058000051) -> {}\n", c,
            if c == 0xf9400008f9401c00 { "OK" } else if c == 0xd61f022058000051 { "DEJA DETOURNE (autre plugin)" } else { "DIFFERENT" });
        let m = std::path::Path::new("sd:/ultimate/mods").exists();
        s += &format!("dossier sd:/ultimate/mods : {}\n", m);
    }
    if let Ok(mut f) = std::fs::File::create("sd:/sol_diag.txt") {
        let _ = f.write_all(s.as_bytes());
    }
}
