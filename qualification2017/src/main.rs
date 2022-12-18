use std::str::FromStr;
use threadpool::ThreadPool;

use std::collections::HashMap;
use std::io::BufRead;
use std::time::Instant;

use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::writeln;

use std::io::Read;
use helpers::red::Red;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    /*
        a n_cache 10
        a n_endpoints 10
        a n_req_desc 100
        a n_videos 100

        b n_cache 500
        b n_endpoints 1000
        b n_req_desc 200000
        b n_videos 10000

        c n_cache 100
        c n_endpoints 100
        c n_req_desc 100000
        c n_videos 10000

        d n_cache 100
        d n_endpoints 100
        d n_req_desc 100000
        d n_videos 10000
    */
    /*
        a Final score: 507906
        b Final score: 1021680
        c Final score: 499970
        d Final score: 608303
        total 2637859
    */
    let files = [
        ("./input/a.txt", "./output/a.txt"),
        ("./input/b.txt", "./output/b.txt"),
        ("./input/c.txt", "./output/c.txt"),
        ("./input/d.txt", "./output/d.txt"),
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
            println!("total {:?}", &scores.iter().sum::<i64>());
        }
        Some("solve") => {
            let timer = Instant::now();
            let pool = ThreadPool::new(files.len());
            for (in_file, out_file) in files {
                let closure = move || {
                    let timer = Instant::now();
                    solve(in_file, out_file).unwrap();
                    println!("{} time: {}", in_file, timer.elapsed().as_millis());
                };
                const MULTI_THREAD: bool = false;
                if MULTI_THREAD {
                    pool.execute(closure);
                } else {
                    closure();
                }
            }
            pool.join();
            println!("total time: {}", timer.elapsed().as_millis());
        }
        _ => panic!("pass either check or solve"),
    }

    Ok(())
}

struct Req {
    vid_id: i32,
    endpoint_id: i32, 
    n_requests: i32,
}

struct Input {
    n_videos: i32,
    _n_endpoints: i32,
    n_req_desc: i32,
    n_servers: i32,
    server_capacity: i32,

    video_sizes: Vec<i32>,
    server_endpoints: Vec<Vec<i32>>,
    endpoint_servers: Vec<Vec<i32>>,

    endp_lats: Vec<HashMap<i32, i32>>,

    reqs: Vec<Req>,

    endp_reqs: Vec<Vec<i32>>,
}

const DC_ID: i32 = -1; // ID of datacenter

fn read_problem(in_file: &str) -> Input {
    let file = std::fs::File::open(in_file.to_string());
    let iter = std::io::BufReader::new(file.unwrap())
        .bytes()
        .map(Result::unwrap);
    let mut red = Red::new(iter);

    let n_videos = red.read::<i32>();
    let n_endpoints = red.read::<i32>();
    let n_req_desc = red.read::<i32>();
    let n_servers = red.read::<i32>();  
    let server_capacity = red.read::<i32>();  

    let video_sizes = red.read_vec::<i32>(n_videos as usize);

    // which endpoints are connected to the server
    let mut server_endpoints = vec![vec![]; n_servers as usize];
    // which servers are connected to the endpoint
    let mut endpoint_servers = vec![vec![]; n_endpoints as usize];

    // endpoint id -> server id -> latency
    let mut endp_lats = vec![HashMap::<i32, i32>::new(); n_endpoints as usize];

    for i in 0..n_endpoints {
        let latency_datacenter = red.read::<i32>();
        let n_server_connected = red.read::<i32>();

        endp_lats[i as usize].insert(DC_ID, latency_datacenter);

        for _ in 0..n_server_connected {
            let server_id = red.read::<i32>();
            let latency_server = red.read::<i32>();

            server_endpoints[server_id as usize].push(i);
            endpoint_servers[i as usize].push(server_id);

            endp_lats[i as usize].insert(server_id, latency_server);
        }
    }

    let mut reqs: Vec<Req> = Vec::with_capacity(n_req_desc as usize);
    let mut endp_reqs = vec![vec![]; n_endpoints as usize];
    for i in 0..n_req_desc {
        let vid_id = red.read::<i32>();
        let endpoint_id = red.read::<i32>();
        let n_requests = red.read::<i32>();
        reqs.push(Req {
            vid_id, 
            endpoint_id,
            n_requests
        });

        endp_reqs[endpoint_id as usize].push(i);
    }

    Input {
        n_videos,
        _n_endpoints: n_endpoints,
        n_req_desc,
        n_servers,
        server_capacity,
    
        video_sizes,
        server_endpoints,
        endpoint_servers,

        endp_lats,

        reqs,
    
        endp_reqs,
    }
}

