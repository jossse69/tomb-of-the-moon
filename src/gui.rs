

use bracket_lib::terminal::{letter_to_option, to_cp437, BTerm, DistanceAlg, FontCharType, Point, Rect, VirtualKeyCode, BLACK, BLUE, CYAN, GRAY, GREY, MAGENTA, RED, RGB, WHITE, YELLOW};
use specs::prelude::*;
use crate::{components::{CombatStats, Equipped, InBackpack, Name, Player, Position, Viewshed}, gamelog::GameLog, map::Map, rect, RunState, State};
use crate::saveload_system::does_save_exist;

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection { NewGame, LoadGame, Quit }

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult { NoSelection{ selected : MainMenuSelection }, Selected{ selected: MainMenuSelection } }


pub fn draw_ui(ecs: &World, ctx : &mut BTerm) {;
    let rect =Rect{x1: 0, y1: 43, x2: 79, y2: 49};
    ctx.fill_region(rect, to_cp437(' '), RGB::from_f32(0.4, 0.6, 0.6), RGB::from_f32(0.4, 0.6, 0.6));
    ctx.draw_hollow_box(0, 43, 79, 6, RGB::named(BLACK), RGB::from_f32(0.4, 0.6, 0.6));
    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    for (_player, stats) in (&players, &combat_stats).join() {
        let health = format!("HP: {}/{}", stats.hp, stats.max_hp);

        ctx.draw_bar_horizontal(39, 44, 20, stats.hp, stats.max_hp, RGB::named(RED), RGB::from_f32(0.4, 0.6, 0.6));
        ctx.print_color(39, 45, RGB::named(WHITE), RGB::from_f32(0.4, 0.6, 0.6), &health);
    }

    let map = ecs.fetch::<Map>();
    let depth = format!("Depth: {}", map.depth);
    ctx.print_color(39, 47, RGB::named(WHITE), RGB::from_f32(0.4, 0.6, 0.6), &depth);

    let log = ecs.fetch::<GameLog>();

    let logrect = Rect{x1: 2, y1: 44, x2: 38, y2: 49};
    ctx.fill_region(logrect, to_cp437(' '), RGB::from_f32(1.0, 1.0, 1.0), RGB::from_f32(0.0, 0.0, 0.0));

    let mut y = 44;
    for s in log.entries.iter().rev() {
        if y < 49 { ctx.print_color(2 , y, RGB::named(WHITE), RGB::from_f32(0.0, 0.0, 0.0), s); }
        y += 1;
    }

    // Draw mouse cursor
    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::from_f32(1.0, 1.0, 1.0));

    draw_tooltips(ecs, ctx)
}

fn draw_tooltips(ecs: &World, ctx : &mut BTerm) {
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();

    let mouse_pos = ctx.mouse_pos();
    if mouse_pos.0 >= map.width || mouse_pos.1 >= map.height { return; }
    let mut tooltip : Vec<String> = Vec::new();
    for (name, position) in (&names, &positions).join() {
        let idx = map.xy_idx(position.x, position.y);
        if position.x == mouse_pos.0 && position.y == mouse_pos.1 && map.visible_tiles[idx] {
            tooltip.push(name.name.to_string());
        }
    }

    if !tooltip.is_empty() {
        let mut width :i32 = 0;
        for s in tooltip.iter() {
            if width < s.len() as i32 { width = s.len() as i32; }
        }
        width += 3;

        if mouse_pos.0 > 40 {
            let arrow_pos = Point::new(mouse_pos.0 - 2, mouse_pos.1);
            let left_x = mouse_pos.0 - width;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(left_x, y, RGB::named(WHITE), RGB::named(GREY), s);
                let padding = (width - s.len() as i32)-1;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x - i, y, RGB::named(WHITE), RGB::named(GREY), &" ".to_string());
                }
                y += 1;
            }
            ctx.print_color(arrow_pos.x, arrow_pos.y, RGB::named(WHITE), RGB::named(GREY), &"->".to_string());
        } else {
            let arrow_pos = Point::new(mouse_pos.0 + 1, mouse_pos.1);
            let left_x = mouse_pos.0 +3;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(left_x + 1, y, RGB::named(WHITE), RGB::named(GREY), s);
                let padding = (width - s.len() as i32)-1;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x + 1 + i, y, RGB::named(WHITE), RGB::named(GREY), &" ".to_string());
                }
                y += 1;
            }
            ctx.print_color(arrow_pos.x, arrow_pos.y, RGB::named(WHITE), RGB::named(GREY), &"<-".to_string());
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult { Cancel, NoResponse, Selected }

