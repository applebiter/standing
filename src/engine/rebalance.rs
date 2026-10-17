//! Automatic Peer Set Rebalancing.
//!
//! When the user's composite standing exceeds that of every active
//! Comparable, a new Comparable is introduced from the user's extended social
//! graph. Every choice is drawn from a generator seeded with the profile's
//! seed and the number of Comparables introduced so far, so results are
//! reproducible.

use std::collections::HashMap;

use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

use super::adjustment;
use super::constants::rebalance::{ABOVE_RANGE, BELOW_RANGE, MAX_ATTEMPTS, PROBABILITY_ABOVE};
use super::index::{Inputs, Reading};
use crate::model::rating;

/// A relational archetype. `{name}` is replaced with an existing
/// Comparable's name, `{field}` with the user's line of work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Template {
    pub key: &'static str,
    pub text: &'static str,
}

impl Template {
    fn needs_name(&self) -> bool {
        self.text.contains("{name}")
    }

    fn needs_field(&self) -> bool {
        self.text.contains("{field}")
    }
}

/// The archetype bank. Keys are stored with each Comparable introduced and
/// must never change.
pub const TEMPLATES: &[Template] = &[
    Template {
        key: "brother_in_law_boat",
        text: "{name}'s brother-in-law (the one with the boat)",
    },
    Template {
        key: "managers_manager",
        text: "Your manager's manager",
    },
    Template {
        key: "younger_in_field",
        text: "Someone ten years younger than you in {field}",
    },
    Template {
        key: "high_school_remembers",
        text: "A person from your high school you don't remember, who remembers you",
    },
    Template {
        key: "sisters_new_partner",
        text: "Your sister's new partner",
    },
    Template {
        key: "retired_last_year",
        text: "A person your age who retired last year",
    },
    Template {
        key: "conference_friend",
        text: "{name}'s friend from the conference",
    },
    Template {
        key: "college_roommate",
        text: "{name}'s college roommate",
    },
    Template {
        key: "orientation_neighbor",
        text: "The person who sat next to you at orientation",
    },
    Template {
        key: "former_intern",
        text: "Your former intern, now your peer",
    },
    Template {
        key: "neighbor_extension",
        text: "{name}'s neighbor with the extension",
    },
    Template {
        key: "pickup_marathoner",
        text: "The parent at school pickup who runs marathons",
    },
    Template {
        key: "cousins_husband",
        text: "Your cousin's husband, recently promoted",
    },
    Template {
        key: "left_to_start_something",
        text: "A former colleague who left to start something",
    },
    Template {
        key: "field_podcast",
        text: "Someone in {field} with a podcast",
    },
    Template {
        key: "climbing_gym_friend",
        text: "{name}'s new friend from the climbing gym",
    },
    Template {
        key: "got_the_job",
        text: "The person who got the job you interviewed for",
    },
    Template {
        key: "exs_partner",
        text: "Your ex's current partner",
    },
    Template {
        key: "now_on_boards",
        text: "Someone you went to school with who now sits on boards",
    },
    Template {
        key: "dentist_socially",
        text: "Your dentist, socially",
    },
    Template {
        key: "successful_sibling",
        text: "{name}'s sibling, the successful one",
    },
    Template {
        key: "partners_work_friend",
        text: "Your partner's friend from work",
    },
    Template {
        key: "happy_wedding_couple",
        text: "The couple from the wedding who seemed so happy",
    },
    Template {
        key: "field_book",
        text: "Someone your age in {field} who has already written the book",
    },
    Template {
        key: "landlord",
        text: "Your landlord",
    },
];

/// Whether a Comparable should be introduced: the user's composite exceeds
/// every active Comparable's, there is at least one, and none has been
/// introduced today.
pub fn due(reading: &Reading, introduced_today: bool) -> bool {
    let Some(own) = reading.own_composite else {
        return false;
    };
    !introduced_today
        && !reading.comparables.is_empty()
        && reading
            .comparables
            .iter()
            .all(|c| c.composite.is_some_and(|theirs| own > theirs))
}

/// An archetype ready to be added to the peer set.
#[derive(Debug, Clone, PartialEq)]
pub struct Archetype {
    pub template: &'static str,
    pub name: String,
    /// Provisional raw ratings, by dimension.
    pub ratings: HashMap<String, f64>,
}