fn solve(in_file: &str, out_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    /*
        new map best_request_latency: request -> best latency for request (fill out with latency to DC)

        new map server_requests: server -> list of requests
        or better server -> endpoints
        endpoint -> list of requests

        fn consider (server, video) -> how much latency we save
            for each endpoint connected to the server
                for each request for that endpoint
                    that is for the video
                        sum(best_request_latency[request] - new latency * n_requests)

        for each server
            for each video that has a connection to that server
                consider(server, video)
        when found the best score, add this video to the server
    */

    let Input {
        n_videos,
        n_req_desc,
        n_servers,
        server_capacity,
    
        video_sizes,
        server_endpoints,
        endpoint_servers,

        endp_lats,

        reqs,
    
        endp_reqs,
        ..
    } = read_problem(in_file);


    // this data structure takes 800 MB
    // server -> video -> list of requests
    let mut serv_vid_reqs: Vec<HashMap<i32, Vec<i32>>> = vec![HashMap::new(); n_servers as usize];
    for (i, req) in reqs.iter().enumerate() {
        for &serv in &endpoint_servers[req.endpoint_id as usize] {
            serv_vid_reqs[serv as usize].entry(req.vid_id).or_default().push(i as i32);
        }
    }

    let mut best_request_latency = vec![-1; n_req_desc as usize];
    for i in 0..n_req_desc as usize {
        let endpoint_id = reqs[i].endpoint_id;
        best_request_latency[i] = endp_lats[endpoint_id as usize][&DC_ID];
    }
    let mut server_capacities = vec![server_capacity; n_servers as usize];
    let mut serv_vid_score_table = vec![ vec![ 0i64; n_videos as usize ]; n_servers as usize ];
    for server in 0..n_servers {
        for video in 0..n_videos {
            if server_capacities[server as usize] < video_sizes[video as usize] { continue; }

            let the_reqs = serv_vid_reqs[server as usize].get(&video);
            if the_reqs.is_none() { continue; }
            let the_reqs = the_reqs.unwrap();

            let mut server_score = 0i64;
            for &req_id in the_reqs {
                let endp_id = reqs[req_id as usize].endpoint_id;
                if reqs[req_id as usize].vid_id != video { continue; }

                let old_latency = best_request_latency[req_id as usize] as i64;
                let new_latency = endp_lats[endp_id as usize][&server] as i64;
                
                if old_latency <= new_latency { continue; }

                let n_reqs = reqs[req_id as usize].n_requests as i64;
                let score = (old_latency - new_latency) * n_reqs;
                server_score += score;
            }

            server_score /= video_sizes[video as usize] as i64;


            serv_vid_score_table[server as usize][video as usize] = server_score;
        }
    }

    
    let mut answer = vec![vec![]; n_servers as usize]; // the videos we put in each server
    let mut time_last_printed = Instant::now() - std::time::Duration::from_millis(1000);
    let mut total_score = 0i64; // not normalized (not divided by total n requests and stuff)
    let mut capacity_left = server_capacity as i64 * n_servers as i64;
    let capacity_initial = capacity_left;
    loop {
        let mut score_serv_vid = (0, -1, -1);

        for server in 0..n_servers {
            for video in 0..n_videos {
                if server_capacities[server as usize] < video_sizes[video as usize] { continue; }
    
                let server_score = serv_vid_score_table[server as usize][video as usize];
                score_serv_vid = std::cmp::max(score_serv_vid, (server_score, server, video));
            }
        }

        let (score, server, video) = score_serv_vid;
        if server == -1 { break; }

        server_capacities[server as usize] -= video_sizes[video as usize];

        for &endp_id in &server_endpoints[server as usize] {
            for &req_id in &endp_reqs[endp_id as usize] {
                if reqs[req_id as usize].vid_id != video { continue; }

                let old_latency = best_request_latency[req_id as usize];
                let new_latency = endp_lats[endp_id as usize][&server];

                if old_latency <= new_latency { continue; }

                best_request_latency[req_id as usize] = new_latency;
            }
        }

        answer[server as usize].push(video);

        total_score += score;

        capacity_left -= video_sizes[video as usize] as i64;

        if time_last_printed.elapsed().as_millis() > 500 {
            print!("\x1B[2J\x1B[1;1H"); // clears the console
            println!("{} Total score: {}", in_file, total_score);
            println!("{} Capacity left: {} / {}", in_file, capacity_left, capacity_initial);

            time_last_printed = Instant::now();
        }

        for server in 0..n_servers {
            if server_capacities[server as usize] < video_sizes[video as usize] { continue; }

            let the_reqs = serv_vid_reqs[server as usize].get(&video);
            if the_reqs.is_none() { continue; }
            let the_reqs = the_reqs.unwrap();

            let mut server_score = 0i64;
            for &req_id in the_reqs {
                let endp_id = reqs[req_id as usize].endpoint_id;
                if reqs[req_id as usize].vid_id != video { continue; }

                let old_latency = best_request_latency[req_id as usize] as i64;
                let new_latency = endp_lats[endp_id as usize][&server] as i64;
                
                if old_latency <= new_latency { continue; }

                let n_reqs = reqs[req_id as usize].n_requests as i64;
                let score = (old_latency - new_latency) * n_reqs;
                server_score += score;
            }

            server_score /= video_sizes[video as usize] as i64;

            serv_vid_score_table[server as usize][video as usize] = server_score;
        }
    }

    let mut out_file = BufWriter::new(File::create(out_file)?);
    writeln!(&mut out_file, "{}", answer.iter().filter(|v| !v.is_empty()).count())?;
    for i in 0..answer.len() {
        if answer[i].is_empty() { continue; }
        write!(&mut out_file, "{}", i)?;
        for &j in &answer[i] {
            write!(&mut out_file, " {}", j)?;
        }
        writeln!(&mut out_file)?;
    }

    Ok(())
}

