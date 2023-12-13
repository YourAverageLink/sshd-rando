#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

use crate::actor;
use crate::event;
use crate::flag;
use crate::math;
use crate::player;
use crate::savefile;
use crate::yuzu;

use core::arch::asm;
use core::ffi::c_void;
use static_assertions::assert_eq_size;

// repr(C) prevents rust from reordering struct fields.
// packed(1) prevents rust from aligning structs to the size of the largest
// field.

// Using u64 or 64bit pointers forces structs to be 8-byte aligned.
// The vanilla code seems to be 4-byte aligned. To make extra sure, used
// packed(1) to force the alignment to match what you define.

// Always add an assert_eq_size!() macro after defining a struct to ensure it's
// the size you expect it to be.

#[repr(C, packed(1))]
#[derive(Copy, Clone)]
pub struct dAcItem {
    pub base:                    actor::ActorObjectBase,
    pub itemid:                  u16,
    pub _0:                      [u8; 6],
    pub item_model_ptr:          u64,
    pub _1:                      [u8; 2784],
    pub actor_list_element:      u32,
    pub _2:                      [u8; 816],
    pub freestanding_y_offset:   f32,
    pub _3:                      [u8; 44],
    pub final_determined_itemid: u16,
    pub _4:                      [u8; 10],
    pub prevent_drop:            u8,
    pub _5:                      [u8; 3],
    pub no_longer_waiting:       u8,
    pub _6:                      [u8; 19],
}
assert_eq_size!([u8; 0x1288], dAcItem);

#[repr(C, packed(1))]
#[derive(Copy, Clone)]
pub struct dAcTbox {
    pub base:                         actor::ActorObjectBase,
    pub mdlAnmChr_c:                  [u8; 0xB8],
    pub _0:                           [u8; 0x12E8],
    pub state_mgr:                    [u8; 0x70],
    pub _1:                           [u8; 0xA8],
    pub dowsing_target:               [u8; 0x28],
    pub goddess_chest_dowsing_target: [u8; 0x28],
    pub register_dowsing_target:      [u8; 0x10],
    pub unregister_dowsing_target:    [u8; 0x10],
    pub _2:                           [u8; 0x58],
    pub itemid_0x1ff:                 flag::ITEMFLAGS,
    pub item_model_index:             u16,
    pub chest_opened:                 u8,
    pub spawn_sceneflag:              u8,
    pub set_sceneflag:                u8,
    pub chestflag:                    u8,
    pub unk:                          u8,
    pub chest_subtype:                u8,
    pub _3:                           [u8; 2],
    pub is_chest_opened_related:      u8,
    pub _4:                           [u8; 4],
    pub do_obstructed_check:          bool,
    pub _5:                           [u8; 6],
}
assert_eq_size!([u8; 0x19A8], dAcTbox);

// IMPORTANT: when using vanilla code, the start point must be declared in
// symbols.yaml and then added to this extern block.
extern "C" {
    static PLAYER_PTR: *mut player::dPlayer;

    static FILE_MGR: *mut savefile::FileMgr;
    static ROOM_MGR: *mut actor::RoomMgr;
    static HARP_RELATED: *mut event::HarpRelated;

    static DUNGEONFLAG_MGR: *mut flag::DungeonflagMgr;

    static mut STATIC_DUNGEONFLAGS: [u16; 8];
    static mut CURRENT_STAGE_NAME: [u8; 8];

    static EQUIPPED_SWORD: u8;
    static mut ITEM_GET_BOTTLE_POUCH_SLOT: u32;
    static mut NUMBER_OF_ITEMS: u32;

    // Functions
    fn sinf(x: f32) -> f32;
    fn cosf(x: f32) -> f32;
    fn resolveItemMaybe(itemid: u64) -> u64;
}

