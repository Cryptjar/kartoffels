use crate::{
    AliveBot, BotEntry, BotId, BotInfo, CreateConnection, KillBot, QueuedBot,
    Request, Shutdown, World,
};
use anyhow::{anyhow, Result};
use kartoffels_vm as vm;
use std::borrow::Cow;
use tokio::sync::mpsc;
use tracing::{debug, info};

pub fn run(world: &mut World) {
    while let Ok(msg) = world.rx.try_recv() {
        match msg {
            Request::Listen { tx } => {
                let (tx2, rx2) = mpsc::channel(256);

                world.event_tx = Some(tx2);

                _ = tx.send(rx2);
            }

            Request::Join { id, tx } => {
                let (tx2, rx2) = mpsc::channel(32);

                world.events.send(CreateConnection { id, tx: tx2 });

                _ = tx.send(rx2);
            }

            Request::Pause { paused } => {
                world.paused = paused;
            }

            Request::Close { tx } => {
                info!("initiating shutdown");

                world.events.send(Shutdown { tx });
            }

            Request::UploadBot { src, tx } => {
                _ = tx.send(do_upload_bot(world, src));
            }

            Request::RestartBot { id } => {
                world.events.send(KillBot {
                    id,
                    reason: "forcefully restarted".into(),
                    killer: None,
                });
            }

            Request::DestroyBot { id } => {
                world.bots.remove(id);
            }

            Request::GetBots { tx } => {
                let bots = world
                    .bots
                    .iter()
                    .map(|bot| {
                        let id = match bot {
                            BotEntry::Alive(bot) => bot.id,
                            BotEntry::Dead(bot) => bot.id,
                            BotEntry::Queued(bot) => bot.id,
                        };

                        BotInfo { id }
                    })
                    .collect();

                _ = tx.send(bots);
            }

            Request::SetSpawnPoint { at } => {
                world.spawn_point = at;
            }
        }
    }
}

fn do_upload_bot(world: &mut World, src: Cow<'static, [u8]>) -> Result<BotId> {
    let fw = vm::Firmware::new(&src)?;
    let vm = vm::Runtime::new(fw);
    let mut bot = AliveBot::new(&mut world.rng, vm);

    bot.log("uploaded and queued".into());

    let id = loop {
        let id = BotId::new(&mut world.rng);

        if !world.bots.contains(id) {
            break id;
        }
    };

    if world.bots.queued.len() < world.policy.max_queued_bots {
        world.bots.queued.push(QueuedBot {
            id,
            bot,
            requeued: false,
        });

        debug!(?id, "bot queued");

        Ok(id)
    } else {
        debug!(?id, "bot discarded (queue full)");

        Err(anyhow!("too many robots queued, try again in a moment"))
    }
}
