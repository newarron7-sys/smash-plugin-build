use super::*;
use globals::*;

// FIREBREATH
unsafe extern "C" fn breath_move_main(weapon: &mut L2CWeaponCommon) -> L2CValue {
    let boma = weapon.boma();
    let owner_boma = &mut *sv_battle_object::module_accessor((WorkModule::get_int(boma, *WEAPON_INSTANCE_WORK_ID_INT_LINK_OWNER)) as u32);
    let is_fireball = WorkModule::get_float(owner_boma,*FIGHTER_KOOPA_STATUS_BREATH_WORK_FLOAT_GENE_ANGLE) > 360.0;
    if (!is_fireball) {
        return smashline::original_status(Main, weapon, *WEAPON_KOOPA_BREATH_STATUS_KIND_MOVE)(weapon);
    }
    else {
        WorkModule::set_customize_no(weapon.module_accessor, 1, 0);
        PostureModule::set_scale(weapon.module_accessor, 1.0, false);

        MotionModule::change_motion(weapon.module_accessor, Hash40::new("max"), 0.0, 1.0, false, 0.0, false, false);
        let param_life = 60; // valeur HDR (vl.prcxml) : les params du jeu de base font disparaître la boule tout de suite
        WorkModule::set_int(weapon.module_accessor, param_life, *WEAPON_INSTANCE_WORK_ID_INT_LIFE);
        WorkModule::set_int(weapon.module_accessor, param_life, *WEAPON_INSTANCE_WORK_ID_INT_INIT_LIFE);
        
        let param_hit_decrease = 60; // valeur HDR
        WorkModule::set_int(weapon.module_accessor, param_hit_decrease, *WEAPON_KOOPA_BREATH_INSTANCE_WORK_ID_INT_HIT_FRAME);

        KineticModule::enable_energy(weapon.module_accessor, *WEAPON_KINETIC_ENERGY_RESERVE_ID_NORMAL);
        let lr = PostureModule::lr(weapon.module_accessor);
        let param_speed = 4.0f32; // valeur HDR
        WorkModule::set_float(weapon.module_accessor, param_speed, *WEAPON_KOOPA_BREATH_INSTANCE_WORK_ID_FLOAT_SPEED_MUL);
        
        sv_kinetic_energy!(set_speed, weapon, WEAPON_KINETIC_ENERGY_RESERVE_ID_NORMAL, param_speed * lr, 0.0);
        sv_kinetic_energy!(set_stable_speed, weapon, WEAPON_KINETIC_ENERGY_RESERVE_ID_NORMAL, -1.0, -1.0);
        sv_kinetic_energy!(set_accel, weapon, WEAPON_KINETIC_ENERGY_RESERVE_ID_NORMAL, 0.0, 0.0);
        
        // Dark Bowser (c55) : la boule de feu ne dépend plus du motion « max » (absent du motion_list
        // du slot c55) : hitbox et effets posés ici au lieu de game_max / effect_max.
        boule_attaque(weapon, 18.0, 4.5);
        boule_effets(weapon);

        weapon.global_table[SUB_STATUS].assign(&L2CValue::Ptr(breath_move_max_substatus as *const () as _));
        weapon.fastshift(L2CValue::Ptr(breath_move_max_main_loop as *const () as _));
    }

    0.into()
}

unsafe extern "C" fn breath_move_max_substatus(weapon: &mut L2CWeaponCommon, param_3: L2CValue) -> L2CValue {
    if param_3.get_bool() {
        WorkModule::dec_int(weapon.module_accessor, *WEAPON_INSTANCE_WORK_ID_INT_LIFE);
    }

    0.into()
}

unsafe fn boule_attaque(weapon: &mut L2CWeaponCommon, degats: f32, taille: f32) {
    ATTACK(weapon, 0, 0, Hash40::new("top"), degats, 361, 72, 0, 50, taille, 0.0, 0.0, 0.0, None, None, None, 1.0, 1.0, *ATTACK_SETOFF_KIND_OFF, *ATTACK_LR_CHECK_SPEED, false, 10, 0.0, 0, true, true, false, false, false, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_NO_FLOOR, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_fire"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_FIRE, *ATTACK_REGION_NONE);
    AttackModule::enable_safe_pos(weapon.module_accessor);
}

unsafe fn boule_effets(weapon: &mut L2CWeaponCommon) {
    EFFECT_FOLLOW(weapon, Hash40::new("sys_damage_fire_fly"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.9, true);
    EFFECT_FOLLOW(weapon, Hash40::new("koopa_breath_m_fire"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 1.0, true);
}

unsafe extern "C" fn breath_move_max_main_loop(weapon: &mut L2CWeaponCommon) -> L2CValue {
    // temps écoulé depuis le tir (la vie baisse d'1 par frame)
    let ecoule = WorkModule::get_int(weapon.module_accessor, *WEAPON_INSTANCE_WORK_ID_INT_INIT_LIFE)
        - WorkModule::get_int(weapon.module_accessor, *WEAPON_INSTANCE_WORK_ID_INT_LIFE);
    if ecoule == 10 {
        boule_attaque(weapon, 15.0, 4.0);
    }
    if ecoule > 0 && ecoule % 30 == 0 {
        boule_effets(weapon);
    } else if ecoule > 0 && ecoule % 15 == 0 {
        EFFECT_FOLLOW(weapon, Hash40::new("sys_damage_fire_fly"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.9, true);
    }
    if AttackModule::is_infliction(weapon.module_accessor, *COLLISION_KIND_MASK_HIT) {
        let param_hit_decrease = WorkModule::get_int(weapon.module_accessor, *WEAPON_KOOPA_BREATH_INSTANCE_WORK_ID_INT_HIT_FRAME);
        WorkModule::sub_int(weapon.module_accessor, param_hit_decrease, *WEAPON_INSTANCE_WORK_ID_INT_LIFE);
    }
    if WorkModule::get_int(weapon.module_accessor,*WEAPON_INSTANCE_WORK_ID_INT_LIFE) <= 0
    || (L2CWeaponCommon::sub_ground_module_is_touch_all_consider_speed(weapon).get_bool() && GroundModule::is_touch(weapon.module_accessor, *GROUND_TOUCH_FLAG_SIDE as u32)) {
        EFFECT_OFF_KIND(weapon,Hash40::new("koopa_breath_m_fire"),false,false);
        EFFECT_OFF_KIND(weapon,Hash40::new("sys_damage_fire_fly"),false,false);
        AttackModule::clear_all(weapon.module_accessor);
        MotionModule::change_motion(weapon.module_accessor, Hash40::new("end"), 0.0, 1.0, false, 0.0, false, false);
        StatusModule::change_status_force(weapon.module_accessor, WEAPON_KOOPA_BREATH_STATUS_KIND_NONE.into(), false.into());
        return 0.into();
    }
    
    0.into()
}

pub fn install(agent: &mut Agent) {
    agent.status(Main, *WEAPON_KOOPA_BREATH_STATUS_KIND_MOVE, breath_move_main);
}