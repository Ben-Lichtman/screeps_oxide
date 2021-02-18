use wasm_bindgen::prelude::*;

use js_sys::{Array, JsString};

use serde::{Deserialize, Serialize};

use screeps::{
	constants::{Part, ResourceType, ReturnCode, StructureType},
	objects::{ConstructionSite, Creep, Flag, Room, RoomObject, Structure, StructureSpawn},
	StructureExtension,
};

use num_traits::cast::FromPrimitive;

use std::collections::HashMap;

use crate::{
	error::{Error, Result},
	jobs::Job,
	structures::{CreepParts, CreepTarget},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DistributeEnergyState {
	Entry,
	Distributing(CreepTarget),
	Building(CreepTarget),
	Upgrading,
	Done,
}

impl Default for DistributeEnergyState {
	fn default() -> Self { Self::Entry }
}

impl Job for DistributeEnergyState {
	fn finished(&self) -> bool {
		if let DistributeEnergyState::Done = self {
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
			DistributeEnergyState::Entry => {
				let pos = creep.pos().ok_or(Error::Unknown)?;
				let room = creep.room().ok_or(Error::Unknown)?;

				// Find target using order of priorities

				let energy_capacity = room.energy_capacity_available();
				let energy_available = room.energy_available();

				if energy_available < energy_capacity {
					// Distribute to spawner or extensions

					let spawns = spawns
						.iter()
						.filter(|x| x.store().get_free_capacity(Some(ResourceType::Energy)) > 0)
						.collect::<Vec<_>>();

					let extensions = structures
						.iter()
						.filter(|x| x.structure_type() == StructureType::Extension)
						.map(JsValue::from)
						.map(StructureExtension::from)
						.filter(|x| x.store().get_free_capacity(Some(ResourceType::Energy)) > 0)
						.collect::<Vec<_>>();

					let combined = spawns
						.into_iter()
						.map(JsValue::from)
						.chain(extensions.into_iter().map(JsValue::from))
						.collect::<Array>();

					let closest = pos
						.find_closest_by_path(&combined, None)
						.map(JsValue::from)
						.map(Structure::from)
						.ok_or(Error::NoneFound)?;

					let target = CreepTarget::from_id(&closest.id());

					*self = DistributeEnergyState::Distributing(target);
					creep.say(&JsString::from("Distribute"), false);

					Ok(())
				}
				else if !construction_sites.is_empty() {
					// Help build construction sites

					let array = construction_sites
						.iter()
						.map(JsValue::from)
						.collect::<Array>();

					let closest = pos
						.find_closest_by_path(&array, None)
						.map(JsValue::from)
						.map(ConstructionSite::from)
						.ok_or(Error::NoneFound)?;

					let target = CreepTarget::from_id(&closest.id().ok_or(Error::NoneFound)?);

					*self = DistributeEnergyState::Building(target);
					creep.say(&JsString::from("Build"), false);

					Ok(())
				}
				else if room.controller().is_some() {
					// Upgrade the controller

					*self = DistributeEnergyState::Upgrading;
					creep.say(&JsString::from("Upgrade"), false);

					Ok(())
				}
				else {
					*self = DistributeEnergyState::Done;
					Ok(())
				}
			}
			DistributeEnergyState::Distributing(target) => {
				let store = creep.store();
				let energy_used_capacity = store.get_used_capacity(Some(ResourceType::Energy));
				if energy_used_capacity == 0 {
					*self = DistributeEnergyState::Done;
					return Ok(());
				}
				let target = match target.to_value() {
					Ok(t) => t,
					Err(_) => {
						// Lost target - retarget
						*self = DistributeEnergyState::Entry;
						return Ok(());
					}
				};

				match ReturnCode::from_i8(creep.transfer(
					&RoomObject::from(JsValue::from(&target)),
					ResourceType::Energy,
					None,
				))
				.unwrap()
				{
					ReturnCode::Ok => Ok(()),
					ReturnCode::NotInRange => {
						creep.move_to(&target, None);
						Ok(())
					}
					ReturnCode::Full => {
						// Full target - retarget
						*self = DistributeEnergyState::Entry;
						Ok(())
					}
					x => Err(Error::UnhandledErrorCode(x)),
				}
			}
			DistributeEnergyState::Building(target) => {
				let store = creep.store();
				let energy_used_capacity = store.get_used_capacity(Some(ResourceType::Energy));
				if energy_used_capacity == 0 {
					*self = DistributeEnergyState::Done;
					return Ok(());
				}
				let target = match target.to_value() {
					Ok(t) => t,
					Err(_) => {
						// Lost target - retarget
						*self = DistributeEnergyState::Entry;
						return Ok(());
					}
				};

				match ReturnCode::from_i8(
					creep.build(&ConstructionSite::from(JsValue::from(&target))),
				)
				.unwrap()
				{
					ReturnCode::Ok => Ok(()),
					ReturnCode::NotInRange => {
						creep.move_to(&target, None);
						Ok(())
					}
					x => Err(Error::UnhandledErrorCode(x)),
				}
			}
			DistributeEnergyState::Upgrading => {
				let store = creep.store();
				let energy_used_capacity = store.get_used_capacity(Some(ResourceType::Energy));
				if energy_used_capacity == 0 {
					*self = DistributeEnergyState::Done;
					return Ok(());
				}

				let room = creep.room().ok_or(Error::Unknown)?;

				let controller = room.controller().unwrap();
				match ReturnCode::from_i8(creep.upgrade_controller(&controller)).unwrap() {
					ReturnCode::Ok => Ok(()),
					ReturnCode::NotInRange => {
						creep.move_to(&controller, None);
						Ok(())
					}
					x => Err(Error::UnhandledErrorCode(x)),
				}
			}
			DistributeEnergyState::Done => Ok(()),
		}
	}
}
