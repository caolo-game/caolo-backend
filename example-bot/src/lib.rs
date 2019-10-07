use caolo_api::{
    bots::{
        get_my_bots, move_to, send_dropoff_intent, send_mine_intent, send_move_intent, Bot,
        DropoffIntent, MineIntent, MoveIntent,
    },
    point::{Circle, Point},
    print, rand_range,
    resources::{find_resources_in_range, Resource, ResourceType},
    structures::{get_my_structures, send_spawn_intent, SpawnIntent, Structure},
};

const RESOURCE_SEARCH_RANGE: u32 = 40;

fn move_bot_randomly(bot: Bot) {
    let x = rand_range(-1, 2);
    let mut y = rand_range(0, 2);

    if x != 0 && y != 0 {
        y *= -x;
    }

    let delta = Point::new(x, y);

    let intent = MoveIntent {
        id: bot.id,
        position: bot.position + delta,
    };
    let res = send_move_intent(intent);
    print(&format!("Move result {:?} {:?}", bot, res));
}

fn dropoff(bot: Bot) {
    print("running dropoff");
    let spawn = get_my_structures()
        .into_iter()
        .filter_map(|structure| match structure {
            Structure::Spawn(s) => Some(s),
        })
        .next()
        .expect("spawn");

    if spawn.position.hex_distance(bot.position) > 1 {
        print("moving to spawn");
        move_to(&bot, spawn.position);
    } else {
        print("transferring to spawn");
        let res = send_dropoff_intent(DropoffIntent {
            id: bot.id,
            target: spawn.id,
            amount: bot.carry,
            ty: ResourceType::Mineral,
        });
        print(&format!("transferring to spawn done {:?}", res));
    }

    print("dropoff done");
}

fn run_bot(bot: Bot) {
    if bot.carry == bot.carry_max {
        return dropoff(bot);
    }
    let resources = find_resources_in_range(Circle {
        center: bot.position,
        radius: RESOURCE_SEARCH_RANGE,
    });
    if resources
        .as_ref()
        .map(|r| r.resources.len() == 0)
        .unwrap_or(true)
    {
        move_bot_randomly(bot);
    } else {
        let resources = resources.unwrap().resources;
        let target = resources
            .iter()
            .filter(|r| match r {
                Resource::Mineral(r) => r.energy > 0,
            })
            .next(); // resources are sorted by their distance from 'center' so the first one is closest
        match target {
            Some(Resource::Mineral(target)) => {
                if target.position.hex_distance(bot.position) > 1 {
                    let res = move_to(&bot, target.position);
                    print(&format!("MoveTo result {:?}", res));
                } else {
                    let intent = MineIntent {
                        id: bot.id,
                        target: target.id,
                    };
                    let res = send_mine_intent(intent);
                    print(&format!("Mine result {:?}", res));
                }
            }
            _ => {
                move_bot_randomly(bot);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn run() {
    print("Hello from wasm client!");

    for structure in get_my_structures() {
        match structure {
            Structure::Spawn(s) => {
                send_spawn_intent(SpawnIntent {
                    id: s.id,
                    bot: Bot {
                        speed: 1,
                        ..Default::default()
                    },
                });
            }
        }
    }

    let bots = get_my_bots();

    for bot in bots {
        run_bot(bot);
    }
}