// IMPORTANT: when adding functions here that need to get called from the game,
// add `#[no_mangle]` and add a .global *symbolname* to
// additions/rust-additions.asm
#[no_mangle]
pub fn give_item(itemid: u8) {
    unsafe {
        NUMBER_OF_ITEMS = 0;
        ITEM_GET_BOTTLE_POUCH_SLOT = 0xFFFFFFFF;

        let new_itemid = resolveItemMaybe(itemid as u64);

        actor::spawn_actor(
            actor::ACTORID::ITEM,
            (*ROOM_MGR).roomid.into(),
            new_itemid as u32 | 0x5BFC00,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0xFFFFFFFF,
        );

        ITEM_GET_BOTTLE_POUCH_SLOT = 0xFFFFFFFF;
        NUMBER_OF_ITEMS = 0;
    }
}

#[no_mangle]
pub fn handle_crest_hit_give_item(crest_actor: *mut actor::dAcOSwSwordBeam) {
    unsafe {
        // Reset position so that we don't void out before getting the items
        let position = math::Vec3f {
            x: 0.0,
            y: 0.0,
            z: 304.0,
        };
        (*PLAYER_PTR).obj_base_members.base.pos = position;

        // Goddess Sword Reward
        if flag::check_local_sceneflag(50) == 0 {
            let goddess_sword_reward: u8 =
                ((*crest_actor).base.basebase.members.param1 >> 0x18) as u8;
            give_item(goddess_sword_reward);
            flag::set_local_sceneflag(50);
        }
        if (EQUIPPED_SWORD < 2) {
            return;
        }

        // Longsword Reward
        if flag::check_local_sceneflag(51) == 0 {
            let longsword_reward: u8 = ((*crest_actor).base.basebase.members.param1 >> 0x10) as u8;
            give_item(longsword_reward);
            flag::set_local_sceneflag(51);
        }
        if (EQUIPPED_SWORD < 3) {
            return;
        }

        // White Sword Reward
        if flag::check_local_sceneflag(52) == 0 {
            let whitesword_reward: u8 = ((*crest_actor).base.members.base.param2 >> 0x18) as u8;
            give_item(whitesword_reward);
            flag::set_local_sceneflag(52);
        }
    }
}

#[no_mangle]
pub fn handle_custom_item_get(item_actor: *mut dAcItem) -> u16 {
    const BK_TO_FLAGINDEX: [usize; 7] = [
        12,  // AC BK - item id 25
        15,  // FS BK - item id 26
        18,  // SSH BK - item id 27
        255, // unused, shouldn't happen
        11,  // SV BK - item id 29
        14,  // ET - item id 30
        17,  // LMF - item id 31
    ];

    const SK_TO_FLAGINDEX: [usize; 7] = [
        11, // SV SK - item id 200
        17, // LMF SK - item id 201
        12, // AC SK - item id 202
        15, // FS SK - item id 203
        18, // SSH SK - item id 204
        20, // SK SK - item id 205
        9,  // Caves SK - item id 206
    ];

    const MAP_TO_FLAGINDEX: [usize; 7] = [
        11, // SV MAP - item id 207
        14, // ET MAP - item id 208
        17, // LMF MAP - item id 209
        12, // AC MAP - item id 210
        15, // FS MAP - item id 211
        18, // SSH MAP - item id 212
        20, // SK MAP - item id 213
    ];

    unsafe {
        let itemid = (*item_actor).itemid;

        let mut dungeon_item_mask = 0;

        if (itemid >= 25 && itemid <= 27) || (itemid >= 29 && itemid <= 31) {
            dungeon_item_mask = 0x80; // boss keys
        }

        if dungeon_item_mask == 0 {
            if itemid >= 200 && itemid <= 206 {
                dungeon_item_mask = 0x0F; // small keys
            }
        }

        if dungeon_item_mask == 0 {
            if itemid >= 207 && itemid <= 213 {
                dungeon_item_mask = 0x02; // maps
            }
        }

        if dungeon_item_mask != 0 {
            let current_scene_index = (*DUNGEONFLAG_MGR).sceneindex as usize;
            let mut dungeon_item_scene_index = 0xFF;

            if dungeon_item_mask == 0x80 {
                dungeon_item_scene_index = BK_TO_FLAGINDEX[(itemid - 25) as usize];
            }

            if dungeon_item_mask == 0x0F {
                dungeon_item_scene_index = SK_TO_FLAGINDEX[(itemid - 200) as usize];
            }

            if dungeon_item_mask == 0x02 {
                dungeon_item_scene_index = MAP_TO_FLAGINDEX[(itemid - 207) as usize];
            }

            // Set the local flag if the item is in its vanilla scene.
            if current_scene_index == dungeon_item_scene_index {
                if dungeon_item_mask != 0x0F {
                    STATIC_DUNGEONFLAGS[0] |= dungeon_item_mask;
                } else {
                    STATIC_DUNGEONFLAGS[1] += 1;
                }
            }
            // Otherwise, set the global flag.
            if dungeon_item_mask != 0x0F {
                (*FILE_MGR).FA.dungeonflags[dungeon_item_scene_index][0] |= dungeon_item_mask;
            } else {
                (*FILE_MGR).FA.dungeonflags[dungeon_item_scene_index][1] += 1;
            }
        }

        // Get necessary params for setting a custom flag if this item has one
        let (flag, sceneindex, flag_space_trigger, original_itemid) =
            unpack_custom_item_params(item_actor);

        if flag != 0x7F {
            // Use different flag spaces depending on the value of the
            // flag_space_trigger
            match flag_space_trigger {
                0 => flag::set_global_sceneflag(sceneindex as u16, flag as u16),
                1 => flag::set_global_dungeonflag(sceneindex as u16, flag as u16),
                _ => {},
            }
        }

        return (*item_actor).final_determined_itemid;
    }
}

