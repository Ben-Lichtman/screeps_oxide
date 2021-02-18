pub mod builds;
pub mod error;
pub mod jobs;
pub mod strategy;
pub mod structures;
pub mod util;

use wasm_bindgen::prelude::*;

use js_sys::Object;

use screeps::{
	objects::{ConstructionSite, Creep, Flag, Room, Structure, StructureSpawn},
	Game,
};

use std::collections::HashMap;

use crate::{
	error::Result,
	strategy::{drive_creeps, execute_strategy},
	util::{copy_state_in, copy_state_out, log},
};

#[wasm_bindgen(js_name = setup)]
pub fn setup_entry() {
	match setup() {
		Ok(_) => (),
		Err(e) => log(format!("Setup error: {:?}", e)),
	}
}

fn setup() -> Result<()> { Ok(()) }

#[wasm_bindgen(js_name = loop)]
pub fn game_loop_entry() {
	match game_loop() {
		Ok(_) => (),
		Err(e) => log(format!("Game loop error: {:?}", e)),
	}
}

fn game_loop() -> Result<()> {
	let construction_sites = Object::values(&Game::construction_sites())
		.iter()
		.map(ConstructionSite::from)
		.collect::<Vec<_>>();

	let creeps = Object::values(&Game::creeps())
		.iter()
		.map(Creep::from)
		.collect::<Vec<_>>();

	let flags = Object::values(&Game::flags())
		.iter()
		.map(Flag::from)
		.collect::<Vec<_>>();

	let rooms = Object::values(&Game::rooms())
		.iter()
		.map(Room::from)
		.collect::<Vec<_>>();

	let spawns = Object::values(&Game::spawns())
		.iter()
		.map(StructureSpawn::from)
		.collect::<Vec<_>>();

	let structures = Object::values(&Game::structures())
		.iter()
		.map(Structure::from)
		.collect::<Vec<_>>();

	// Pair each creep with its memory structure
	let mut creep_pairs = creeps
		.into_iter()
		.map(copy_state_in)
		.collect::<Result<Vec<_>>>()?;

	execute_strategy(
		&construction_sites,
		&mut creep_pairs,
		&flags,
		&rooms,
		&spawns,
		&structures,
	)?;

	drive_creeps(
		&construction_sites,
		&mut creep_pairs,
		&flags,
		&rooms,
		&spawns,
		&structures,
	)?;

	// Apply state changes
	creep_pairs
		.into_iter()
		.map(|(creep, state)| copy_state_out(&creep, state))
		.collect::<Result<()>>()?;

	Ok(())
}
