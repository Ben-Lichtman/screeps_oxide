mod distribute_energy;
mod harvest;

use wasm_bindgen::prelude::*;

use js_sys::Array;

use serde::{Deserialize, Serialize};

use screeps::objects::{ConstructionSite, Creep, Flag, Room, Structure, StructureSpawn};

use std::{
	hash::{Hash, Hasher},
	mem::discriminant,
};

use crate::{
	error::Result,
	structures::{CreepParts, CreepState},
};

pub trait Job: Default {
	fn finished(&self) -> bool;

	fn min_required(&self) -> CreepParts;

	fn drive(
		&mut self,
		creep: &Creep,
		construction_sites: &[ConstructionSite],
		flags: &[Flag],
		rooms: &[Room],
		spawns: &[StructureSpawn],
		structures: &[Structure],
	) -> Result<()>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CreepJob {
	None,
	Harvest(harvest::HarvestState),
	DistributeEnergy(distribute_energy::DistributeEnergyState),
}

impl PartialEq for CreepJob {
	fn eq(&self, other: &Self) -> bool { discriminant(self) == discriminant(other) }
}

impl Eq for CreepJob {}

impl Hash for CreepJob {
	fn hash<H: Hasher>(&self, state: &mut H) { discriminant(self).hash(state); }
}
