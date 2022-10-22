use std::io::Read;
use helpers::red::Red;

pub type RideId = usize;
pub type CarId = usize;
pub type TimeT = u32;
pub type CoordT = i32;

#[derive(Copy, Clone, Debug, Default)]
pub struct Position(pub CoordT, pub CoordT);

impl Position {
    pub fn distance(&self, other: &Position) -> TimeT {
        let x = (self.0 - other.0).abs() as TimeT;
        let y = (self.1 - other.1).abs() as TimeT;
        x + y
    }
}

#[derive(Clone, Debug)]
pub struct Ride {
    pub id: RideId,
    pub c_start: Position,
    pub c_finish: Position,
    pub t_start: TimeT,
    pub t_finish: TimeT,
    pub used: bool,
}

impl Ride {
    pub fn length(&self) -> TimeT {
        self.c_start.distance(&self.c_finish)
    }
}

#[derive(Clone, Debug)]
pub struct Car {
    pub id: CarId,
    pub c: Position,
    pub t: TimeT,
    pub rides: Vec<RideId>,
}

impl Car {
    pub fn new(id: CarId) -> Self {
        Self {
            id,
            c: Position::default(),
            t: 0,
            rides: Vec::new(),
        }
    }

    pub fn assign(&mut self, ride: &Ride) {
        let distance_to_start_ride = self.c.distance(&ride.c_start);
        let when_arrive_start = self.t + distance_to_start_ride;
        let when_start = std::cmp::max(when_arrive_start, ride.t_start);
        let when_finish = when_start + ride.length();

        self.t = when_finish;
        self.rides.push(ride.id);
        self.c = ride.c_finish;
    }
}

pub fn read_problem(
    file_path: impl ToString,
) -> (usize, usize, usize, usize, usize, usize, Vec<Ride>) {
    let file = std::fs::File::open(file_path.to_string());
    let iter = std::io::BufReader::new(file.unwrap())
        .bytes()
        .map(Result::unwrap);
    let mut red = Red::new(iter);

    let n_rows = red.read::<usize>();
    let n_cols = red.read::<usize>();
    let n_cars = red.read::<usize>();
    let n_rides = red.read::<usize>();
    let on_time_bonus = red.read::<usize>();
    let n_steps = red.read::<usize>();

    let mut rides = Vec::with_capacity(n_rides);
    for id in 0..n_rides {
        let c_start = Position(red.read::<CoordT>(), red.read::<CoordT>());
        let c_finish = Position(red.read::<CoordT>(), red.read::<CoordT>());
        let t_start = red.read::<TimeT>();
        let t_finish = red.read::<TimeT>();
        let used = false;
        rides.push(Ride {
            id,
            c_start,
            c_finish,
            t_start,
            t_finish,
            used,
        });
    }

    (
        n_rows,
        n_cols,
        n_cars,
        n_rides,
        on_time_bonus,
        n_steps,
        rides,
    )
}

pub struct SolutionCar {
    pub id: CarId,
    pub rides: Vec<RideId>,
}

pub fn read_solution(num_cars: usize, file_path: impl ToString) -> Vec<SolutionCar> {
    let file = std::fs::File::open(file_path.to_string());
    let iter = std::io::BufReader::new(file.unwrap())
        .bytes()
        .map(Result::unwrap);
    let mut red = Red::new(iter);

    let mut cars = Vec::new();
    for id in 0..num_cars {
        let mut rides = Vec::new();
        let n_rides = red.read::<usize>();
        for _ in 0..n_rides {
            rides.push(red.read::<RideId>());
        }
        cars.push(SolutionCar { id, rides });
    }

    cars
}
