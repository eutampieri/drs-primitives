const TOLERANCE: f64 = 1e-9;
static SPATIAL_TOLERANCE: f64 = 0.5e-3; // 50 cm

#[derive(Copy,Clone,Debug)]
pub struct Coord{
    pub lat: f64,
    pub lon: f64
}
impl std::ops::Sub for Coord {
    type Output = Coord;
    fn sub(self, other: Coord) -> Coord {
        Coord{lat:self.lat-other.lat,lon:self.lon-other.lon}
    }
}
impl std::ops::Add for Coord {
    type Output = Coord;
    fn add(self, other: Coord) -> Coord {
        Coord{lat:self.lat+other.lat,lon:self.lon+other.lon}
    }
}
impl std::ops::Div<f64> for Coord {
    type Output = Coord;
    fn div(self, d: f64) -> Coord {
        Coord{lat:self.lat/d,lon:self.lon/d}
    }
}
impl std::ops::Mul<f64> for Coord {
    type Output = Coord;
    fn mul(self, m: f64) -> Coord {
        Coord{lat:self.lat*m,lon:self.lon*m}
    }
}
impl std::cmp::Ord for Coord {
    fn cmp(&self, other: &Coord) -> std::cmp::Ordering {
        if self.lon==other.lon {
            if self.lat<other.lat { std::cmp::Ordering::Less }
            else if self.lat>other.lat { std::cmp::Ordering::Greater }
            else { std::cmp::Ordering::Equal }
        } else {
            if self.lon<other.lon { std::cmp::Ordering::Less }
            else { std::cmp::Ordering::Greater }
        }
    }
}

pub enum Direction {
    Forward,
    Backward
}

pub type Point = Coord;

impl PartialEq for Coord{
    fn eq(&self, other: &Self) -> bool {
        self.lat == other.lat && self.lon == other.lon
    }
}

impl Coord{
    /// Get the distance between two points in km
    pub fn distance(&self, from: &Coord) -> f64{
        let p = std::f64::consts::PI / 180.0;
        let a = 0.5 - ((from.lat - self.lat) * p).cos()/2.0 +
            (self.lat * p).cos() * (from.lat * p).cos() *
            (1.0 - ((from.lon - self.lon) * p).cos())/2.0;
        12742.0 * a.sqrt().asin()
    }
    pub fn norm(&self) -> f64 {
        (self.lat*self.lat+self.lon*self.lon).sqrt()
    }
    pub fn normalized(&self) -> Coord {
        *self/self.norm()
    }
    pub fn dot(&self, o: Coord) -> f64 {
        o.lat*self.lat+o.lon*self.lon
    }
}

pub struct Segment {
    pub a: Coord,
    pub b: Coord,
    pub layer: Option<i8>,
}

impl Segment {
    pub fn into_tuple(&self) -> ((f64, f64), (f64, f64)){
        ((self.a.lat, self.a.lon), ((self.b.lat, self.b.lon)))
    }

    /// Check if two segments are contiguous
    /// (i.e. if the end of one segment corresponds with the beginning of another one
    /// or a segment touches another segment without intersecting it)
    pub fn is_contiguous(&self, with: &Segment) -> bool {
        if self.a==with.a && self.b==with.b {
            false
        } else {
            self.contains(&with.a)||self.contains(&with.b)||with.contains(&self.a)||with.contains(&self.b)
        }
    }

    /// Check if two segments intersect
    pub fn intersection(&self, other: &Segment) -> Option<Coord> {
        if let Some(_) = self.layer{
            if !self.is_contiguous(other) && self.layer !=other.layer{
                return None;
            }
        }
        let p1 = self.a;
        let p2 = self.b;
        let q1 = other.a;
        let q2 = other.b;
        let ap: f64 = p2.lon-p1.lon;
        let bp: f64 = p1.lat-p2.lat;
        let cp: f64 = -p2.lat*ap-p2.lon*bp;

        let aq: f64 = q2.lon-q1.lon;
        let bq: f64 = q1.lat-q2.lat;
        let cq: f64 = -q2.lat*aq-q2.lon*bq;

        let over: f64 = ap*bq-aq*bp;

        let ans = Coord{lat:(bp*cq-bq*cp)/over, lon:(aq*cp-ap*cq)/over};

        if self.contains(&ans) && other.contains(&ans) {
            Some(ans)
        } else {
            None
        }
    }

    /// Get the segment's length
    pub fn length(&self) -> f64 {
        self.a.distance(&self.b)
    }

    pub fn distance_from_point(&self, point: &Point) -> (f64, Coord) {
        let normb = (self.b-self.a).normalized();
        let a = *point-self.a;
        let res = normb*normb.dot(a)+self.a;
        if self.contains(&res) {
            (res.distance(point),res)
        } else {
            let distance_a = point.distance(&self.a);
            let distance_b = point.distance(&self.b);
            if distance_a < distance_b {
                (distance_a, self.a)
            } else {
                (distance_b, self.b)
            }
        }
    }

