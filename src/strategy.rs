use wasm_bindgen::prelude::*;

use once_cell::sync::Lazy;

use screeps::{
	constants::ResourceType,
	objects::{ConstructionSite, Creep, Flag, Room, Structure, StructureSpawn},
};

use std::collections::HashMap;

use crate::{
	builds::CreepBuild,
	error::{Error, Result},
	jobs::{CreepJob, Job},
	structures::CreepState,
	util::{log, spawn_creep},
};

static SPAWNER_TIERS: Lazy<Vec<(u8, u32, HashMap<CreepBuild, u16>)>> = Lazy::new(|| {
	let mut ordered = {
		[
			// (min level, [(build, proportion)])
			(1, vec![(CreepBuild::Worker1_1, 1)]),
			(2, vec![(CreepBuild::Worker2_1, 1)]),
			(2, vec![(CreepBuild::Worker2_2, 1)]),
		]
		.iter()
		.map(|(min_level, vec)| {
			let max_cost = vec
				.iter()
				.map(|(build, _)| build.parts().cost())
				.max()
				.unwrap();
			let hash_map = vec.into_iter().cloned().collect::<HashMap<_, _>>();
			(*min_level, max_cost, hash_map)
		})
		.collect::<Vec<_>>()
	};
	ordered.sort_unstable_by(|(a_1, a_2, _), (b_1, b_2, _)| (b_1, b_2).cmp(&(a_1, a_2)));
	ordered
});

pub fn execute_strategy(
	construction_sites: &[ConstructionSite],
	creep_pairs: &mut [(Creep, CreepState)],
	flags: &[Flag],
	rooms: &[Room],
	spawns: &[StructureSpawn],
	structures: &[Structure],
) -> Result<()> {
	// Top level strategy dispatch

	let mut creeps_by_build = HashMap::<_, u16>::new();
	let mut creeps_by_job = HashMap::<_, u16>::new();

	creep_pairs.iter().for_each(|(_, state)| {
		creeps_by_build
			.entry(state.build.clone())
			.and_modify(|x| *x += 1)
			.or_insert(1);
		creeps_by_job
			.entry(state.job.clone())
			.and_modify(|x| *x += 1)
			.or_insert(1);
	});

	spawner_strategy(
		construction_sites,
		creep_pairs,
		flags,
		rooms,
		spawns,
		structures,
		&creeps_by_build,
		&creeps_by_job,
	)?;

	creep_strategy(
		construction_sites,
		creep_pairs,
		flags,
		rooms,
		spawns,
		structures,
		&creeps_by_build,
		&creeps_by_job,
	)?;

	Ok(())
}

fn spawner_strategy(
	construction_sites: &[ConstructionSite],
	creep_pairs: &mut [(Creep, CreepState)],
	flags: &[Flag],
	rooms: &[Room],
	spawns: &[StructureSpawn],
	structures: &[Structure],
	creeps_by_build: &HashMap<CreepBuild, u16>,
	creeps_by_job: &HashMap<CreepJob, u16>,
) -> Result<()> {
	spawns.iter().for_each(|spawner| {
		let room = spawner.room().unwrap();
		let controller = room.controller();
		let room_level = controller.unwrap().level();

		let room_energy_capacity = room.energy_capacity_available();
		let room_energy_available = room.energy_available();

		let creep_count = creep_pairs.len();

		let (_, _, recipe) = SPAWNER_TIERS
			.iter()
			.find(|(min_level, max_cost, _)| {
				room_level >= *min_level && room_energy_capacity >= *max_cost
			})
			.unwrap();

		let total_in_recipe = recipe.iter().map(|(_, proportion)| proportion).sum::<u16>();

		let total_in_world = creeps_by_build
			.iter()
			.filter(|(build, _)| recipe.contains_key(build))
			.map(|(_, count)| count)
			.sum::<u16>();

		let chosen = recipe
			.iter()
			.map(|(build, proportion)| {
				let world_proportion =
					*creeps_by_build.get(build).unwrap_or(&0) as f32 / total_in_world as f32;

				let wanted_proportion = *proportion as f32 / total_in_recipe as f32;

				(world_proportion / wanted_proportion, build)
			})
			.min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

		if let Some((_, build)) = chosen {
			match room_level {
				1 => {
					if total_in_world < 5 {
						spawn_creep(spawner, build);
					}
				}
				2 => {
					if total_in_world < 10 {
						spawn_creep(spawner, build);
					}
				}
				3 => {
					if total_in_world < 10 {
						spawn_creep(spawner, build);
					}
				}

				_ => {
					if total_in_world < 10 {
						spawn_creep(spawner, build);
					}
				}
			}
		}
	});

	Ok(())
}

fn creep_strategy(
	construction_sites: &[ConstructionSite],
	creep_pairs: &mut [(Creep, CreepState)],
	flags: &[Flag],
	rooms: &[Room],
	spawns: &[StructureSpawn],
	structures: &[Structure],
	creeps_by_build: &HashMap<CreepBuild, u16>,
	creeps_by_job: &HashMap<CreepJob, u16>,
) -> Result<()> {
	creep_pairs
		.iter_mut()
		.for_each(|(creep, state)| match &state.job {
			CreepJob::None => match &state.build {
				CreepBuild::Worker1_1 | CreepBuild::Worker2_1 | CreepBuild::Worker2_2 => {
					let store = creep.store();

					let energy_free_capacity = store.get_free_capacity(Some(ResourceType::Energy));
					let energy_used_capacity = store.get_used_capacity(Some(ResourceType::Energy));
					if energy_free_capacity >= energy_used_capacity {
						state.job = CreepJob::Harvest(Default::default());
					}
					else {
						state.job = CreepJob::DistributeEnergy(Default::default());
					}
				}
			},
			CreepJob::Harvest(job_state) => {
				if job_state.finished() {
					state.job = CreepJob::None
				}
			}
			CreepJob::DistributeEnergy(job_state) => {
				if job_state.finished() {
					state.job = CreepJob::None
				}
			}
		});

	Ok(())
}

pub fn drive_creeps(
	construction_sites: &[ConstructionSite],
	creep_pairs: &mut [(Creep, CreepState)],
	flags: &[Flag],
	rooms: &[Room],
	spawns: &[StructureSpawn],
	structures: &[Structure],
) -> Result<()> {
	for (creep, state) in creep_pairs {
		let res = match &mut state.job {
			CreepJob::None => Ok(()),
			CreepJob::Harvest(job_state) => {
				job_state.drive(creep, construction_sites, flags, rooms, spawns, structures)
			}
			CreepJob::DistributeEnergy(job_state) => {
				job_state.drive(creep, construction_sites, flags, rooms, spawns, structures)
			}
		};

		match res {
			Ok(_) => (),
			Err(e) => {
				let pos = creep.pos().unwrap();
				log(format!(
					"Error while driving creep at [{}, {}]: {}\nState: {:?}",
					pos.x(),
					pos.y(),
					e,
					state
				));
			}
		}
	}

	Ok(())
}
