use wasm_bindgen::prelude::*;

use js_sys::{Array, JsString};

use serde::{Deserialize, Serialize};

use screeps::{constants::Part, JsObjectId};

use std::{collections::HashMap, iter::repeat};

use crate::{
	builds::CreepBuild,
	error::{Error, Result},
	jobs::CreepJob,
};

pub struct CreepParts(pub HashMap<Part, u8>);

impl CreepParts {
	pub fn fulfils_requirements(&self, requirements: &CreepParts) -> bool {
		for (part, required_num) in &requirements.0 {
			match self.0.get(&part) {
				None => return false,
				Some(num) => {
					if num < required_num {
						return false;
					}
				}
			}
		}
		true
	}
}

impl CreepParts {
	pub fn cost(&self) -> u32 {
		self.0
			.iter()
			.map(|(part, count)| part.cost() as u32 * *count as u32)
			.sum()
	}

	pub fn to_array(&self) -> Array {
		self.0
			.iter()
			.flat_map(|(part, count)| repeat(part).take(*count as usize))
			.copied()
			.map(JsValue::from)
			.collect::<Array>()
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreepState {
	pub build: CreepBuild,
	pub job: CreepJob,
}

impl CreepState {
	pub fn new(build: CreepBuild, job: CreepJob) -> Self { CreepState { build, job } }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct CreepTarget(String);

impl CreepTarget {
	pub fn from_id(id: &JsString) -> Self { Self(id.as_string().unwrap()) }

	pub fn to_value(&self) -> Result<JsValue> {
		JsObjectId::from(JsString::from(self.0.clone()))
			.resolve()
			.ok_or(Error::IDResolve)
	}
}

#[derive(Serialize, Deserialize)]
pub struct CreepOptions {
	pub memory: Option<CreepState>,
}
