//! Dump d'une animation .nuanmb : noms des os, et par frame translation + rotation des os demandés.
//! usage : animdump <fichier.nuanmb> [os1,os2,...]
use ssbh_data::anim_data::{AnimData, GroupType, TrackValues};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let anim = AnimData::from_file(&a[1]).expect("anim");
    println!("final_frame_index {}", anim.final_frame_index);
    let voulus: Vec<String> = a.get(2).map(|s| s.split(',').map(|x| x.to_lowercase()).collect()).unwrap_or_default();
    for g in &anim.groups {
        if g.group_type != GroupType::Transform { continue; }
        let noms: Vec<&str> = g.nodes.iter().map(|n| n.name.as_str()).collect();
        println!("os ({}): {}", noms.len(), noms.join(" "));
        for n in &g.nodes {
            if !voulus.iter().any(|v| n.name.to_lowercase() == *v) { continue; }
            for t in &n.tracks {
                if let TrackValues::Transform(v) = &t.values {
                    println!("== {} ({} valeurs)", n.name, v.len());
                    for (i, x) in v.iter().enumerate() {
                        println!("{} {:.2} {:.2} {:.2} | {:.3} {:.3} {:.3} {:.3}", i,
                            x.translation.x, x.translation.y, x.translation.z,
                            x.rotation.x, x.rotation.y, x.rotation.z, x.rotation.w);
                    }
                }
            }
        }
    }
}
