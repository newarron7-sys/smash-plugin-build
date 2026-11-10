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
            let mut v = Vec::new();
            for (k, nom) in ["game_specialnhit2", "game_specialnhit3", "game_specialnhit4"].iter().enumerate() {
                let off = crate::speciaux::SEG[k];
                v.push(Ev { f: off.max(0.0), rate: 1.0, ..crate::E });
                if let Some(m) = crate::find(nom) {
                    // demandé (le combo s'arrêtait après le poing : l'adversaire partait trop loin) : poing et 1er
                    // pied attirent l'adversaire vers le coup (angle 366) au lieu de l'éjecter ; dégâts de REX gardés,
                    // le dernier pied éjecte comme dans REX
                    v.extend(m.evs.iter().map(|e| {
                        let mut e2 = Ev { f: e.f + off, ..*e };
                        if k < 2 && !e.clear && e.body < 0 && e.rate <= 0.0 {
                            e2.angle = 366;
                            e2.kbg = 100;
                            e2.fkb = 40;
                            e2.bkb = 0;
                        }
                        e2
                    }));
                }
            }
            v.push(Ev { f: crate::speciaux::SEG[3], rate: 1.0, ..crate::E });
            v.sort_by(|x, y| x.f.partial_cmp(&y.f).unwrap());
            v
        }
        "game_attacklw4" => evs.iter().map(|e| if e.rate > 0.0 { *e } else { Ev { f: e.f + 4.0, ..*e } }).collect(),
        _ => evs.to_vec(),
    }
}
