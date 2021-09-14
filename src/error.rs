#[derive(Debug, Fail)]
pub enum GymError {
	#[fail(display = "Invalid action")]
	InvalidAction,
	#[fail(display = "Invalid conversion")]
	InvalidConversion,
	#[fail(display = "Wrong type")]
	WrongType,
	#[fail(display = "Unable to parse step result")]
	WrongStepResult,
}