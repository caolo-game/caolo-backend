use crate::components::{DecayComponent, HpComponent};
use crate::indices::EntityId;
use crate::join;
use crate::profile;
use crate::storage::views::UnsafeView;
use crate::tables::JoinIterator;
use tracing::{debug, trace};

pub fn decay_update(
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
                    time_remaining,
                },
                HpComponent { hp, .. },
            ),
        )| match time_remaining {
            0 => {
                *hp = (*hp).saturating_sub(*hp_amount);
                *time_remaining = *interval;
                trace!("Decayed entity {:?}. Current hp: {}", _id, *hp);
            }
            _ => {
                *time_remaining -= 1;
            }
        },
    );

    debug!("update decay system done");
}