/// What an introduction may draw on.
pub struct Context<'a> {
    pub seed: i64,
    /// Comparables introduced before this one.
    pub introduced: u64,
    /// Names available to `{name}` templates.
    pub names: &'a [String],
    pub field: Option<&'a str>,
    /// Template keys already used; avoided while any remain.
    pub used: &'a [String],
}

fn rng(seed: i64, introduced: u64) -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(
        (seed as u64).wrapping_add(introduced.wrapping_mul(0x9E37_79B9_7F4A_7C15)),
    )
}

/// Chooses and fills a template, and generates provisional ratings that place
/// the archetype ahead of the user.
pub fn introduce(ctx: &Context<'_>, inputs: &Inputs) -> Archetype {
    let mut rng = rng(ctx.seed, ctx.introduced);
    let field = ctx.field.map(str::trim).filter(|f| !f.is_empty());
    let usable: Vec<&Template> = TEMPLATES
        .iter()
        .filter(|t| !t.needs_name() || !ctx.names.is_empty())
        .filter(|t| !t.needs_field() || field.is_some())
        .collect();
    let fresh: Vec<&Template> = usable
        .iter()
        .copied()
        .filter(|t| !ctx.used.iter().any(|u| u == t.key))
        .collect();
    let pool = if fresh.is_empty() { &usable } else { &fresh };
    let template = pool[rng.random_range(0..pool.len())];

    let mut name = template.text.to_owned();
    if template.needs_name() {
        let chosen = &ctx.names[rng.random_range(0..ctx.names.len())];
        name = name.replace("{name}", chosen);
    }
    if let Some(field) = field {
        name = name.replace("{field}", field);
    }

    let ratings = provisional_ratings(&mut rng, inputs);
    Archetype {
        template: template.key,
        name,
        ratings,
    }
}

/// For each applicable dimension the user has rated: usually somewhat above
/// the user's raw rating, otherwise slightly below. Regenerated until the
/// archetype's composite exceeds the user's.
fn provisional_ratings(rng: &mut ChaCha8Rng, inputs: &Inputs) -> HashMap<String, f64> {
    let rated: Vec<(&str, f64)> = inputs
        .dimensions
        .iter()
        .filter_map(|d| Some((d.id.as_str(), *inputs.own.get(&d.id)?)))
        .collect();
    let own = super::index::own_composite(inputs);
    for attempt in 0..MAX_ATTEMPTS {
        let ratings: HashMap<String, f64> = rated
            .iter()
            .map(|&(id, own_raw)| {
                let raw = if attempt + 1 == MAX_ATTEMPTS || rng.random_bool(PROBABILITY_ABOVE) {
                    own_raw + rng.random_range(ABOVE_RANGE)
                } else {
                    own_raw - rng.random_range(BELOW_RANGE)
                };
                (
                    id.to_owned(),
                    round_to_step(raw.clamp(rating::MIN, rating::MAX)),
                )
            })
            .collect();
        if exceeds(inputs, &ratings, own) {
            return ratings;
        }
    }
    unreachable!("the final attempt places every rating above the user's")
}

/// Ratings are recorded on the half-point scale.
fn round_to_step(v: f64) -> f64 {
    (v / rating::STEP).round() * rating::STEP
}

