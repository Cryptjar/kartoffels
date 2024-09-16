use crate::{cfg, AliveBotEntryMut, BotId, KillBot, World};

pub fn run(world: &mut World) {
    let ids = world.bots.alive.pick_ids(&mut world.rng);

    for _ in 0..cfg::SIM_TICKS {
        tick(world, &ids);
    }

    world
        .mode
        .on_after_tick(&mut world.rng, &mut world.theme, &mut world.map);
}

fn tick(world: &mut World, ids: &[BotId]) {
    for &id in ids {
        let Some(AliveBotEntryMut { pos, bot, locator }) =
            world.bots.alive.get_mut(id)
        else {
            // Our bot got killed in the meantime, happens
            continue;
        };

        let kill = match bot.tick(&mut world.rng, &world.map, &locator, pos) {
            Ok(state) => state.apply(world, id, pos),

            Err(err) => Some(KillBot {
                id,
                reason: format!("firmware crashed: {}", err),
                killer: None,
            }),
        };

        if let Some(kill) = kill {
            super::kill::run(world, kill);
        }
    }
}
