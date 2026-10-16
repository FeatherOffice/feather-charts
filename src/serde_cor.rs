//! Serialização de `Cor` como texto `#rrggbb` ou `#rrggbbaa`, para os modelos
//! JSON. `Cor` vem do `feather-draw`, que não conhece serde.
use feather_draw::Cor;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serializer};

pub fn para_texto(c: &Cor) -> String {
    if c.a == 255 {
        format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
    } else {
        format!("#{:02x}{:02x}{:02x}{:02x}", c.r, c.g, c.b, c.a)
    }
}

pub fn de_texto(t: &str) -> Result<Cor, String> {
    let h = t.strip_prefix('#').ok_or_else(|| format!("cor sem '#': {t}"))?;
    if !(h.len() == 6 || h.len() == 8) || !h.is_ascii() {
        return Err(format!("cor inválida: {t}"));
    }
    let b = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).map_err(|_| format!("cor inválida: {t}"));
    Ok(Cor::rgba(b(0)?, b(2)?, b(4)?, if h.len() == 8 { b(6)? } else { 255 }))
}

pub mod cor {
    use super::*;
    pub fn serialize<S: Serializer>(c: &Cor, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&para_texto(c))
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Cor, D::Error> {
        de_texto(&String::deserialize(d)?).map_err(D::Error::custom)
    }
}

pub mod opt {
    use super::*;
    pub fn serialize<S: Serializer>(c: &Option<Cor>, s: S) -> Result<S::Ok, S::Error> {
        match c {
            Some(c) => s.serialize_some(&para_texto(c)),
            None => s.serialize_none(),
        }
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Cor>, D::Error> {
        Option::<String>::deserialize(d)?
            .map(|t| de_texto(&t).map_err(D::Error::custom))
            .transpose()
    }
}

pub mod lista {
    use super::*;
    pub fn serialize<S: Serializer>(v: &[Cor], s: S) -> Result<S::Ok, S::Error> {
        s.collect_seq(v.iter().map(para_texto))
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Cor>, D::Error> {
        Vec::<String>::deserialize(d)?
            .iter()
            .map(|t| de_texto(t).map_err(D::Error::custom))
            .collect()
    }
}
