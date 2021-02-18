use serde::{Deserialize, Serialize};

use screeps::constants::Part;

use std::{
	collections::HashMap,
	fmt::{Display, Formatter, Result},
	iter::repeat,
};

use crate::structures::CreepParts;

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum CreepBuild {
	Worker1_1,
	Worker2_1,
	Worker2_2,
}

impl Display for CreepBuild {
	fn fmt(&self, f: &mut Formatter) -> Result { write!(f, "{:?}", self) }
}

impl CreepBuild {
	pub fn parts(&self) -> CreepParts {
		CreepParts(
			match self {
				CreepBuild::Worker1_1 => vec![(Part::Move, 1), (Part::Work, 1), (Part::Carry, 1)],

				CreepBuild::Worker2_1 => vec![(Part::Move, 2), (Part::Work, 1), (Part::Carry, 2)],
				CreepBuild::Worker2_2 => vec![(Part::Move, 3), (Part::Work, 2), (Part::Carry, 4)],
			}
			.iter()
			.copied()
			.collect::<HashMap<_, _>>(),
		)
	}
}
