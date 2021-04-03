use serde::ser::{Serialize, Serializer};

use super::Axial;

impl Serialize for Axial {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let payload = [self.q, self.r];
        <[i32; 2] as Serialize>::serialize(&payload, serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{assert_tokens, Token};

    #[test]
    fn de_serialize_json() {
        let a = Axial::new(12, 69);

        let pl = serde_json::to_string(&a).unwrap();

        let b = serde_json::from_str(pl.as_str()).unwrap();

        assert_eq!(a, b);
    }

    #[test]
    fn de_serialize_yaml() {
        let a = Axial::new(12, 69);

        let pl = serde_yaml::to_string(&a).unwrap();

        let b = serde_yaml::from_str(pl.as_str()).unwrap();

        assert_eq!(a, b);
    }

    #[test]
    fn serde_tokens() {
        let a = Axial::new(1, 2);

        assert_tokens(
            &a,
            &[
                Token::Tuple { len: 2 },
                Token::I32(1),
                Token::I32(2),
                Token::TupleEnd,
            ],
        );
    }
}
