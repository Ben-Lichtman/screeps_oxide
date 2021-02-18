use wasm_bindgen::prelude::*;

use js_sys::{JsString, Object, JSON::stringify};

use web_sys::console;

use screeps::{
	objects::{Creep, StructureSpawn},
	Game,
};

use crate::{
	builds::CreepBuild,
	error::{Error, Result},
	jobs::CreepJob,
	structures::{CreepOptions, CreepState},
};

pub fn log(x: impl Into<JsValue>) { console::log_1(&x.into()); }

pub fn log_pretty(x: impl Into<Object>) {
	let o = x.into();
	console::log_1(&stringify(&o).unwrap());
}

pub fn copy_state_in(creep: Creep) -> Result<(Creep, CreepState)> {
	let state = creep
		.memory()
		.clone()
		.into_serde::<CreepState>()
		.map_err(|_| {
			let name = creep.name().as_string().unwrap();
			let pos = creep.pos().unwrap();
			Error::Deserialize(name, pos.x(), pos.y())
		})?;
	Ok((creep, state))
}

pub fn copy_state_out(creep: &Creep, state: CreepState) -> Result<()> {
	creep.set_memory(&JsValue::from_serde(&state)?);

	Ok(())
}

pub fn spawn_creep(spawner: &StructureSpawn, build: &CreepBuild) {
	let spawner_name = spawner.name().as_string().unwrap();
	let name = JsString::from(format!("{}:{}:{}", build, spawner_name, Game::time()));

	let creep = CreepState::new(build.clone(), CreepJob::None);
	let creep_options = CreepOptions {
		memory: Some(creep.clone()),
	};

	spawner.spawn_creep(
		&creep.build.parts().to_array(),
		&name,
		Some(Object::from(JsValue::from_serde(&creep_options).unwrap())),
	);
}