    /// Return a reversed Segment
    pub fn reverse(&self) -> Segment {
        Segment{a: self.b, b: self.a, layer: self.layer}
    }

    pub fn contains(&self, point: &Point) -> bool{
        if point.lon<(self.a.lon).min(self.b.lon)-TOLERANCE
        || point.lon>(self.a.lon).max(self.b.lon)+TOLERANCE
        || point.lat<(self.a.lat).min(self.b.lat)-TOLERANCE
        || point.lat>(self.a.lat).max(self.b.lat)+TOLERANCE
        {
            return false;
        }
        let a = *point-self.a;
        let b = self.b-self.a;
        (a.lon*b.lat-a.lat*b.lon).abs()<TOLERANCE
    }

    /// Check if a point lies on a segment, making sure it's not one of its ends
    pub fn strictly_contains(&self, point: &Point) -> bool{
        if (point.lat-self.a.lat).abs()<TOLERANCE&&(point.lon-self.a.lon).abs()<TOLERANCE {
            return false;
        }
        if (point.lat-self.b.lat).abs()<TOLERANCE&&(point.lon-self.b.lon).abs()<TOLERANCE {
            return false;
        }
        return self.contains(point);
    }
}

pub struct Road{
    pub segments: Vec<Segment>,
    pub name: Option<String>,
    pub forbidden_to_pedestrians: bool,
    pub forbidden_to_bikes: bool,
}
impl Road{

    pub fn center(&self) -> Coord {
        let mut result = Coord{lat: 0.0, lon: 0.0};
        let min = &self.segments[0].a;
        let max = &self.segments[&self.segments.len()-1].b;
        result.lat = (min.lat+max.lat)/2.0;
        result.lon = (min.lon+max.lon)/2.0;
        result
    }

    /// Get the road's total length
    pub fn length(&self) -> f64 {
        let mut total_length = 0.0;
        for segment in &self.segments{
            total_length += segment.length();
        }
        total_length
    }

    /// Get a tuple containing the distance from the nearest point of the road and the coordinates of the said point
    pub fn distance_from_nearest_point(&self, point: &Coord) -> (f64, Coord) {
        let mut min: (f64, Point) = (std::f64::MAX, Point{lat: 0.0, lon: 0.0});
        for segment in &self.segments{
            let tmp = segment.distance_from_point(&point);
            if tmp.0 < min.0 {
                min = tmp;
            }
        }
        min
    }

    /// Get the distance from a point to an end of the road
    pub fn length_from(&self, point: &Coord, direction: Direction) -> f64{
        //let nearest_point = self.distance_from_nearest_point(point).1;
        let mut distance = 0.0;
        match direction{
            Direction::Forward => {
                for segment in &self.segments{
                    if segment.strictly_contains(point){
                        distance += segment.a.distance(point);
                        break;
                    } else {
                        distance += segment.length();
                    }
                }
            },
            Direction::Backward => {
                for segment in self.segments.iter().rev(){
                    if segment.strictly_contains(point){
                        distance += segment.reverse().a.distance(point);
                        break;
                    } else {
                        distance += segment.length();
                    }
                }
            }
        }
        distance
    }

    /// Get the intersections with another road
    pub fn intersections(&self, with: &Road) -> Vec<Coord> {
        let mut result: Vec<Coord> = Vec::new();
        for a_segment in &self.segments {
            for b_segment in &with.segments {
                if let Some(intersection) = a_segment.intersection(b_segment){
                    if result.contains(&intersection) {
                        continue;
                    }
                    result.push(intersection)
                }
            }
        }
        result
    }
}

pub struct BusStop {
    pub position: Coord,
    pub id: String,
    pub name: String,
}

pub struct TrainStation {
    pub name: String,
    pub id: String,
    pub region_id: u8,
    pub position: Coord
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_point_from_segment_near_test() {
        let segment = Segment{
            a: Point{lat:1.0, lon: 1.0},
            b: Point{lat: 4.0, lon: 4.0},
            layer: None
        };
        let point = Point{lat: 1.0, lon: 3.0};

        assert_eq!(segment.distance_from_point(&point).1, Point{lat:2.0, lon: 2.0});
    }

    #[test]
    fn nearest_point_from_segment_far_left_test() {
        let segment = Segment{
            a: Point{lat:1.0, lon: 1.0},
            b: Point{lat: 4.0, lon: 4.0},
            layer: None
        };
        let point = Point{lat: -1.0, lon: 2.0};

        assert_eq!(segment.distance_from_point(&point).1, Point{lat:1.0, lon: 1.0});
    }

    #[test]
    fn nearest_point_from_segment_far_right_test() {
        let segment = Segment{
            a: Point{lat:1.0, lon: 1.0},
            b: Point{lat: 4.0, lon: 4.0},
            layer: None
        };
        let point = Point{lat: 6.0, lon: 5.0};

        assert_eq!(segment.distance_from_point(&point).1, Point{lat:4.0, lon: 4.0});
    }

