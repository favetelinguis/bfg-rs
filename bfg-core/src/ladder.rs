use std::collections::{btree_map::Entry, BTreeMap};

use num::FromPrimitive;

// LOB - limit order book consist of two sides, a side consist of multiple levels.
// a level is volume and an odds, odds is an array containing timestamps for each
// update at that level.
#[derive(Debug, Clone, Default)]
pub struct Stake {
    odds: Odds,
    timestamp: u8,
    amount: u8,
}
type Odds = u8;
// Is an ordered map
pub enum Side {
    Back(Stake),
    Lay(Stake),
}

// fractional ods num crate
// decimal odds not Ord?
// moneyline aka AmericanOdds
// TODO use Rational32 as key since float can not be key
#[derive(Debug, Clone, Default)]
pub struct MapLadder {
    // TODO also need some match value here?
    back: BTreeMap<Odds, Vec<Stake>>,
    lay: BTreeMap<Odds, Vec<Stake>>,
}

impl MapLadder {
    fn new() -> Self {
        Default::default()
    }
}

impl Ladder for MapLadder {
    // TODO need to seperate back/lay
    fn update(&mut self, side: Side) {
        match side {
            Side::Back(stake) => {
                self.back
                    .entry(stake.odds)
                    // .and_modify(|curr| curr.push(stake))
                    .or_default()
                    .push(stake);
            }
            Side::Lay(stake) => {
                self.lay.entry(stake.odds).or_default().push(stake);
                // .and_modify(|curr| curr.push(stake))
                // .or_insert(vec![stake]);
            }
        }
    }
}

pub trait Ladder {
    // Update orderbook at level
    // TODO take timestamp odds and volue
    // Update a value in the ladder, if the odds already
    // exist append if not create the initial entry.
    fn update(&mut self, side: Side);
    // Get the best n back or lay, observe that all 0 stake values must be ignored.
    // Must check the lasp value in the vec to get the most recent value.
    // fn top(&self, n: u8) -> Stake;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_ladder() {
        let mut sut = MapLadder::new();
        sut.update(Side::Back(Stake {
            odds: 3,
            timestamp: 4,
            amount: 5,
        }));
        let result = sut.back.get(&3);
        assert!(result.is_some(), "unable to insert update in ladder");
        assert!(result.unwrap().last().is_some(), "no element inserted");
    }
    // #[test]
    // fn fractional_to_decimal() {
    //     let odds = num::Rational32::from_f32(1.2 - 1.);
    // }

    // TODO read up on https://github.com/pnxenopoulos/implied
    #[test]
    fn decimal_to_fractional() {
        let odds = num::Rational32::from_f32(1.2 - 1.).unwrap();
        assert_eq!(odds.numer(), &1);
        assert_eq!(odds.denom(), &5);
    }
}