// Unpacks our custom item params into separate variables
#[no_mangle]
pub fn unpack_custom_item_params(item_actor: *mut dAcItem) -> (u32, u32, u32, u32) {
    unsafe {
        let param2: u32 = (*item_actor).base.members.base.param2;
        let flag: u32 = (param2 & (0x00007F00)) >> 8;
        let mut sceneindex: u32 = (param2 & (0x00018000)) >> 15;
        let flag_space_trigger: u32 = (param2 & (0x00020000)) >> 17;
        let mut original_itemid: u32 = (param2 & (0x00FC0000)) >> 18;

        // Transform the scene index into one of the unused ones
        match sceneindex {
            0 => sceneindex = 6,
            1 => sceneindex = 13,
            2 => sceneindex = 16,
            3 => sceneindex = 19,
            _ => {},
        }

        // Transform the original_itemid into its proper itemid
        match original_itemid {
            1 => original_itemid = 42, // Stamina Fruit
            2 => original_itemid = 2,  // Green Rupee
            3 => original_itemid = 3,  // Blue Rupee
            4 => original_itemid = 4,  // Red Rupee
            5 => original_itemid = 34, // Rupoor
            _ => {},
        }

        return (flag, sceneindex, flag_space_trigger, original_itemid);
    }
}

#[no_mangle]
pub fn check_and_modify_item_actor(item_actor: *mut dAcItem) {
    unsafe {
        // Get necessary params for checking if this item has a custom
        // flag
        let (flag, sceneindex, flag_space_trigger, original_itemid) =
            unpack_custom_item_params(item_actor);

        // Despawn the item if it's one of the stamina fruit on LMF that
        // shouldn't exist until the dungeon has been raised. Actors are
        // identified by Z position
        let zPos: f32 = (*item_actor).base.members.base.pos.z;
        if (&CURRENT_STAGE_NAME[..5] == b"F300\0"
            && flag::check_storyflag(8) == 0 // LMF is not raised
            && (zPos == 46.531517028808594 || zPos == 105.0 || zPos == 3495.85009765625))
        {
            // Set itemid to 0 which despawns it later in the init function
            (*item_actor).base.basebase.members.param1 &= !0x1FF;
        }

        // Don't give a textbox for rupees
        match (*item_actor).base.basebase.members.param1 & 0x1FF {
            2 | 3 | 4 | 31 | 32 => {
                (*item_actor).base.basebase.members.param1 |= 0x200;
            },
            _ => {},
        }

        // Check if the flag is on
        let mut flag_is_on = 0;
        match flag_space_trigger {
            0 => flag_is_on = flag::check_global_sceneflag(sceneindex as u16, flag as u16),
            1 => flag_is_on = flag::check_global_dungeonflag(sceneindex as u16, flag as u16),
            _ => {},
        }

        // If we have a custom flag and it's been set, revert this item back to what
        // it originally was
        if flag != 0x7F && flag_is_on != 0 {
            (*item_actor).base.basebase.members.param1 &= !0x1FF;
            (*item_actor).base.basebase.members.param1 |= original_itemid;
            // Set bit 9 for no textbox
            (*item_actor).base.basebase.members.param1 |= 0x200;
        // Otherwise, if we have a custom flag, potentially fix
        // the horizontal offset if necessary
        } else if (flag != 0x7F) {
            fix_freestanding_item_horizontal_offset(item_actor);
        }

        // Fix the y offset if necessary
        fix_freestanding_item_y_offset(item_actor);

        // Replaced Code
        if (((*item_actor).base.basebase.members.param1 >> 10) & 0xFF) == 0xFF {
            asm!("mov x19, #1");
            asm!("cmp x19, #1");
        }
        asm!("mov x19, {0}", in(reg) item_actor);
    }
}