fn check(in_file: &str, out_file: &str) -> Result<i64, Box<dyn std::error::Error>> {
    let Input {
        n_servers,
        server_capacity,
    
        video_sizes,

        endp_lats,
    
        reqs,

        ..
    } = read_problem(in_file);

    let mut answer = vec![vec![]; n_servers as usize]; // the videos we put in each server
    
    let file = std::fs::File::open(out_file.to_string());
    let mut reader = std::io::BufReader::new(file.unwrap());
    let mut buf = String::with_capacity(100000);
    reader.read_line(&mut buf)?;
    let n_cache_descr = buf.split_whitespace().map(usize::from_str).collect::<Result<Vec<_>, _>>()?[0];

    for _ in 0..n_cache_descr {
        buf.clear();
        reader.read_line(&mut buf)?;
        let vec = buf.split_whitespace().map(i32::from_str).collect::<Result<Vec<_>, _>>()?;
        for i in 1..vec.len() {
            answer[vec[0] as usize].push(vec[i]);
        }
    }

    let mut max_space_left = -1;
    for server_id in 0..n_servers as usize {
        let mut size = 0;
        for &video_id in &answer[server_id] {
            size += video_sizes[video_id as usize];
        }
        assert!(size <= server_capacity);
        max_space_left = std::cmp::max(max_space_left, server_capacity - size);
    }
    println!("Max space left for a server: {} / {}", max_space_left, server_capacity);

    let mut score = 0i64;
    for req in &reqs {
        let lats = &endp_lats[req.endpoint_id as usize];
        let mut min_lat = lats[&DC_ID];
        for (&serv_id, &lat) in lats.iter() {
            if serv_id == DC_ID || !answer[serv_id as usize].contains(&req.vid_id) { continue; }
            min_lat = std::cmp::min(min_lat, lat);
        }
        let diff = lats[&DC_ID] - min_lat;
        score += diff as i64 * req.n_requests as i64;
    }
    let final_score = score * 1000 / reqs.iter().map(|r| r.n_requests as i64).sum::<i64>();
    println!("Final score: {}", final_score);

    Ok(final_score)
}
