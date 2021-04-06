use crate::components::{DecayComponent, HpComponent};
use crate::indices::EntityId;
use crate::join;
use crate::profile;
use crate::storage::views::UnsafeView;
use crate::tables::JoinIterator;
use tracing::debug;

pub fn update(
    (mut hps, mut decays): (
        UnsafeView<EntityId, HpComponent>,
        UnsafeView<EntityId, DecayComponent>,
    ),
    (): (),
) {
    profile!("DecaySystem update");
    debug!("update decay system called");

    let decays = decays.iter_mut();
    let hps = hps.iter_mut();
    join!([decays, hps]).for_each(
        |(
            _id,
            (
                DecayComponent {
                    hp_amount,
                    interval,
                    ref mut time_remaining,
                },
                HpComponent { ref mut hp, .. },
            ),
        )| match time_remaining {
            0 => {
                *hp -= *hp.min(hp_amount);
                *time_remaining = *interval;
            }
            _ => {
                *time_remaining -= 1;
            }
        },
    );

    debug!("update decay system done");
}