#[no_mangle]
pub fn activation_checks_for_goddess_walls() -> bool {
    unsafe {
        // Replaced code
        if (*HARP_RELATED).some_check_for_continuous_strumming == 0
            || (*HARP_RELATED).some_other_harp_thing != 0
        {
            // Additional check for BotG
            if flag::check_itemflag(flag::ITEMFLAGS::BALLAD_OF_THE_GODDESS) == 1 {
                return true;
            }
        }

        return false;
    }
}

#[no_mangle]
pub fn fix_freestanding_item_y_offset(item_actor: *mut dAcItem) {
    unsafe {
        let actor_param1 = (*item_actor).base.basebase.members.param1;

        if (actor_param1 >> 9) & 0x1 == 0 {
            let mut use_default_scaling = false;
            let mut y_offset = 0.0f32;
            let item_rot = (*item_actor).base.members.base.rot;

            // Item id
            match actor_param1 & 0x1FF {
                // Sword | Harp | Mitts | Beedle's Insect Cage | Sot | Songs
                10 | 16 | 56 | 159 | 180 | 186..=193 => y_offset = 20.0,
                // Bow | Sea Chart | Wooden Shield | Hylian Shield
                19 | 98 | 116 | 125 => y_offset = 23.0,
                // Clawshots | Spiral Charge
                20 | 21 => y_offset = 25.0,
                // AC BK | FS BK
                25 | 26 => y_offset = 30.0,
                // SSH BK, ET Key, SV BK, ET BK | Amber Tablet
                27..=30 | 179 => y_offset = 24.0,
                // LMF BK
                31 => y_offset = 27.0,
                // Crystal Pack | 5 Bombs | 10 Bombs | Single Crystal | Beetle | Pouch | Small Bomb Bag | Eldin Ore
                35 | 40 | 41 | 48 | 53 | 112 | 134 | 165 => y_offset = 18.0,
                // Bellows | Bug Net | Bomb Bag
                49 | 71 | 92 => y_offset = 26.0,
                52          // Slingshot
                | 68        // Water Dragon's Scale
                | 100..=104 // Medals
                | 108       // Wallets
                | 114       // Life Medal
                | 153       // Empty Bottle
                | 161..=164 // Treasures
                | 166..=170 // Treasures
                | 172..=174 // Treasures
                | 178       // Ruby Tablet
                | 198       // Life Tree Fruit
                | 199 => y_offset = 16.0,
                // Semi-rare | Rare Treasure
                63 | 64 => y_offset = 15.0,
                // Heart Container
                93 => use_default_scaling = true,
                95..=97 => {
                    y_offset = 24.0;
                    use_default_scaling = true;
                },
                // Seed Satchel | Golden Skull
                128 | 175 => y_offset = 14.0,
                // Quiver | Whip | Emerald Tablet | Maps
                131 | 137 | 177 | 207..=213 => y_offset = 19.0,
                // Earrings
                138 => y_offset = 6.0,
                // Letter | Monster Horn
                158 | 171 => y_offset = 12.0,
                // Rattle
                160 => {
                    y_offset = 5.0;
                    use_default_scaling = true;
                },
                // Goddess Plume
                176 => y_offset = 17.0,
                _ => y_offset = 0.0,
            }

            // Only apply the offset if the item isn't tilted
            if item_rot.x < 0x2000 || item_rot.x > 0xE000 {
                (*item_actor).freestanding_y_offset = y_offset;
            }

            if use_default_scaling {
                (*item_actor).base.members.base.rot.y |= 1;
            } else {
                (*item_actor).base.members.base.rot.y &= 0xFFFE;
            }
        }
    }
}

