use lib::*;
use threadpool::ThreadPool;

use std::collections::HashSet;

use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::sync::RwLock;

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
            for (in_file, out_file) in &files {
                let timer = Instant::now();
                solve(in_file, out_file)?;
                println!("{} time: {}", in_file, timer.elapsed().as_millis());
            }
            println!("total time: {}", timer.elapsed().as_millis());
        }
        _ => panic!("pass either check or solve"),
    }

    Ok(())
}

fn solve(in_file: &str, out_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (_n_books, n_libs, n_days, book_scores, libs) = read_problem(in_file);

    let mut res = Vec::new();

    let n_workers = 12;
    let pool = ThreadPool::new(n_workers);

    let mut curr_day = 0;
    let book_scores = Arc::new(RwLock::new(book_scores));
    let libs = Arc::new(RwLock::new(libs));
    let (tx, rx) = channel();
    loop {
        let counter = Arc::new(AtomicUsize::new(0));
        for _ in 0..n_workers {
            let tx = tx.clone();
            let book_scores = book_scores.clone();
            let libs = libs.clone();
            let counter = counter.clone();
            let mut result = Vec::with_capacity(n_libs);
            pool.execute(move || {
                let book_scores = book_scores.read().unwrap();
                let libs = libs.read().unwrap();
                loop {
                    let lib_id = counter.fetch_add(1, Relaxed);
                    if lib_id >= n_libs {
                        break;
                    }

                    let lib = &libs[lib_id];
                    if lib.is_used() {
                        continue;
                    }

                    let books = lib.get_best_books(curr_day, n_days, &book_scores);
                    let mut score = books
                        .iter()
                        .map(|b| book_scores[*b] as LibraryScore)
                        .sum::<LibraryScore>();

                    // oh god, really?
                    score /= lib.n_days as LibraryScore;

                    result.push((lib.id, books, score));
                }
                tx.send(result)
                    .expect("channel will be there waiting for the pool");
            });
        }

        match rx.iter().take(n_workers).flatten().max_by_key(|t| t.2) {
            Some((id, books, _score)) => {
                let mut libs = libs.write().unwrap();
                curr_day += libs[id].n_days;
                if curr_day >= n_days {
                    break;
                }

                let mut book_scores = book_scores.write().unwrap();
                for book_id in &books {
                    book_scores[*book_id] = 0;
                }
                res.push((id, books));
                libs[id].mark_as_used();
            }
            None => break,
        }
    }

    let mut out_file = BufWriter::new(File::create(out_file)?);
    writeln!(&mut out_file, "{}", res.len())?;
    for (id, books) in res {
        writeln!(&mut out_file, "{} {}", id, books.len())?;
        for book_id in books {
            write!(&mut out_file, "{} ", book_id)?;
        }
        writeln!(&mut out_file)?;
    }

    Ok(())
}

fn check(in_file: &str, out_file: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let (_n_books, n_libs, n_days, book_scores, libs) = read_problem(in_file);
    let solution_libs = read_solution(out_file);

    assert!(solution_libs.len() <= n_libs);
    assert!(
        solution_libs
            .iter()
            .map(|l| l.id)
            .collect::<HashSet<_>>()
            .len()
            == solution_libs.len()
    );

    let mut score = 0usize;

    let mut all_books_used = HashSet::new();
    let mut days_used = 0;
    for lib in solution_libs.into_iter() {
        days_used += libs[lib.id].n_days;
        let books_used = libs[lib.id].n_ship_daily * (n_days - days_used);
        let books_used = std::cmp::min(lib.books.len(), books_used);
        let books = &lib.books[0..books_used];
        let lib_books = libs[lib.id].books.iter().collect::<HashSet<_>>();
        for book in books {
            assert!(lib_books.contains(book));
            if !all_books_used.contains(book) {
                all_books_used.insert(*book);
                score += book_scores[*book] as usize;
            }
        }
    }

    Ok(score)
}
