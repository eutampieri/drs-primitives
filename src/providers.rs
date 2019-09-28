pub struct TrainStation {}
pub struct TrainTrip {}

pub trait BikeSharingStation{
	fn get_position(&self) -> super::Coord;
	fn get_name(&self) -> Option<String>;
	fn get_id(&self) -> String;
	fn has_bikes(&self) -> Option<bool>;
}

pub trait TrainServiceProvider {
	fn get_name(&self) -> &str;
	fn get_id(&self) -> String;
	fn get_nearest_station(&self, from: &super::Coord) -> TrainStation;
	fn get_trip(&self, from: &TrainStation, to: &TrainStation, at: u64) -> TrainTrip;
	fn get_train_delay() -> Option<i32>;
}

pub trait BikeSharingServiceProvider {
	fn get_name(&self) -> String;
	fn get_id(&self) -> String;
	fn get_nearest_station(&self, from: &super::Coord) -> Box<dyn BikeSharingStation>;
	fn get_station_list(&self) -> Vec<Box<dyn BikeSharingStation>>;
}
