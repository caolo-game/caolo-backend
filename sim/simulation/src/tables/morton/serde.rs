use super::MortonTable;
use super::TableRow;
use crate::prelude::Axial;
use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::fmt;
use std::marker::PhantomData;

impl<Row> Serialize for MortonTable<Row>
where
    Row: TableRow + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("MortonTable", 1)?;
        state.serialize_field("values", &self.values)?;
        state.end()
    }
}

struct MortonVisitor<V>
where
    V: TableRow,
{
    _m: PhantomData<(Axial, V)>,
}

impl<'de, Row> Visitor<'de> for MortonVisitor<Row>
where
    Row: TableRow + Deserialize<'de>,
{
    type Value = MortonTable<Row>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a single 'values' field containing a list of [Axial, Row] tuples")
    }

    fn visit_map<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        use std::borrow::Cow;
        let mut values: Option<Vec<(Axial, Row)>> = None;
        'a: while let Some(key) = seq.next_key::<Cow<String>>()? {
            if key == Cow::Borrowed("values") {
                let value = seq.next_value()?;
                values = value;
                break 'a;
            }
        }
        let values = values.ok_or_else(|| de::Error::missing_field("values"))?;
        let len = values.len();
        MortonTable::from_vec(values).map_err(|e| {
            de::Error::invalid_length(
                len,
                &format!("Failed to build MortonTable {:?}", e).as_str(),
            )
        })
    }
}

impl<'de, Row> Deserialize<'de> for MortonTable<Row>
where
    Row: TableRow + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["values"];
        deserializer.deserialize_struct(
            "MortonTable",
            FIELDS,
            MortonVisitor::<Row> {
                _m: Default::default(),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Axial;
    use rand::{thread_rng, Rng};

    #[test]
    fn test_de_serialize() {
        let mut rng = thread_rng();

        let points = (0..128)
            .map(|_| {
                let a = Axial::new(rng.gen_range(0, 15_000), rng.gen_range(0, 15_000));
                let val = rng.gen_range(0.0f32, 128.0);
                (a, val)
            })
            .collect::<Vec<_>>();

        let table = MortonTable::from_iterator(points.iter().cloned()).unwrap();

        let s = serde_json::to_string(&table).unwrap();
        dbg!(&s);
        let res: MortonTable<f32> = serde_json::from_str(s.as_str()).unwrap();

        table
            .keys
            .iter()
            .zip(res.keys.iter())
            .for_each(|(a, b)| assert_eq!(a.0, b.0));

        for (p, v) in points.iter() {
            let a = table.at(*p).unwrap();
            let b = res.at(*p).unwrap();

            assert_eq!(a, b);
            assert_eq!(a, v);
        }
    }
}
