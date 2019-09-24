pub trait TrainServiceProvider {
	pub fn get_name(&self) -> &str;
	pub fn get_nearest_station(&self, from: &super:Coord) -> impl TrainStation;
	pub fn get_trip(&self, from: &impl TrainStation, to: &impl TrainStation, at: u64) -> TrainTrip;
	pub fn get_train_delay() -> Option<i16>;
}
