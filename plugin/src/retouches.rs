//! Retouches demandées sur les coups (à partir des hitbox de REX, sans en recopier les valeurs ici) :
//! - jab 3 : 4 petits coups (le coup faible de REX) puis le coup final de REX, comme une rafale ;
//! - up smash : le coup faible de REX répété jusqu'au coup final de REX ;
//! - down smash : hitbox 8 images plus tard (elle touchait avant que les boules soient visibles).

use crate::Ev;

fn faible(evs: &[Ev]) -> Option<Ev> {
    evs.iter().copied().filter(|e| !e.clear && e.body < 0 && e.rate <= 0.0).min_by(|a, b| a.dmg.partial_cmp(&b.dmg).unwrap())
}
fn fort(evs: &[Ev]) -> Option<Ev> {
    evs.iter().copied().filter(|e| !e.clear && e.body < 0 && e.rate <= 0.0).max_by(|a, b| a.dmg.partial_cmp(&b.dmg).unwrap())
}
fn vide(f: f32) -> Ev {
    Ev { f, clear: true, ..crate::E }
}

pub fn evs(script: &str, evs: &[Ev]) -> Vec<Ev> {
    let autres: Vec<Ev> = evs.iter().copied().filter(|e| e.rate > 0.0 || e.body >= 0).collect();
    match script {
        "game_attack13" => {
            let (Some(a), Some(b)) = (faible(evs), fort(evs)) else { return evs.to_vec() };
            let mut v = autres;
            for f in [10.0, 13.0, 16.0, 19.0] {
                v.push(Ev { f, ..a });
                v.push(vide(f + 2.0));
            }
            v.push(Ev { f: 22.0, ..b });
            v.push(vide(31.0));
            v.sort_by(|x, y| x.f.partial_cmp(&y.f).unwrap());
            v
        }
        "game_attackhi4" => {
            let (Some(a), Some(b)) = (faible(evs), fort(evs)) else { return evs.to_vec() };
            let mut v = autres;
            let mut f = a.f;
            while f < b.f - 2.0 {
                v.push(Ev { f, ..a });
                v.push(vide(f + 2.0));
                f += 4.0;
            }
            v.push(Ev { f: b.f, ..b });
            v.push(vide(b.f + 4.0));
            v.sort_by(|x, y| x.f.partial_cmp(&y.f).unwrap());
            v
        }
        // B neutre : combo de REX (poing, pied, pied qui éjecte, pose) bout à bout dans une seule animation
        // (special_n_hit de Sonic) ; vitesse remise à 1 au début de chaque partie
        "game_specialnhit" => {
            // demandé : combo plus rapide, hitbox au moment où il frappe vraiment. Chaque partie un peu accélérée
            // (x1,25, sans les ralentis de REX), hitbox de REX actives des images 3 à 9 de la partie (au lieu de 5
            // à 8 seulement) pour couvrir le geste
            let mut v = Vec::new();
            for (k, nom) in ["game_specialnhit2", "game_specialnhit3", "game_specialnhit4"].iter().enumerate() {
                let off = crate::speciaux::SEG[k];
                v.push(Ev { f: off, rate: 1.25, ..crate::E });
                if let Some(m) = crate::find(nom) {
                    for e in m.evs.iter().filter(|e| !e.clear && e.body < 0 && e.rate <= 0.0) {
                        let mut e2 = Ev { f: off + 3.0, ..*e };
                        // poing et 1er pied : l'adversaire est attiré vers le coup (angle 366) pour que tout
                        // s'enchaîne ; le dernier pied éjecte comme dans REX
                        if k < 2 {
                            e2.angle = 366;
                            e2.kbg = 100;
                            e2.fkb = 40;
                            e2.bkb = 0;
                        }
                        v.push(e2);
                    }
                }
                v.push(Ev { f: off + 10.0, clear: true, ..crate::E });
            }
            v.push(Ev { f: crate::speciaux::SEG[3], rate: 1.25, ..crate::E });
            v.sort_by(|x, y| x.f.partial_cmp(&y.f).unwrap());
            v
        }
        "game_attacklw4" => evs.iter().map(|e| if e.rate > 0.0 { *e } else { Ev { f: e.f + 4.0, ..*e } }).collect(),
        _ => evs.to_vec(),
    }
}
