use threadpool::ThreadPool;

use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::time::Instant;

use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::writeln;

use std::io::Read;
use helpers::red::Red;

use bit_set::BitSet;
use rand::Rng;
use serde_json;



fn main() -> Result<(), Box<dyn std::error::Error>> {
    /*
    ./input/b.txt Score: 981457
    ./input/c.txt Score: 730357
    ./input/d.txt Score: 1182
    ./input/e.txt Score: 645626
    ./input/f.txt Score: 661608
    total 3020230
    */
    let files = [
        // ("./input/a.txt", "./output/a.txt"),
        ("./input/b.txt", "./output/b.txt"),
        ("./input/c.txt", "./output/c.txt"),
        ("./input/d.txt", "./output/d.txt"),
        ("./input/e.txt", "./output/e.txt"),
        ("./input/f.txt", "./output/f.txt"),
    ];

    let args = std::env::args().collect::<Vec<_>>();
    match args.get(1).map(String::as_str) {
        Some("check") => {
            let mut scores = Vec::new();
            for (in_file, out_file) in &files {
                let score = check(in_file, out_file)?;
                println!("{} score: {}", in_file, score);
                scores.push(score);
            }
            println!("total {:?}", &scores.iter().sum::<usize>());
        }
        Some("solve") => {
            let timer = Instant::now();
            let pool = ThreadPool::new(5);
            for (in_file, out_file) in files {
                pool.execute(move || {
                    let timer = Instant::now();

                    match &in_file as &str {
                        "./input/a.txt" => solve(in_file, out_file).unwrap(),
                        "./input/b.txt" => solve(in_file, out_file).unwrap(),
                        "./input/c.txt" => solve(in_file, out_file).unwrap(),
                        "./input/d.txt" => solve(in_file, out_file).unwrap(),
                        "./input/e.txt" => solve(in_file, out_file).unwrap(),
                        "./input/f.txt" => solve(in_file, out_file).unwrap(),
                        _ => panic!("default reached"),
                    };

                    println!("{} time: {}", in_file, timer.elapsed().as_millis());
                })
            }
            pool.join();
            println!("total time: {}", timer.elapsed().as_millis());
        }
        _ => panic!("pass either check or solve"),
    }

    Ok(())
}

#[derive(Debug)]
pub struct Street {
    pub id: u32,
    pub from: u32,
    pub to: u32,
    pub len: i32,
    pub name: String,
}

pub struct Car {
    pub id: u32,
    pub trip: Vec<u32>,
    pub dist: i32,
}

pub struct Input {
    pub n_intersections: usize,
    pub bonus: i32,
    pub duration: i32,
    
    pub streets: Vec<Street>,
    pub cars: Vec<Car>,

    pub street_popularity: Vec<i32>,
    pub intersection_popularity: Vec<i32>,

    pub intersection_to_incoming_streets: Vec<Vec<u32>>,
}

/*
./input/a.txt n intersections 4, unused: 0
./input/a.txt n streets 5, unused: 0
./input/a.txt duration 6

./input/b.txt n intersections 7073, unused: 458
./input/b.txt n streets 9102, unused: 741
./input/b.txt duration 5070

./input/c.txt n intersections 10000, unused: 2174
./input/c.txt n streets 35030, unused: 23152
./input/c.txt duration 1640

./input/d.txt n intersections 8000, unused: 0
./input/d.txt n streets 95928, unused: 11917
./input/d.txt duration 8071

./input/e.txt n intersections 500, unused: 0
./input/e.txt n streets 998, unused: 34
./input/e.txt duration 676

./input/f.txt n intersections 1662, unused: 19
./input/f.txt n streets 10000, unused: 4434
./input/f.txt duration 1992
 */
