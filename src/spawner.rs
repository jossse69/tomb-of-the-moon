use std::collections::HashMap;

use bracket_lib::prelude::*;
use specs::{saveload::{MarkedBuilder, SimpleMarker}, Builder, Entity, World, WorldExt};
use crate::{components::{AreaOfEffect, BlocksTile, CombatStats, Confusion, Consumable, DefenseBonus, EquipmentSlot, Equippable, InflictsDamage, Item, MeleePowerBonus, Monster, Name, Player, Position, Potion, ProvidesHealing, Ranged, Renderable, SerializeMe, Viewshed}, map::MAPWIDTH, random_table::RandomTable};

const MAX_MONSTERS : i32 = 4;
const MAX_ITEMS : i32 = 2;

pub fn player(ecs : &mut World, player_x : i32, player_y : i32) -> Entity {

    let ply = ecs
    .create_entity()
    .with(Position { x: player_x, y: player_y })
    .with(Renderable {
        glyph: to_cp437('☻'),
        fg: RGB::named(YELLOW),
        bg: RGB::named(BLACK),
        render_order: 0
    })
    .with(Player{})
    .with(Viewshed{ visible_tiles : Vec::new(), range : 8, dirty : true})
    .with(Name{name: "Player".to_string() })
    .with(CombatStats{ max_hp: 30, hp: 30, defense: 2, power: 5 })
    .marked::<SimpleMarker<SerializeMe>>()
    .build();

    ply
}

/// Spawns a random monster at a given location
pub fn random_monster(ecs: &mut World, x: i32, y: i32) {
    let roll :i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 2);
    }
    match roll {
        1 => { bat(ecs, x, y) }
        _ => { skeleton(ecs, x, y) }
    }
}

fn monster<S : ToString>(ecs: &mut World, x: i32, y: i32, glyph : FontCharType, name : S) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph,
            fg: RGB::named(RED),
            bg: RGB::named(BLACK),
            render_order: 1
        })
        .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
        .with(Monster{})
        .with(Name{ name : name.to_string() })
        .with(BlocksTile{})
        .with(CombatStats{ max_hp: 16, hp: 16, defense: 1, power: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

/// Fills a room with stuff!
#[allow(clippy::map_entry)]
pub fn spawn_room(ecs: &mut World, room : &Rect, map_depth: i32) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points : HashMap<usize, String> = HashMap::new();

    // Scope to keep the borrow checker happy
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_spawns = rng.roll_dice(1, MAX_MONSTERS + 3) + (map_depth - 1) - 3;

        for _i in 0 .. num_spawns {
            let mut added = false;
            let mut tries = 0;
            while !added && tries < 20 {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !spawn_points.contains_key(&idx) {
                    spawn_points.insert(idx, spawn_table.roll(&mut rng));
                    added = true;
                } else {
                    tries += 1;
                }
            }
        }
    }

    // Actually spawn the monsters
    for spawn in spawn_points.iter() {
        let x = (*spawn.0 % MAPWIDTH) as i32;
        let y = (*spawn.0 / MAPWIDTH) as i32;

        match spawn.1.as_ref() {
            "Bat" => bat(ecs, x, y),
            "Skeleton" => skeleton(ecs, x, y),
            "Red Glowing Potion" => health_potion(ecs, x, y),
            "Fireball Scroll" => fireball_scroll(ecs, x, y),
            "Dizzily Scroll" => confusion_scroll(ecs, x, y),
            "Magic Missile Scroll" => magic_missile_scroll(ecs, x, y),
            "Knife" => knife(ecs, x, y),
            "Shield" => shield(ecs, x, y),
            "Longsword" => longsword(ecs, x, y),
            "Tower Shield" => tower_shield(ecs, x, y),
            _ => {}
        }
    }
}
fn random_item(ecs: &mut World, x: i32, y: i32) {
    let roll :i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 4);
    }
    match roll {
        1 => { health_potion(ecs, x, y) }
        2 => { fireball_scroll(ecs, x, y) }
        3 => { confusion_scroll(ecs, x, y) }
        _ => { magic_missile_scroll(ecs, x, y) }
    }
}

fn room_table(map_depth: i32) -> RandomTable {
    RandomTable::new()
        .add("Bat", 10)
        .add("Skeleton", 1 + map_depth)
        .add("Red Glowing Potion", 7)
        .add("Fireball Scroll", 2 + map_depth)
        .add("Dizzily Scroll", 2 + map_depth)
        .add("Magic Missile Scroll", 4)
        .add("Knife", 3)
        .add("Shield", 3)
        .add("Longsword", map_depth - 1)
        .add("Tower Shield", map_depth - 1)
}

fn bat (ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, to_cp437('b'), "Bat".to_string()) }
fn skeleton (ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, to_cp437('s'), "Skeleton".to_string()) }

fn health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: to_cp437('ô'),
            fg: RGB::named(MAGENTA),
            bg: RGB::named(BLACK),
            render_order: 2
        })
        .with(Name{ name : "Red Glowing Potion".to_string() })
        .with(Item{})
        .with(Consumable{})
        .with(ProvidesHealing{ heal_amount: 8 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn magic_missile_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: to_cp437('~'),
            fg: RGB::named(CYAN),
            bg: RGB::named(BLACK),
            render_order: 2
        })
        .with(Name{ name : "Magic Missile Scroll".to_string() })
        .with(Item{})
        .with(Consumable{})
        .with(Ranged{ range: 6 })
        .with(InflictsDamage{ damage: 8 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn fireball_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: to_cp437('~'),
            fg: RGB::named(CYAN),
            bg: RGB::named(BLACK),
            render_order: 2
        })
        .with(Name{ name : "Fireball Scroll".to_string() })
        .with(Item{})
        .with(Consumable{})
        .with(Ranged{ range: 6 })
        .with(InflictsDamage{ damage: 20 })
        .with(AreaOfEffect{ radius: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn confusion_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: to_cp437('~'),
            fg: RGB::named(CYAN),
            bg: RGB::named(BLACK),
            render_order: 2
        })
        .with(Name{ name : "Dizzily Scroll".to_string() })
        .with(Item{})
        .with(Consumable{})
        .with(Ranged{ range: 6 })
        .with(Confusion{ turns: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn knife(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: to_cp437('/'),
            fg: RGB::named(GREEN),
            bg: RGB::named(BLACK),
            render_order: 2
        })
        .with(Name{ name : "Knife".to_string() })
        .with(Item{})
        .with(Equippable{ slot: EquipmentSlot::Melee })
        .with(MeleePowerBonus{ power: 2 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn shield(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: to_cp437('('),
            fg: RGB::named(GREEN),
            bg: RGB::named(BLACK),
            render_order: 2
        })
        .with(Name{ name : "Shield".to_string() })
        .with(Item{})
        .with(Equippable{ slot: EquipmentSlot::Shield })
        .with(DefenseBonus{ defense: 1 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn longsword(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: to_cp437('/'),
            fg: RGB::named(GREEN),
            bg: RGB::named(BLACK),
            render_order: 2
        })
        .with(Name{ name : "Longsword".to_string() })
        .with(Item{})
        .with(Equippable{ slot: EquipmentSlot::Melee })
        .with(MeleePowerBonus{ power: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn tower_shield(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: to_cp437('('),
            fg: RGB::named(GREEN),
            bg: RGB::named(BLACK),
            render_order: 2
        })
        .with(Name{ name : "Tower Shield".to_string() })
        .with(Item{})
        .with(Equippable{ slot: EquipmentSlot::Shield })
        .with(DefenseBonus{ defense: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}