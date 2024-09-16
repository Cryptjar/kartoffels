use crate::{bots, Clock, Event, KillBot, Request, Shutdown, World};
use std::ops::ControlFlow;
use std::sync::Arc;
use tokio::sync::mpsc::error::TryRecvError;

pub fn run(world: &mut World) -> ControlFlow<Shutdown, ()> {
    loop {
        let request = match world.clock {
            Clock::Auto { .. } => world.rx.try_recv(),

            Clock::Manual { .. } => {
                if world.tick.is_some() {
                    world.rx.try_recv()
                } else {
                    world.rx.blocking_recv().ok_or(TryRecvError::Disconnected)
                }
            }
        };

        match request {
            Ok(Request::Tick { tx }) => {
                assert!(world.tick.is_none());

                world.tick = Some(tx);
            }

            Ok(Request::Pause { tx }) => {
                world.paused = true;
                _ = tx.send(());
            }

            Ok(Request::Resume { tx }) => {
                world.paused = false;
                _ = tx.send(());
            }

            Ok(Request::Shutdown { tx }) => {
                break ControlFlow::Break(Shutdown { tx: Some(tx) });
            }

            Ok(Request::CreateBot { src, pos, tx }) => {
                _ = tx.send(bots::create::run(world, src, pos));
            }

            Ok(Request::RestartBot { id, tx }) => {
                bots::kill::run(
                    world,
                    KillBot {
                        id,
                        reason: "forcefully restarted".into(),
                        killer: None,
                    },
                );

                _ = tx.send(());
            }

            Ok(Request::DestroyBot { id, tx }) => {
                world.bots.remove(id);

                _ = world.events.send(Arc::new(Event::BotKilled { id }));
                _ = tx.send(());
            }

            Ok(Request::SetSpawn { point, dir, tx }) => {
                world.spawn = (point, dir);
                _ = tx.send(());
            }

            Ok(Request::SetMap { map, tx }) => {
                world.map = map;
                _ = tx.send(());
            }

            Err(TryRecvError::Empty) => {
                break ControlFlow::Continue(());
            }

            Err(TryRecvError::Disconnected) => {
                break ControlFlow::Break(Shutdown { tx: None });
            }
        }
    }
}
