/**
 * Not my solution, used the code and approach of Errichto, the winner of the contest
 * https://codeforces.com/blog/entry/58118?#comment-417923
 * https://ideone.com/wzBByv
 */

use lib::*;
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // a: 2
    // c: 1764
    // b: 27501
    // d: 384525
    // e: 549197
    // total 962989
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

fn solve(in_file: &str, out_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut input = read_problem(in_file);
    let mut n_images = input.n_images;
    let mut all_tags = input.all_tags;
    let mut images = input.images;
    
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
