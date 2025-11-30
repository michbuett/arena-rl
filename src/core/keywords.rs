use std::marker::PhantomData;

use bevy::prelude::*;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum ActionKeyword {
    Action,
    Reaction,
    Physical,
    Melee,
    ActWithPhysicalStr,
    ActWithPhysicalAg,
    ActWithMentalStr,
    ActWithMentalAg,
}

impl Into<u64> for ActionKeyword {
    fn into(self) -> u64 {
        self as u64
    }
}

#[derive(Debug, Clone, Copy)]
pub struct KeywordSet<K: Into<u64> + Copy>(u64, PhantomData<K>);

impl<'de> Deserialize<'de> for KeywordSet<ActionKeyword> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let vec: Vec<ActionKeyword> = Vec::deserialize(deserializer)?;
        Ok(KeywordSet::new(&vec))
    }
}

impl<K: Into<u64> + Copy> KeywordSet<K> {
    pub fn empty() -> Self {
        Self(0, PhantomData)
    }

    pub fn all() -> Self {
        Self(u64::MAX, PhantomData)
    }

    pub fn new(keywords: &Vec<K>) -> Self {
        let mut result = Self::empty();

        for keyword in keywords.iter() {
            result = result.add(*keyword);
        }

        return result;
    }

    pub fn add(mut self, keyword: K) -> Self {
        let bit: u64 = keyword.into();
        self.0 = self.0 | (1 << bit);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn contains(&self, other: KeywordSet<K>) -> bool {
        other.is_empty() || (self.0 & other.0) == other.0
    }

    pub fn has(&self, keyword: K) -> bool {
        let bit: u64 = keyword.into();
        let v = self.0 & (1 << bit);
        v > 0
    }
}

#[macro_export]
macro_rules! keyword_set {
    ( $x:expr $( , $more:expr )* ) => {
        crate::core::KeywordSet::new(&vec![$x $(, $more )* ])
    };
}

// macro_rules! keyword_insert {
//     ($set:expr) => {
//         $set
//     };

//     // ($set:expr, $kw:expr) => {
//     //     $set.insert($kw)
//     // };

//     ($set:expr, $kw:expr $(, $more:expr )*) => {{
//         // keyword_insert!($set, $kw);
//         $set.insert($kw);
//         keyword_insert!($set $(, $more )* )
//     }};
// }

#[test]
fn can_keyword_set_from_vec_of_keywords() {
    use ActionKeyword::*;
    let set1 = KeywordSet::new(&vec![Physical]);
    let set2 = KeywordSet::new(&vec![Physical, Melee, Physical]);
    let set3: KeywordSet<ActionKeyword> = KeywordSet::new(&vec![]);

    assert_eq!(false, set1.is_empty());
    assert_eq!(true, set1.has(Physical));
    assert_eq!(false, set1.has(Melee));
    assert_eq!(true, set1.contains(set1));
    assert_eq!(false, set1.contains(set2));
    assert_eq!(true, set1.contains(set3));

    assert_eq!(false, set2.is_empty());
    assert_eq!(true, set2.has(Physical));
    assert_eq!(true, set2.has(Melee));
    assert_eq!(true, set2.contains(set1));
    assert_eq!(true, set2.contains(set2));
    assert_eq!(true, set2.contains(set3));

    assert_eq!(true, set3.is_empty());
    assert_eq!(false, set3.has(Physical));
    assert_eq!(false, set3.has(Melee));
    assert_eq!(false, set3.contains(set1));
    assert_eq!(false, set3.contains(set2));
    assert_eq!(true, set3.contains(set3));
}

#[test]
fn can_keyword_set_with_macro() {
    use ActionKeyword::*;
    let set1 = keyword_set![Action];
    let set2 = keyword_set![Action, Physical, Melee];

    assert!(set1.has(Action));
    assert!(set2.has(Action));
    assert!(set2.has(Physical));
    assert!(set2.has(Melee));
}