pub fn show_inventory(gs : &mut State, ctx : &mut BTerm) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();
    let inventory = (&backpack, &names).join().filter(|item| item.0.owner == *player_entity );
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y-2, 31, (count+3) as i32, RGB::named(BLACK), RGB::from_f32(0.4, 0.6, 0.6));
    ctx.print_color(18, y-2, RGB::named(YELLOW), RGB::from_f32(0.4, 0.6, 0.6), "Inventory");
    ctx.print_color(18, y+count as i32+1, RGB::named(YELLOW), RGB::from_f32(0.4, 0.6, 0.6), "ESC to cancel");

    let mut equippable : Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _pack, name) in (&entities, &backpack, &names).join().filter(|item| item.1.owner == *player_entity ) {
        ctx.set(17, y, RGB::named(WHITE), RGB::named(BLACK), to_cp437('['));
        ctx.set(18, y, RGB::named(YELLOW), RGB::named(BLACK), 97+j as FontCharType);
        ctx.set(19, y, RGB::named(WHITE), RGB::named(BLACK), to_cp437(']'));

        ctx.print_color(21, y, RGB::named(WHITE), RGB::named(BLACK), &name.name.to_string());
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => { (ItemMenuResult::Cancel, None) }
                _ => { 
                    let selection = letter_to_option(key);
                    if selection > -1 && selection < count as i32 {
                        return (ItemMenuResult::Selected, Some(equippable[selection as usize]));
                    }  
                    (ItemMenuResult::NoResponse, None)
                }
            }
        }
    }

}

pub fn drop_item_menu(gs : &mut State, ctx : &mut BTerm) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names).join().filter(|item| item.0.owner == *player_entity );
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y-2, 31, (count+3) as i32, RGB::named(BLACK), RGB::from_f32(0.4, 0.6, 0.6));
    ctx.print_color(18, y-2, RGB::named(YELLOW), RGB::from_f32(0.4, 0.6, 0.6), "Drop Which Item?");
    ctx.print_color(18, y+count as i32+1, RGB::named(YELLOW), RGB::from_f32(0.4, 0.6, 0.6), "ESC to cancel");

    let mut equippable : Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _pack, name) in (&entities, &backpack, &names).join().filter(|item| item.1.owner == *player_entity ) {
        ctx.set(17, y, RGB::named(WHITE), RGB::named(BLACK), to_cp437('['));
        ctx.set(18, y, RGB::named(YELLOW), RGB::named(BLACK), 97+j as FontCharType);
        ctx.set(19, y, RGB::named(WHITE), RGB::named(BLACK), to_cp437(']'));

        ctx.print(21, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => { (ItemMenuResult::Cancel, None) }
                _ => { 
                    let selection = letter_to_option(key);
                    if selection > -1 && selection < count as i32 {
                        return (ItemMenuResult::Selected, Some(equippable[selection as usize]));
                    }  
                    (ItemMenuResult::NoResponse, None)
                }
            }
        }
    }
}

pub fn ranged_target(gs : &mut State, ctx : &mut BTerm, range : i32) -> (ItemMenuResult, Option<Point>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let player_pos = gs.ecs.fetch::<Point>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();

    ctx.print_color(5, 0, RGB::named(YELLOW), RGB::named(BLACK), "Select Target:");

    // Highlight available target cells
    let mut available_cells = Vec::new();
    let visible = viewsheds.get(*player_entity);
    if let Some(visible) = visible {
        // We have a viewshed
        for idx in visible.visible_tiles.iter() {
            let distance = DistanceAlg::Pythagoras.distance2d(*player_pos, *idx);
            if distance <= range as f32 {
                ctx.set_bg(idx.x, idx.y, RGB::named(BLUE));
                available_cells.push(idx);
            }
        }
    } else {
        return (ItemMenuResult::Cancel, None);
    }

    // Draw mouse cursor
    let mouse_pos = ctx.mouse_pos();
    let mut valid_target = false;
    for idx in available_cells.iter() { if idx.x == mouse_pos.0 && idx.y == mouse_pos.1 { valid_target = true; } }
    if valid_target {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(CYAN));
        if ctx.left_click {
            return (ItemMenuResult::Selected, Some(Point::new(mouse_pos.0, mouse_pos.1)));
        }
    } else {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(RED));
        if ctx.left_click {
            return (ItemMenuResult::Cancel, None);
        }
    }

    (ItemMenuResult::NoResponse, None)
}