#[no_mangle]
pub fn fix_freestanding_item_horizontal_offset(item_actor: *mut dAcItem) {
    unsafe {
        // If the item is facing sideways, apply a horizontal offset (i.e. stamina
        // fruit on walls) and rotate the item if necessary
        let item_rot = (*item_actor).base.members.base.rot;
        if item_rot.x > 0x2000 && item_rot.x < 0xE000 {
            let actor_param1 = (*item_actor).base.basebase.members.param1;
            let mut h_offset = 0.0f32;
            let mut angle_change_x = 0u16;
            let mut angle_change_y = 0u16;
            let mut angle_change_z = 0u16;

            // Item id
            match actor_param1 & 0x1FF {
                // Rupees
                2 | 3 | 4 | 32 | 33 | 34 => h_offset = 20.0,
                // Progressive Sword
                10 => {
                    h_offset = 7.0;
                    angle_change_x = 0xD900;
                    angle_change_y = 0xF400;
                    angle_change_z = 0xF600;
                },
                // Goddess's Harp | All Songs
                16 | 186..=193 => {
                    h_offset = 17.0;
                    angle_change_x = 0x0800;
                    angle_change_y = 0x2500;
                    angle_change_z = 0x0800;
                },
                // Progressive Bow
                19 => h_offset = 17.0,
                // Clawshots
                20 => {
                    h_offset = 25.0;
                    angle_change_x = 0x0500;
                    angle_change_y = 0x2400;
                },
                21 => {
                    h_offset = 27.0;
                    angle_change_y = 0x3000;
                    angle_change_z = 0x0300;
                },
                // AC BK
                25 => {
                    h_offset = 50.0;
                    angle_change_x = 0xEF00;
                },
                // FS BK | SV BK
                26 | 29 => h_offset = 40.0,
                // SSH BK
                27 => h_offset = 47.0,
                // Key Piece
                28 => {
                    h_offset = 10.0;
                    angle_change_x = 0x0800;
                    angle_change_y = 0x2000;
                    angle_change_z = 0x0800;
                },
                // ET BK
                30 => h_offset = 60.0,
                // LMF BK | Small Seed Satchel | Whip
                31 | 128 | 137 => h_offset = 25.0,
                // Gratitude Crystal Pack | Single Crystal
                35 | 48 => h_offset = 28.0,
                // 5 Bombs | 10 Bombs
                40 | 41 => {
                    h_offset = 20.0;
                    angle_change_y = 0x1600;
                },
                // Gust Bellows
                49 => {
                    h_offset = 35.0;
                    angle_change_x = 0x1100;
                    angle_change_z = 0x2000;
                },
                // Progressive Slingshot
                52 => {
                    h_offset = 30.0;
                    angle_change_x = 0x1000;
                    angle_change_z = 0x1000;
                },
                // Progressive Beetle
                53 => {
                    h_offset = 40.0;
                    angle_change_x = 0xE000;
                    angle_change_y = 0xCB00;
                    angle_change_z = 0xB000;
                },
                // Progressive Mitts
                56 => {
                    h_offset = 45.0;
                    angle_change_y = 0xE800;
                },
                // Water Dragon Scale | Sea Chart
                68 | 98 => h_offset = 15.0,
                // Bug Medal | Life Medal
                70 | 114 => {
                    h_offset = 15.0;
                    angle_change_x = 0x0A80;
                },
                // Progressive Bug Net
                71 => {
                    h_offset = 30.0;
                    angle_change_x = 0x1000;
                    angle_change_y = 0xE800;
                    angle_change_z = 0x2000;
                },
                // Bomb Bag
                92 => h_offset = 45.0,
                // Heart Container | Progressive Pouch | Life Tree Fruit
                93 | 112 | 198 => h_offset = 35.0,
                // Heart Piece
                94 => h_offset = 40.0,
                // Triforce Pieces
                95 | 96 | 97 => h_offset = 75.0,
                // Heart Medal | Rupee Medal | Treasure Medal | Potion Medal | Cursed Medal
                100..=104 => {
                    h_offset = 15.0;
                    angle_change_y = 0x4000;
                    angle_change_z = 0x0A00;
                },
                // Progressive Wallet | Bottle | Tumbleweed | Extra Wallet
                108 | 153 | 163 | 199 => h_offset = 20.0,
                // Wooden Shield | Hylian Shield
                116 | 125 => {
                    h_offset = 25.0;
                    angle_change_x = 0x0800;
                    angle_change_y = 0x2400;
                    angle_change_z = 0x1000;
                },
                // Small Quiver
                131 => {
                    h_offset = 25.0;
                    angle_change_x = 0x1000;
                    angle_change_z = 0x1000;
                },
                // Small Bomb Bag
                134 => h_offset = 30.0,
                // Fireshield Earrings
                138 => h_offset = 20.0,
                // Cawlin's Letter
                158 => {
                    h_offset = 15.0;
                    angle_change_y = 0x2000;
                },
                // Beedle's Insect Cage
                159 => {
                    h_offset = 40.0;
                    angle_change_y = 0x2000;
                },
                // Rattle
                160 => {
                    h_offset = 25.0;
                    angle_change_y = 0xE000;
                },
                // All Treasures
                63 | 64 | 165..=176 => h_offset = 25.0,
                // Tablets
                177..=179 => {
                    h_offset = 10.0;
                    angle_change_x = 0x0800;
                    angle_change_y = 0x2000;
                    angle_change_z = 0x0800;
                },
                // Stone of Trials
                180 => {
                    h_offset = 20.0;
                    angle_change_x = 0x0800;
                    angle_change_y = 0x2000;
                    angle_change_z = 0x0800;
                },
                // Small Keys
                200..=206 => {
                    h_offset = 5.0;
                    angle_change_x = 0x0C00;
                    angle_change_y = 0x1000;
                    angle_change_z = 0x0600;
                },
                // Maps
                207..=213 => {
                    h_offset = 30.0;
                    angle_change_x = 0x0800;
                    angle_change_y = 0x1000;
                    angle_change_z = 0x0800;
                },
                _ => h_offset = 0.0,
            }

            // Use trigonometry to figure out the horizontal offsets
            // Assume items are tilted on the x rotation and turned with the
            // y rotation to get whatever angle they have. If they're rotated with z
            // change it accordingly
            let mut facing_angle = item_rot.y;
            if facing_angle == 0 {
                facing_angle = 0 - item_rot.z;
                (*item_actor).base.members.base.rot.y = facing_angle;
                (*item_actor).base.members.base.rot.z = 0;
            }
            let facing_angle_radians: f32 = (facing_angle as f32 / 65535 as f32) * 2.0 * 3.14159;
            let xOffset = sinf(facing_angle_radians) * h_offset;
            let zOffset = cosf(facing_angle_radians) * h_offset;
            (*item_actor).base.members.base.pos.x += xOffset;
            (*item_actor).base.members.base.pos.z += zOffset;
            (*item_actor).base.members.base.rot.x = 0;
            (*item_actor).base.members.base.rot.x += angle_change_x;
            (*item_actor).base.members.base.rot.y += angle_change_y;
            (*item_actor).base.members.base.rot.z += angle_change_z;
        }
    }
}