fn exceeds(inputs: &Inputs, ratings: &HashMap<String, f64>, own: Option<f64>) -> bool {
    let Some(own) = own else {
        return true;
    };
    let (sum, weights) = inputs
        .dimensions
        .iter()
        .filter_map(|d| Some((d.weight, adjustment::peer(*ratings.get(&d.id)?))))
        .fold((0.0, 0.0), |(s, w), (weight, v)| {
            (s + weight * v, w + weight)
        });
    weights > 0.0 && sum / weights > own
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::index::{ComparableReading, WeightedDimension};

    fn reading(own: Option<f64>, peers: &[Option<f64>]) -> Reading {
        Reading {
            computed_at: "2026-10-16T12:00:00Z".parse().unwrap(),
            own_composite: own,
            momentum: 0.0,
            psi: 100.0,
            comparables: peers
                .iter()
                .enumerate()
                .map(|(i, &composite)| ComparableReading {
                    id: i as i64,
                    composite,
                    momentum: 0.0,
                    value: 100.0,
                })
                .collect(),
        }
    }

    fn inputs(own: &[(&str, f64)]) -> Inputs {
        Inputs {
            dimensions: own
                .iter()
                .map(|&(id, _)| WeightedDimension {
                    id: id.into(),
                    weight: 1.0,
                })
                .chain(std::iter::once(WeightedDimension {
                    id: "UNRATED".into(),
                    weight: 1.0,
                }))
                .collect(),
            own: own.iter().map(|&(d, v)| (d.to_owned(), v)).collect(),
            ..Default::default()
        }
    }

    #[test]
    fn due_only_when_leading_everyone() {
        assert!(due(&reading(Some(6.0), &[Some(5.0), Some(5.9)]), false));
        assert!(!due(&reading(Some(6.0), &[Some(5.0), Some(6.0)]), false));
        assert!(!due(&reading(Some(6.0), &[Some(5.0), Some(7.0)]), false));
        assert!(!due(&reading(Some(6.0), &[]), false));
        assert!(!due(&reading(None, &[None]), false));
    }

    #[test]
    fn at_most_one_a_day() {
        assert!(!due(&reading(Some(6.0), &[Some(1.0)]), true));
    }

    #[test]
    fn the_bank_is_large_and_keys_are_unique() {
        assert!(TEMPLATES.len() >= 20);
        let mut keys: Vec<_> = TEMPLATES.iter().map(|t| t.key).collect();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), TEMPLATES.len());
    }

    fn ctx<'a>(introduced: u64, names: &'a [String], used: &'a [String]) -> Context<'a> {
        Context {
            seed: 42,
            introduced,
            names,
            field: Some("accounting"),
            used,
        }
    }

    #[test]
    fn introductions_are_reproducible_and_vary() {
        let names = vec!["Dana".to_owned(), "Marcus".to_owned()];
        let i = inputs(&[("A", 7.0), ("B", 3.0), ("C", 10.0)]);
        let a = introduce(&ctx(0, &names, &[]), &i);
        assert_eq!(a, introduce(&ctx(0, &names, &[]), &i));
        let firsts: std::collections::HashSet<_> = (0..10)
            .map(|n| introduce(&ctx(n, &names, &[]), &i).template)
            .collect();
        assert!(firsts.len() > 3);
    }

    #[test]
    fn placeholders_are_filled() {
        let names = vec!["Dana".to_owned()];
        let i = inputs(&[("A", 5.0)]);
        for n in 0..60 {
            let a = introduce(&ctx(n, &names, &[]), &i);
            assert!(!a.name.contains('{'), "{}", a.name);
        }
    }

    #[test]
    fn field_templates_need_a_field_and_name_templates_need_names() {
        let i = inputs(&[("A", 5.0)]);
        for n in 0..60 {
            let a = introduce(
                &Context {
                    seed: 7,
                    introduced: n,
                    names: &[],
                    field: None,
                    used: &[],
                },
                &i,
            );
            let t = TEMPLATES.iter().find(|t| t.key == a.template).unwrap();
            assert!(!t.needs_field() && !t.needs_name(), "{}", t.key);
        }
    }

    #[test]
    fn used_templates_are_avoided_while_others_remain() {
        let names = vec!["Dana".to_owned()];
        let i = inputs(&[("A", 5.0)]);
        let used: Vec<String> = TEMPLATES
            .iter()
            .filter(|t| t.key != "landlord")
            .map(|t| t.key.to_owned())
            .collect();
        assert_eq!(introduce(&ctx(3, &names, &used), &i).template, "landlord");
    }

    #[test]
    fn archetypes_always_exceed_the_user() {
        let names = vec!["Dana".to_owned()];
        for own in [
            &[("A", 10.0), ("B", 10.0)][..],
            &[("A", 0.0)],
            &[("A", 4.5), ("B", 9.0), ("C", 1.0)],
        ] {
            let i = inputs(own);
            let own_composite = crate::engine::index::own_composite(&i);
            for n in 0..25 {
                let a = introduce(&ctx(n, &names, &[]), &i);
                assert!(exceeds(&i, &a.ratings, own_composite));
                assert_eq!(a.ratings.len(), own.len(), "only self-rated dimensions");
                assert!(
                    a.ratings
                        .values()
                        .all(|v| (0.0..=10.0).contains(v) && (v * 2.0).fract() == 0.0)
                );
            }
        }
    }
}