pub fn main_menu(gs : &mut State, ctx : &mut BTerm) -> MainMenuResult {
    let save_exists = super::saveload_system::does_save_exist();
    let runstate = gs.ecs.fetch::<RunState>();

    ctx.print_color_centered(15, RGB::named(YELLOW), RGB::named(BLACK), "Tomb Of The Moon");

    if let RunState::MainMenu{ menu_selection : selection } = *runstate {
        if selection == MainMenuSelection::NewGame {
            ctx.print_color_centered(24, RGB::named(CYAN), RGB::named(BLACK), "Begin New Game");
        } else {
            ctx.print_color_centered(24, RGB::named(WHITE), RGB::named(BLACK), "Begin New Game");
        }

        if selection == MainMenuSelection::LoadGame {
            ctx.print_color_centered(25, RGB::named(CYAN), RGB::named(BLACK), "Load Game");
        } else {
            if (!save_exists) {
                ctx.print_color_centered(25, RGB::named(GRAY), RGB::named(BLACK), "Load Game [NO SAVE FILE!]");
            }
            else {
                ctx.print_color_centered(25, RGB::named(WHITE), RGB::named(BLACK), "Load Game")
            };
        }

        if selection == MainMenuSelection::Quit {
            ctx.print_color_centered(26, RGB::named(CYAN), RGB::named(BLACK), "Quit");
        } else {
            ctx.print_color_centered(26, RGB::named(WHITE), RGB::named(BLACK), "Quit");
        }

        match ctx.key {
            None => return MainMenuResult::NoSelection{ selected: selection },
            Some(key) => {
                match key {
                    VirtualKeyCode::Escape => { return MainMenuResult::NoSelection{ selected: MainMenuSelection::Quit } }
                    VirtualKeyCode::Up => {
                        let mut newselection;
                        match selection {
                            MainMenuSelection::NewGame => newselection = MainMenuSelection::Quit,
                            MainMenuSelection::LoadGame => newselection = MainMenuSelection::NewGame,
                            MainMenuSelection::Quit => newselection = MainMenuSelection::LoadGame
                        }
                        if newselection == MainMenuSelection::LoadGame && !save_exists {
                            newselection = MainMenuSelection::NewGame;
                        }
                        return MainMenuResult::NoSelection{ selected: newselection }
                    }
                    VirtualKeyCode::Down => {
                        let mut newselection;
                        match selection {
                            MainMenuSelection::NewGame => newselection = MainMenuSelection::LoadGame,
                            MainMenuSelection::LoadGame => newselection = MainMenuSelection::Quit,
                            MainMenuSelection::Quit => newselection = MainMenuSelection::NewGame
                        }
                        if newselection == MainMenuSelection::LoadGame && !save_exists {
                            newselection = MainMenuSelection::Quit;
                        }
                        return MainMenuResult::NoSelection{ selected: newselection }
                    }
                    VirtualKeyCode::Return => return MainMenuResult::Selected{ selected : selection },
                    _ => return MainMenuResult::NoSelection{ selected: selection }
                }
            }
        }
    }

    MainMenuResult::NoSelection { selected: MainMenuSelection::NewGame }
}

pub fn remove_item_menu(gs : &mut State, ctx : &mut BTerm) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<Equipped>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names).join().filter(|item| item.0.owner == *player_entity );
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y-2, 31, (count+3) as i32, RGB::named(BLACK), RGB::from_f32(0.4, 0.6, 0.6));
    ctx.print_color(18, y-2, RGB::named(YELLOW), RGB::from_f32(0.4, 0.6, 0.6), "Remove Which Item?");
    ctx.print_color(18, y+count as i32+1, RGB::named(YELLOW), RGB::from_f32(0.4, 0.6, 0.6), "ESC to cancel");

    let mut equippable : Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _pack, name) in (&entities, &backpack, &names).join().filter(|item| item.1.owner == *player_entity ) {
        ctx.set(17, y, RGB::named(WHITE), RGB::named(BLACK), to_cp437('['));
        ctx.set(18, y, RGB::named(YELLOW), RGB::named(BLACK), 97+j as FontCharType);
        ctx.set(19, y, RGB::named(WHITE), RGB::named(BLACK), to_cp437(']'));

        ctx.print(21, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => { (ItemMenuResult::Cancel, None) }
                _ => { 
                    let selection = letter_to_option(key);
                    if selection > -1 && selection < count as i32 {
                        return (ItemMenuResult::Selected, Some(equippable[selection as usize]));
                    }  
                    (ItemMenuResult::NoResponse, None)
                }
            }
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum GameOverResult { NoSelection, QuitToMenu }

pub fn game_over(ctx : &mut BTerm) -> GameOverResult {
    ctx.print_color_centered(15, RGB::named(YELLOW), RGB::named(BLACK), "You Died!");
    ctx.print_color_centered(17, RGB::named(WHITE), RGB::named(BLACK), "Now your corpse will roten in the darkness.");
    ctx.print_color_centered(19, RGB::named(RED), RGB::named(BLACK), "The future is still gloom and doom...");

    ctx.print_color_centered(21, RGB::named(CYAN), RGB::named(BLACK), "Press any key to return to the menu.");

    match ctx.key {
        None => GameOverResult::NoSelection,
        Some(_) => GameOverResult::QuitToMenu
    }
}