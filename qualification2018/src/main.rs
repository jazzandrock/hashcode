/**
 * Not my solution, used the code and approach of Errichto, the winner of the contest
 * https://codeforces.com/blog/entry/58118?#comment-417923
 * https://ideone.com/wzBByv
 */

use lib::*;
use threadpool::ThreadPool;

use std::time::Instant;

use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::write;
use std::writeln;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let files = [
        ("./input/a.txt", "./output/a.txt"),
        ("./input/b.txt", "./output/b.txt"),
        ("./input/c.txt", "./output/c.txt"),
        ("./input/d.txt", "./output/d.txt"),
        ("./input/e.txt", "./output/e.txt"),
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
                    solve(in_file, out_file).unwrap();
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

fn solve(in_file: &str, out_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (_n_rows, _n_cols, n_cars, n_rides, _bonus, n_steps, mut rides) = read_problem(in_file);

    let mut cars = (0..n_cars).map(Car::new).collect::<Vec<Car>>();

    rides.sort_by_key(|r| r.t_start);

    let mut far = vec![TimeT::MAX; n_rides];
    for i in 0..n_rides {
        for j in 0..n_rides {
            if i != j {
                far[i] = std::cmp::min(far[i], rides[i].c_finish.distance(&rides[j].c_start));
            }
        }
    }

    loop {
        let mut anything = false;
        for i in 0..n_cars {
            let mut score_ride = (u32::MAX, 0);
            for j in 0..n_rides {
                let distance_to_start_ride = cars[i].c.distance(&rides[j].c_start);
                let when_arrive_start = cars[i].t + distance_to_start_ride;
                let when_start = std::cmp::max(when_arrive_start, rides[j].t_start);
                let time_waiting_for_start = when_start - when_arrive_start;
                let when_finish = when_start + rides[j].length();

                let ride_possible = !rides[j].used && when_finish <= rides[j].t_finish;
                let mut wasted = distance_to_start_ride + time_waiting_for_start;

                if when_finish <= n_steps as u32 / 100 * 98 {
                    wasted += far[j] / 15;
                }

                if ride_possible {
                    score_ride = std::cmp::min(score_ride, (wasted, j));
                }
            }

            let (wasted, j) = score_ride;
            if wasted == u32::MAX {
                continue;
            }

            cars[i].assign(&rides[j]);
            rides[j].used = true;
            anything = true;
        }
        
        if !anything {
            break;
        }
    }

    let mut out_file = BufWriter::new(File::create(out_file)?);
    for v in cars.iter() {
        write!(&mut out_file, "{} ", v.rides.len())?;
        for ride_id in &v.rides {
            write!(&mut out_file, "{} ", ride_id)?;
        }
        writeln!(&mut out_file)?;
    }

    Ok(())
}

fn check(in_file: &str, out_file: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let (_n_rows, _n_cols, n_cars, _n_rides, bonus, _n_steps, mut rides) = read_problem(in_file);
    let cars = read_solution(n_cars, out_file);

    assert!(cars.len() == n_cars);

    let mut score = 0usize;

    for car in cars {
        let mut t: TimeT = 0;
        let mut pos = Position::default();
        for r in car.rides {
            let ride = &mut rides[r];
            assert!(!ride.used);
            
            let to = pos.distance(&ride.c_start);
            let len = ride.length();
            assert!(t + to + len <= ride.t_finish);
            
            if t + to <= ride.t_start {
                score += bonus;
            }
            score += len as usize;

            t = std::cmp::max(ride.t_start + len, t + to + len);
            
            pos = ride.c_finish.clone();
            ride.used = true;
        }
    }

    Ok(score)
}
