use wasm_bindgen::prelude::*;

use js_sys::JsString;

use serde::{Deserialize, Serialize};

use screeps::{
	constants::{Find, Part, ResourceType, ReturnCode},
	objects::{ConstructionSite, Creep, Flag, Room, RoomObject, Structure, StructureSpawn},
	Source,
};

use num_traits::cast::FromPrimitive;

use std::collections::HashMap;

use crate::{
	error::{Error, Result},
	jobs::Job,
	structures::{CreepParts, CreepTarget},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum HarvestState {
	Entry,
	Harvesting(CreepTarget),
	Done,
}

impl Default for HarvestState {
	fn default() -> Self { Self::Entry }
}

impl Job for HarvestState {
	fn finished(&self) -> bool {
		if let HarvestState::Done = self {
			true
		}
		else {
			false
		}
	}

	fn min_required(&self) -> CreepParts {
		CreepParts(
			[(Part::Move, 1), (Part::Work, 1), (Part::Carry, 1)]
				.iter()
				.copied()
				.collect::<HashMap<_, _>>(),
		)
	}

	fn drive(
		&mut self,
		creep: &Creep,
		construction_sites: &[ConstructionSite],
		flags: &[Flag],
		rooms: &[Room],
		spawns: &[StructureSpawn],
		structures: &[Structure],
	) -> Result<()> {
		match self {
			HarvestState::Entry => {
				// Find target
				let pos = creep.pos().ok_or(Error::Unknown)?;

				let closest = pos
					.find_closest_by_path(&JsValue::from(Find::Sources as i32), None)
					.ok_or(Error::NoneFound)?;

				let closest = Source::from(JsValue::from(closest));

				let target = CreepTarget::from_id(&closest.id());

				*self = HarvestState::Harvesting(target);
				creep.say(&JsString::from("Harvest"), false);
				Ok(())
			}
			HarvestState::Harvesting(target) => {
				let store = creep.store();
				let energy_free_capacity = store.get_free_capacity(Some(ResourceType::Energy));
				if energy_free_capacity == 0 {
					*self = HarvestState::Done;
					return Ok(());
				}
				let target = match target.to_value() {
					Ok(t) => t,
					Err(_) => {
						// Lost target - retarget
						*self = HarvestState::Entry;
						return Ok(());
					}
				};
				match ReturnCode::from_i8(creep.harvest(&RoomObject::from(JsValue::from(&target))))
					.unwrap()
				{
					ReturnCode::Ok => Ok(()),
					ReturnCode::NotInRange => {
						creep.move_to(&target, None);
						Ok(())
					}
					ReturnCode::Busy => Ok(()),
					ReturnCode::NotEnough => {
						*self = HarvestState::Entry;
						Ok(())
					}
					x => Err(Error::UnhandledErrorCode(x)),
				}
			}
			HarvestState::Done => Ok(()),
		}
	}
}
