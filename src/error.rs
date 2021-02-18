use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
	#[error(transparent)]
	Serialize(#[from] serde_json::error::Error),
	#[error("could not read state of creep {0} at [{1}, {2}] ")]
	Deserialize(String, u8, u8),
	#[error("could not resolve ID to value")]
	IDResolve,
	#[error("No targets found")]
	NoneFound,
	#[error("Encountered an unhandled error code while performing action: {0:?}")]
	UnhandledErrorCode(screeps::constants::ReturnCode),
	#[error("Unknown error")]
	Unknown,
}