pub fn read_problem(file_path: impl ToString) -> Input {
    let file = std::fs::File::open(file_path.to_string());
    let iter = std::io::BufReader::new(file.unwrap())
        .bytes()
        .map(Result::unwrap);
    let mut red = Red::new(iter);

    let duration = red.read::<i32>();
    let n_intersections = red.read::<usize>();
    let n_streets = red.read::<usize>();
    let n_cars = red.read::<usize>();
    let bonus = red.read::<i32>();

    println!("{} duration {}", file_path.to_string(), duration);

    let mut streets = Vec::with_capacity(n_streets);
    for i in 0..n_streets as u32 {
        let start_intersection = red.read::<u32>();
        let end_intersection = red.read::<u32>();
        let street_name = red.read::<String>();
        let street_length = red.read::<i32>();

        streets.push(Street {
            id: i,
            from: start_intersection,
            to: end_intersection,
            len: street_length,
            name: street_name,
        });
    }

    let mut intersection_to_incoming_streets = vec![Vec::<u32>::new(); n_intersections];
    let mut street_names_to_ids = HashMap::<String, u32>::with_capacity(n_streets);
    for street in &streets {
        intersection_to_incoming_streets[street.to as usize].push(street.id);
        street_names_to_ids.insert(street.name.clone(), street.id);
    }

    let mut cars = Vec::<Car>::with_capacity(n_cars);
    for i in 0..n_cars as u32 {
        let n_trip_streets = red.read::<u32>();
        let mut trip = Vec::<u32>::with_capacity(n_trip_streets as usize);
        let mut dist = 0;
        for _ in 0..n_trip_streets {
            let street_name = red.read::<String>();
            let street_id = street_names_to_ids[&street_name];
            trip.push(street_id);
            
            dist += streets[street_id as usize].len as i32;
        }

        cars.push(Car {
            id: i,
            trip,
            dist,
        });
    }

    let mut street_popularity = vec![0; n_streets];
    let mut intersection_popularity = vec![0; n_intersections];
    for car in &cars {
        // let last_elem = car.trip.last().unwrap();
        // TODO: so do we need to include the last street?
        for &id in &car.trip[..car.trip.len() - 1] {
            let id = id as usize;
            street_popularity[id] += 1;
            intersection_popularity[streets[id].to as usize] += 1;
        }
    }

    let n_unused_streets = street_popularity.iter().filter(|&&p| p == 0).count();
    let n_unused_intersections = intersection_popularity.iter().filter(|&&p| p == 0).count();

    // println!("{} n streets {}, unused: {}", file_path.to_string(), n_streets, n_unused_streets);
    // println!("{} n intersections {}, unused: {}", file_path.to_string(), n_intersections, n_unused_intersections);

    Input {
        n_intersections,
        bonus,
        duration,
        
        streets,
        cars,

        street_popularity,
        intersection_popularity,

        intersection_to_incoming_streets,
    }
}

#[derive(Clone, Eq, PartialEq)]
struct Road {
    pub id: u32,
    pub next_event: i32,
    // pub cars: VecDeque<u32>,
}

use std::cmp::Ordering;


impl Ord for Road {
    fn cmp(&self, other: &Self) -> Ordering {
        self.next_event.cmp(&other.next_event).reverse()
    }
}

