use serde::{de::Deserialize, de::Deserializer, ser::Serialize, ser::Serializer};

use super::DiagDur;
use chrono::Duration;

impl Serialize for DiagDur {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let days = self.0.num_days();
        let hours = self.0.num_hours() % 24;
        let seconds = self.0.num_seconds() % (24 * 60) % 60;

        let costume = DiagDurSerde {
            days,
            hours,
            seconds,
        };

        costume.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DiagDur {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let costume: DiagDurSerde = DiagDurSerde::deserialize(deserializer)?;

        let dur = Duration::days(costume.days)
            + Duration::hours(costume.hours)
            + Duration::seconds(costume.seconds);

        Ok(Self(dur))
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct DiagDurSerde {
    days: i64,
    hours: i64,
    seconds: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{assert_tokens, Token};

    #[test]
    fn test_tokens() {
        let a = DiagDur(Duration::days(12) + Duration::hours(23) + Duration::seconds(42));

        assert_tokens(
            &a,
            &[
                Token::Struct {
                    name: "DiagDurSerde",
                    len: 3,
                },
                Token::Str("days"),
                Token::I64(12),
                Token::Str("hours"),
                Token::I64(23),
                Token::Str("seconds"),
                Token::I64(42),
                Token::StructEnd,
            ],
        );
    }
}
