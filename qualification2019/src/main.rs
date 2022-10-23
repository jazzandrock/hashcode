/**
 * Not my solution, used the code and approach of Errichto, the winner of the contest
 * https://codeforces.com/blog/entry/58118?#comment-417923
 * https://ideone.com/wzBByv
 */

use lib::*;
use rand::seq::SliceRandom;
use threadpool::ThreadPool;

use std::collections::HashMap;
use std::hash::Hash;
use std::time::Instant;

use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::write;
use std::writeln;

use std::io::Read;
use helpers::red::Red;

use bit_set::BitSet;
use rand::Rng; // 0.8.5


fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // a num H: 2, V: 2
    // b num H: 80000, V: 0
    // c num H: 500, V: 500
    // d num H: 30000, V: 60000
    // e num H: 0, V: 80000

    // a: 2
    // c: 1764
    // b: 27501
    // d: 384525
    // e: 549197
    // total 962989
    let files = [
        // ("./input/a.txt", "./output/a.txt"),
        ("./input/b.txt", "./output/b.txt"),
        // ("./input/b_cut.txt", "./output/b_cut.txt"),
        // ("./input/a_all_horiz.txt", "./output/a_all_horiz.txt"),
        // ("./input/c.txt", "./output/c.txt"),
        // ("./input/d.txt", "./output/d.txt"),
        // ("./input/e.txt", "./output/e.txt"),
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
                    solve_annealing_all_horizontal(in_file, out_file).unwrap();
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

#[derive(Debug, Clone)]
pub struct Img {
    id: u32,
    used: bool,
    vert: bool,
    tags: Vec<u32>,
}

#[derive(Debug)]
pub struct Input {
    n_images: u32,
    images: Vec<Img>,
    all_tags: HashMap<String, u32>,
}

pub fn read_problem(file_path: impl ToString) -> Input {
    let file = std::fs::File::open(file_path.to_string());
    let iter = std::io::BufReader::new(file.unwrap())
        .bytes()
        .map(Result::unwrap);
    let mut red = Red::new(iter);

    let mut all_tags = HashMap::<String, u32>::new();
    let mut get_tag_id = |s: String| -> u32 {
        let len = all_tags.len() as u32;
        *all_tags.entry(s).or_insert(len)
    };

    let n_images = red.read::<u32>();
    let mut images = Vec::with_capacity(n_images as usize);
    for id in 0..n_images {
        let vert = 'V' == red.read::<char>();
        let n_tags = red.read::<usize>();
        let mut tags = Vec::with_capacity(n_tags);
        for _ in 0..n_tags {
            let tag = red.read::<String>();
            let tag_id = get_tag_id(tag);
            tags.push(tag_id);
        }
        tags.sort();
        images.push(Img {
            id,
            used: false,
            vert,
            tags,
        });
    }

    Input {
        n_images,
        images,
        all_tags,
    }
}

fn intersection_size(a: &[u32], b: &[u32]) -> usize {
    let mut r = 0;
    let mut i = 0;
    let mut j = 0;
    while i < a.len() && j < b.len() {
        if a[i] < b[j] {
            i += 1;
        } else if a[i] == b[j] {
            i += 1;
            j += 1;
            r += 1;
        } else {
            j += 1;
        }
    }
    r
}

fn get_score(a: &[u32], b: &[u32]) -> usize {
    let common = intersection_size(a, b);
    let not_common = std::cmp::min(a.len() - common, b.len() - common);
    std::cmp::min(common, not_common)
}

fn get_score_bits(a: &BitSet, b: &BitSet) -> usize {
    let common = a.intersection(b).count();
    let not_common = std::cmp::min(a.len() - common, b.len() - common);
    std::cmp::min(common, not_common)
}

fn outer_join(a: &[u32], b: &[u32]) -> Vec<u32> {
    let mut result = Vec::with_capacity(a.len() + b.len());

    let mut i = 0;
    let mut j = 0;
    while i < a.len() || j < b.len() {
        if i == a.len() {
            result.push(b[j]);
            j += 1;
        } else if j == b.len() {
            result.push(a[i]);
            i += 1;
        } else if a[i] < b[j] {
            result.push(a[i]);
            i += 1;
        } else if a[i] == b[j] {
            result.push(a[i]);
            i += 1;
            j += 1;
        } else {
            result.push(b[j]);
            j += 1;
        }
    } 

    result
}

fn solve_annealing_all_horizontal(in_file: &str, out_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut input = read_problem(in_file);
    let mut n_images = input.n_images as usize;
    let mut all_tags = input.all_tags;
    let mut images = input.images;

    let mut rng = rand::thread_rng();

    // first, generate random permutation of our images — that would be our initial state
    let mut res = (0..n_images).collect::<Vec<_>>();
    res.shuffle(&mut rng);

    // now, compute the score for it
    let mut total_score = 0i64;
    for i in 1..res.len() {
        let prev_slide = &images[res[i - 1]].tags;
        let curr_slide = &images[res[i]].tags;
        total_score += get_score(prev_slide, curr_slide) as i64;
    }

    let n_iterations_per_t = 50;
    let mut temperature = 10.0;
    let init_temperature = temperature;
    let cooldown = 0.999999;

    let time_start = Instant::now();
    let mut last_time_printed = Instant::now();
    // last_time_printed.elapsed().as_millis()

    let mut avg_delta = 0;
    let mut avg_prob = 0.0;
    let mut n_delta = 0;

    let mut max_score = total_score;
    let mut max_score_temp = temperature;

    let mut total_iterations = 0;
    while temperature > 0.0001 {
        let (delta, id1, id2) = {
            let mut best_score_swap = (i32::MIN, 0, 0);
            for _ in 0..n_iterations_per_t {
                // now choose two images that we swap
                let mut id1 = rng.gen_range(0, n_images);
                let mut id2 = rng.gen_range(0, n_images);
                
                // totally unnecessary
                if id1 > id2 {
                    (id1, id2) = (id2, id1);
                }
        
                let mut delta_score = 0i32;
                for (id1, id2) in [(id1, id2), (id2, id1)] {
                    // when we remove the image from slideshow, we lose scores for 1 or 2 transitions
                    if id1 > 0 {
                        let prev_slide = &images[res[id1 - 1]].tags;
                        let curr_slide = &images[res[id1]].tags;
                        delta_score -= get_score(prev_slide, curr_slide) as i32;
                
                        let new_curr_slide = &images[res[id2]].tags;
                        delta_score += get_score(prev_slide, new_curr_slide) as i32;
                    }
                    if id1 < n_images - 1 {
                        let curr_slide = &images[res[id1]].tags;
                        let next_slide = &images[res[id1 + 1]].tags;
                        delta_score -= get_score(curr_slide, next_slide) as i32;
                
                        let new_curr_slide = &images[res[id2]].tags;
                        delta_score += get_score(new_curr_slide, next_slide) as i32;
                    }
                }
        
                best_score_swap = std::cmp::max(best_score_swap, (delta_score, id1, id2));
            }
    
            best_score_swap
        };

        total_iterations += 1;

        avg_delta += delta;
        n_delta += 1;

        let mut improper_probs = Vec::new();
        let mut prob = 1.0 / (1.0 + (-delta as f64 / temperature).exp());
        if !prob.is_finite() || prob > 1.0 {
            improper_probs.push(prob);
            prob = 1.0;
        }
        avg_prob += prob;
        let take = rng.gen_bool(prob);
        if take {
            res.swap(id1, id2);
            total_score += delta as i64;
        }

        if total_score > max_score {
            max_score = total_score;
            max_score_temp = temperature;
        }

        if last_time_printed.elapsed().as_millis() > 100 {
            print!("\x1B[2J\x1B[1;1H");
            println!("Time running: {}, n iterations: {}", time_start.elapsed().as_secs(), total_iterations);
            println!("Avg delta: {}", avg_delta as f64 / n_delta as f64);
            println!("Avg prob: {}", avg_prob as f64 / n_delta as f64);
            println!("Init T: {}, cooldown: {}, n_iterations: {}", init_temperature, cooldown, n_iterations_per_t);
            println!("Score: {}", total_score);
            println!("Delta: {}", delta);
            println!("Temperature: {}", temperature);
            println!("prob: {}", prob);
            println!("Max score {} at temp {}", max_score, max_score_temp);
            println!("Improper probs: {:?}", &improper_probs);
            last_time_printed = Instant::now();
            avg_delta = 0;
            n_delta = 0;
            avg_prob = 0.0;
        }

        temperature *= cooldown;
    }

    // println!("score: {}, delta: {}, res: {:?}", score, delta_score, &res);

    Ok(())
}

fn solve(in_file: &str, out_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut input = read_problem(in_file);
    let mut n_images = input.n_images;
    let mut all_tags = input.all_tags;
    let mut images = input.images;

    const NON_EXISTENT_IMG: u32 = u32::MAX;

    let mut solution = Vec::<(u32, u32)>::new();
    // later make this a random choice
    {
        let mut rng = rand::thread_rng();
        let n_horiz_images = images.iter().filter(|i| !i.vert).count();
        let n_vert_img = images.len() - n_horiz_images;
        if n_horiz_images > 0 {
            let id = rng.gen_range(0, n_horiz_images);
            let id = images.iter().filter(|i| !i.vert).skip(id).last().map(|i| i.id).unwrap();
            solution.push((id, NON_EXISTENT_IMG));
            images[id as usize].used = true;
        } else {
            let id1 = rng.gen_range(0, n_vert_img);
            let mut id2 = id1;
            while id2 == id1 {
                id2 = rng.gen_range(0, n_vert_img);
            }

            solution.push((id1 as u32, id2 as u32));
            images[id1 as usize].used = true;
            images[id2 as usize].used = true;
        }
    }

    let mut total_score = 0;
    loop {
        let prev_slide = solution.last().unwrap();
        let prev_slide_tags = if prev_slide.1 == NON_EXISTENT_IMG {
            // this is a horizontal img
            images[prev_slide.0 as usize].tags.clone()
        } else {
            // it's two vertical images
            outer_join(
                &images[prev_slide.0 as usize].tags, 
                &images[prev_slide.1 as usize].tags
            )
        };


        let mut best_image = (0, NON_EXISTENT_IMG);
        for j in 0..n_images as usize {
            // let's treat all images as horizontal
            if images[j as usize].used { continue }

            let score = get_score(&prev_slide_tags, &images[j].tags);
            best_image = std::cmp::max(best_image, (score, j as u32));
        }
        let (score, j) = best_image;
        if j == NON_EXISTENT_IMG {
            // we couldn't find a single image even, horizontal or vertical
            break;
        }

        if !images[j as usize].vert {
            // this is a horizontal image, add as is
            solution.push((j, NON_EXISTENT_IMG));
            images[j as usize].used = true;

            total_score += score;
        } else {
            // we found a 'decent' vertical img, now we search for another 'decent' companion for it
            let first_best_vertical_img_id = j;
            
            let mut best_image = (0, NON_EXISTENT_IMG);
            for j in 0..n_images as usize {
                if images[j as usize].used || j == first_best_vertical_img_id as usize { continue }
                let curr_slide_tags = outer_join(
                    &images[first_best_vertical_img_id as usize].tags,
                    &images[j].tags
                );

                let score = get_score(&prev_slide_tags, &curr_slide_tags);
                best_image = std::cmp::max(best_image, (score, j as u32));
            }

            let (score, j) = best_image;
            if j == NON_EXISTENT_IMG {
                break;
            }

            solution.push((first_best_vertical_img_id, j));
            images[first_best_vertical_img_id as usize].used = true;
            images[j as usize].used = true;

            total_score += score;
        }
    }
    println!("{} total score: {}", in_file, total_score);

    Ok(())
}

fn solve_bits(in_file: &str, out_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let Input { n_images, all_tags, mut images } = read_problem(in_file);
    
    let use_bitsets = all_tags.len() <= 500;
    let mut bit_tags = Vec::<BitSet>::new();
    if use_bitsets {
        bit_tags.reserve_exact(n_images as usize);
        for img in &images {
            let mut bits = BitSet::with_capacity(all_tags.len());
            for &tag in &img.tags {
                bits.insert(tag as usize);
            }
            bit_tags.push(bits);
        }
    }

    if !use_bitsets {
        return Ok(());
    }

    const NON_EXISTENT_IMG: u32 = u32::MAX;

    let mut solution = Vec::<(u32, u32)>::new();
    // later make this a random choice
    match images.iter().find(|i| !i.vert).map(|i| i.id) {
        Some(id) => {
            solution.push((id, NON_EXISTENT_IMG));
            images[id as usize].used = true;
        },
        None => {
            solution.push((0, 1));
            images[0].used = true;
            images[1].used = true;
        },
    };

    let mut total_score = 0;
    loop {
        let prev_slide = solution.last().unwrap();
        let prev_slide_tags = if prev_slide.1 == NON_EXISTENT_IMG {
            // this is a horizontal img
            bit_tags[prev_slide.0 as usize].clone()
        } else {
            // it's two vertical images
            let mut bits = bit_tags[prev_slide.0 as usize].clone();
            bits.union_with(&bit_tags[prev_slide.1 as usize]);
            bits
        };


        let mut best_image = (0, NON_EXISTENT_IMG);
        for j in 0..n_images as usize {
            // let's treat all images as horizontal
            if images[j as usize].used { continue }

            let score = get_score_bits(&prev_slide_tags, &bit_tags[j]);
            best_image = std::cmp::max(best_image, (score, j as u32));
        }
        let (score, j) = best_image;
        if j == NON_EXISTENT_IMG {
            // we couldn't find a single image even, horizontal or vertical
            break;
        }

        if !images[j as usize].vert {
            // this is a horizontal image, add as is
            solution.push((j, NON_EXISTENT_IMG));
            images[j as usize].used = true;

            total_score += score;
        } else {
            // we found a 'decent' vertical img, now we search for another 'decent' companion for it
            let first_best_vertical_img_id = j;
            
            let mut best_image = (0, NON_EXISTENT_IMG);
            for j in 0..n_images as usize {
                if images[j as usize].used || j == first_best_vertical_img_id as usize { continue }
                let mut curr_slide_tags = bit_tags[first_best_vertical_img_id as usize].clone();
                curr_slide_tags.union_with(&bit_tags[j]);

                let score = get_score_bits(&prev_slide_tags, &curr_slide_tags);
                best_image = std::cmp::max(best_image, (score, j as u32));
            }

            let (score, j) = best_image;
            if j == NON_EXISTENT_IMG {
                break;
            }

            solution.push((first_best_vertical_img_id, j));
            images[first_best_vertical_img_id as usize].used = true;
            images[j as usize].used = true;

            total_score += score;
        }
    }
    println!("{} total score: {}", in_file, total_score);

    Ok(())
}

fn check(in_file: &str, out_file: &str) -> Result<usize, Box<dyn std::error::Error>> {
    Ok(0)
}