impl PartialOrd for Road {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn solve(in_file: &str, out_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let Input { cars, streets, street_popularity, intersection_to_incoming_streets, duration, bonus, n_intersections, .. } = read_problem(in_file);

    let mut res = HashMap::<u32, Vec<(u32, i32)>>::new();

    for i in 0..intersection_to_incoming_streets.len() {
        let mut incoming_streets = Vec::new();
        let mut popularities = Vec::new();

        for &j in &intersection_to_incoming_streets[i] {
            if street_popularity[j as usize] > 0 {
                incoming_streets.push(j);
                popularities.push(street_popularity[j as usize]);
            }
        }

        let min = popularities.iter().filter(|&&p| p > 0).min().copied();
        if let Some(min) = min {
            let len = popularities.len();
            for k in 0..len {
                popularities[k] /= min;
                // popularities[k] *= 2;
            }

            for (&id, &popularity) in incoming_streets.iter().zip(&popularities) {
                res.entry(i as u32).or_default().push((id, popularity));
            }
        }
    }


    let get_curr_green = |curr_t: i32, intersection: u32| {
        let cycle_len = res[&intersection].iter().map(|&(_street, seconds)| seconds).sum::<i32>();
        let t = curr_t % cycle_len;

        let mut sum_t = 0;
        for &(street, seconds) in res[&intersection].iter() {
            sum_t += seconds;
            if t < sum_t {
                return (street, curr_t, curr_t - t + sum_t);
            }
        }

        panic!();
    };

    let get_next_green_for_street = |curr_t: i32, street: u32| {
        let intersection = streets[street as usize].to;
        println!("intersection {} street {}", intersection, street);
        let cycle_len = res[&intersection].iter().map(|&(_street, seconds)| seconds).sum::<i32>();
        let t = curr_t % cycle_len;

        let mut sum_t = 0;
        for &(edge, seconds) in res[&intersection].iter() {
            if edge == street {
                let mut start = curr_t - t + sum_t;
                let mut end = curr_t - t + sum_t + seconds;

                if end <= t {
                    start += cycle_len;
                    end += cycle_len;
                }

                start = std::cmp::max(start, curr_t);

                return (start, end);
            }
            
            sum_t += seconds;
        }

        panic!();
    };

    let print_intersection = |intersection: u32| {
        println!("Intersection {} schedule:", intersection);
        if let Some(schedule) = res.get(&intersection) {
            for &(street, duration) in schedule {
                println!("street {}, duration {}", streets[street as usize].name, duration);
            }
        }
    };

    let mut trips_left = cars.iter().map(|c| c.trip.iter().copied().collect::<VecDeque<_>>()).collect::<Vec<_>>();
    let mut cars_at_streets = vec![VecDeque::<(i32, u32)>::new(); streets.len()];
    for car in &cars {
        let time_end_street = 0;
        cars_at_streets[car.trip[0] as usize].push_back((time_end_street, car.id));
    }

    // for i in 0..n_intersections as u32 {
    //     print_intersection(i);
    // }

    let mut score = 0;
    for t in 0..duration {
        for (&intersection, schedule) in res.iter() {
            // if schedule.is_empty() { continue; }
            if schedule.is_empty() { panic!(); }

            let (street, ..) = get_curr_green(t, intersection);
            // println!("{} green at {}", t, &streets[street as usize].name);
            if let Some(&(time_end_street, car)) = cars_at_streets[street as usize].front() {
                if t >= time_end_street {
                    // remove the car from the street
                    cars_at_streets[street as usize].pop_front();

                    // remove street from the car trip
                    let trip = &mut trips_left[car as usize];
                    assert!(trip.pop_front() == Some(street));

                    // check the next street in the trip
                    match trip.front() {
                        Some(&next_street) => {
                            // add to the next street
                            let new_time_end_street = t + streets[next_street as usize].len;
                            cars_at_streets[next_street as usize].push_back((new_time_end_street, car));
                            // println!("{} moving car {} from {} to {}", t, car, &streets[street as usize].name, &streets[next_street as usize].name);
                        },
                        None => {
                            score += duration - t + bonus;
                            // println!("{} car {} ended ride on street {}", t, car, &streets[street as usize].name);
                        },
                    }
                }
            }
        }
        
        // println!();
    }

    println!("{} Score: {}", in_file, score);

    Ok(())
}

fn draft_solve(in_file: &str, out_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let Input { cars, streets, street_popularity, intersection_to_incoming_streets, duration, bonus, .. } = read_problem(in_file);

    let mut res = HashMap::<u32, Vec<(u32, i32)>>::new();

    for i in 0..intersection_to_incoming_streets.len() {
        let mut incoming_streets = Vec::new();
        let mut popularities = Vec::new();

        for &j in &intersection_to_incoming_streets[i] {
            if street_popularity[j as usize] > 0 {
                incoming_streets.push(j);
                popularities.push(street_popularity[j as usize]);
            }
        }

        let min = popularities.iter().filter(|&&p| p > 0).min().copied();
        if let Some(min) = min {
            let len = popularities.len();
            for k in 0..len {
                popularities[k] /= min;
                popularities[k] *= 2;
            }

            for (&id, &popularity) in incoming_streets.iter().zip(&popularities) {
                res.entry(i as u32).or_default().push((id, popularity));
            }
        }
    }

    let get_curr_green = |curr_t: i32, intersection: u32| {
        let cycle_len = res[&intersection].iter().map(|&(_street, seconds)| seconds).sum::<i32>();
        let t = curr_t % cycle_len;

        let mut sum_t = 0;
        for &(street, seconds) in res[&intersection].iter() {
            sum_t += seconds;
            if t < sum_t {
                return (street, curr_t, curr_t - t + sum_t);
            }
        }

        panic!();
    };

    let get_next_green_for_street = |curr_t: i32, street: u32| {
        let intersection = streets[street as usize].to;
        println!("intersection {} street {}", intersection, street);
        let cycle_len = res[&intersection].iter().map(|&(_street, seconds)| seconds).sum::<i32>();
        let t = curr_t % cycle_len;

        let mut sum_t = 0;
        for &(edge, seconds) in res[&intersection].iter() {
            if edge == street {
                let mut start = curr_t - t + sum_t;
                let mut end = curr_t - t + sum_t + seconds;

                if end <= t {
                    start += cycle_len;
                    end += cycle_len;
                }

                start = std::cmp::max(start, curr_t);

                return (start, end);
            }
            
            sum_t += seconds;
        }

        panic!();
    };

    let print_intersection = |intersection: u32| {
        println!("Intersection {} schedule:", intersection);
        for &(street, duration) in &res[&intersection] {
            println!("street {}, duration {}", streets[street as usize].name, duration);
        }
    };
    
    
    // let mut pq = BinaryHeap::with_capacity(streets.len());

    let mut trips_left = cars.iter().map(|c| c.trip.iter().copied().collect::<VecDeque<_>>()).collect::<Vec<_>>();
    let mut cars_at_streets = vec![VecDeque::<(i32, u32)>::new(); streets.len()];
    for car in &cars {
        let time_end_street = 0;
        cars_at_streets[car.trip[0] as usize].push_back((time_end_street, car.id));
    }
    let mut pq = (0..streets.len())
        .map(|i| Road { id: i as u32, next_event: 0})
        .collect::<BinaryHeap<Road>>();

    let mut score = 0;
    let mut t = 0;
    while !pq.is_empty() && t <= duration {
        let mut road = pq.pop().unwrap();
        let (start, end) = get_next_green_for_street(t, road.id);
        if t != start {
            road.next_event = start;
            pq.push(road);
            continue;
        }

        t = road.next_event;

        for tt in start..end {
            if cars_at_streets[road.id as usize].is_empty() {
                break;
            }
            let (time_at_street_end, car_id) = cars_at_streets[road.id as usize][0];
            if time_at_street_end <= tt {
                let trip = &mut trips_left[car_id as usize];
                assert!(trip[0] == road.id);

                // this car is removed from this street
                cars_at_streets[road.id as usize].pop_front();

                trip.pop_front();
                if !trip.is_empty() {
                    let next_street = trip[0];
                    let when_end_street = tt + streets[road.id as usize].len;
                    cars_at_streets[next_street as usize].push_back((when_end_street, car_id));
                    println!("Sending car {} to {} at {}", car_id, next_street, when_end_street);
                } else {
                    let when_end_street = tt + streets[road.id as usize].len;
                    if when_end_street <= duration {
                        score += duration - when_end_street + bonus;
                    }
                    println!("Car {} finished at {}", car_id, when_end_street);
                }
            }
        }

        road.next_event = match cars_at_streets[road.id as usize].front() {
            Some(&(time_end_street, car_id)) => time_end_street,
            None => i32::MAX,
        };

        pq.push(road);
    }

    println!("Score: {}", score);

    Ok(())
}

fn check(in_file: &str, out_file: &str) -> Result<usize, Box<dyn std::error::Error>> {
    Ok(0)
}
