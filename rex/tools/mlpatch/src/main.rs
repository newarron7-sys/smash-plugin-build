//! motion_list.bin + motion_patch.yaml (HDR) -> nouveau motion_list.bin (même code que runtime-motion-patcher).
//! usage : mlpatch <base.bin> <patch.yaml> <sortie.bin> <avant.yaml> <apres.yaml>
use std::collections::BTreeMap;
use hash40::Hash40;
use motion_patch::{MotionMapExt, MotionPatch};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut ml = motion_lib::open(&a[1]).expect("base");
    std::fs::write(&a[4], serde_yaml::to_string(&ml).unwrap()).unwrap();
    let patch: BTreeMap<Hash40, MotionPatch> =
        serde_yaml::from_str(&std::fs::read_to_string(&a[2]).unwrap()).expect("yaml");
    println!("motions avant : {}, entrees du patch : {}", ml.list.len(), patch.len());
    patch.apply(&mut ml);
    println!("motions apres : {}", ml.list.len());
    std::fs::write(&a[5], serde_yaml::to_string(&ml).unwrap()).unwrap();
    motion_lib::save(&a[3], &ml).expect("ecriture");
    // relecture de controle
    let r = motion_lib::open(&a[3]).expect("relecture");
    println!("relu : {} motions", r.list.len());
}