    #[test]
    fn segment_intersection_easy_test() {
        let first_segment = Segment{
            a: Point{lat:1.0, lon: 1.0},
            b: Point{lat: 4.0, lon: 4.0},
            layer: None
        };
        let second_segment = Segment{
            a: Point{lat:1.0, lon: 3.0},
            b: Point{lat: 3.0, lon: 1.0},
            layer: None
        };
        assert_eq!(first_segment.intersection(&second_segment), Some(Point{lat: 2.0, lon: 2.0}));
    }

    #[test]
    fn segment_intersection_end_test() {
        let first_segment = Segment{
            a: Point{lat:1.0, lon: 1.0},
            b: Point{lat: 4.0, lon: 4.0},
            layer: None
        };
        let second_segment = Segment{
            a: Point{lat:-1.0, lon: 3.0},
            b: Point{lat: 1.0, lon: 1.0},
            layer: None
        };
        assert_eq!(first_segment.intersection(&second_segment), Some(first_segment.a));
    }

    #[test]
    fn segment_intersection_none_test() {
        let first_segment = Segment{
            a: Point{lat:1.0, lon: 1.0},
            b: Point{lat: 4.0, lon: 4.0},
            layer: None
        };
        let second_segment = Segment{
            a: Point{lat:11.0, lon: 3.0},
            b: Point{lat: 13.0, lon: 1.0},
            layer: None
        };
        assert_eq!(first_segment.intersection(&second_segment), None);
    }

    #[test]
    fn on_segment_test() {
        let segment = Segment{
            a: Point{lat:1.0, lon: 1.0},
            b: Point{lat: 4.0, lon: 4.0},
            layer: None
        };
        let first_point = Point{
            lat: 2.0,
            lon: 2.0
        };
        let second_point = Point{
            lat: 5.0,
            lon: 5.0
        };
        let third_point = Point{
            lat: 1.0,
            lon: 1.0
        };
        assert_eq!(segment.contains(&first_point), true);
        assert_eq!(segment.contains(&second_point), false);
        assert_eq!(segment.contains(&third_point), true);
        assert_eq!(segment.strictly_contains(&third_point), false);
        assert_eq!(segment.strictly_contains(&second_point), false);
        assert_eq!(segment.strictly_contains(&first_point), true);
    }

    #[test]
    fn partial_length_test() {
        let road = Road{
            name: None,
            segments: vec![
                Segment{
                    a: Point{lat:1.0, lon: 1.0},
                    b: Point{lat: 4.0, lon: 4.0},
                    layer: None
                },
                Segment{
                    a: Point{lat:4.0, lon: 4.0},
                    b: Point{lat: 5.0, lon: 4.0},
                    layer: None
                }
            ],
            forbidden_to_pedestrians: false,
            forbidden_to_bikes: false
        };
        let point = Point{lat: 2.0, lon: 2.0};
        assert!( (road.length_from(&point, Direction::Backward) + road.length_from(&point, Direction::Forward) - road.length()).abs() < 0.1 );
    }

    #[test]
    fn contiguity_test() {
        let first_segment = Segment{
            a: Coord{lat:1.0, lon:1.0},
            b: Coord{lat:1.0, lon: 2.0},
            layer: None
        };
        let second_segment = Segment{
            a: Coord{lat:5.0, lon:1.0},
            b: Coord{lat:1.0, lon: 2.0},
            layer: None
        };
        let third_segment = Segment{
            a: Coord{lat:1.0, lon:1.0},
            b: Coord{lat:6.0, lon: 3.0},
            layer: None
        };
        let fourth_segment = Segment{
            b: Coord{lat:1.0, lon:1.0},
            a: Coord{lat:7.0, lon: -1.0},
            layer: None
        };
        let non_contiguous_segment = Segment{
            b: Coord{lat:-1.0, lon:1.0},
            a: Coord{lat:7.0, lon: -1.0},
            layer: None
        };
        assert!(first_segment.is_contiguous(&second_segment));
        assert!(first_segment.is_contiguous(&third_segment));
        assert!(first_segment.is_contiguous(&fourth_segment));
        assert_eq!(first_segment.is_contiguous(&non_contiguous_segment), false);
        assert_eq!(first_segment.is_contiguous(&first_segment), false);
    }

    #[test]
    fn segment_intersection_layer_test() {
        let first_segment = Segment{
            a: Point{lat:1.0, lon: 1.0},
            b: Point{lat: 4.0, lon: 4.0},
            layer: Some(0)
        };
        let second_segment = Segment{
            a: Point{lat:1.0, lon: 3.0},
            b: Point{lat: 3.0, lon: 1.0},
            layer: Some(-1)
        };
        let third_segment = Segment{
            b: Point{lat:1.0, lon: 1.0},
            a: Point{lat: -4.0, lon: -2.0},
            layer: Some(-1)
        };
        assert_eq!(first_segment.intersection(&second_segment), None);
        assert_eq!(first_segment.intersection(&third_segment), Some(Coord{lat: 1.0, lon: 1.0}));
    }

}
