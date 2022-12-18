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

use rand::Rng;
use serde_json;


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
        // ("./input/a.txt", "./output/a.txt"),
        ("./input/b.txt", "./output/b.txt"), // curr solution works ~12 hours 
        // ("./input/c.txt", "./output/c.txt"), // ~2 hours 
        // ("./input/d.txt", "./output/d.txt"),
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
            // let pool = ThreadPool::new(files.len());
            for (in_file, out_file) in files {
                let closure = move || {
                    let timer = Instant::now();

                    match &in_file as &str {
                        "./input/a.txt" => solve(in_file, out_file).unwrap(),
                        "./input/b.txt" => solve(in_file, out_file).unwrap(),
                        "./input/c.txt" => solve(in_file, out_file).unwrap(),
                        "./input/d.txt" => solve(in_file, out_file).unwrap(),
                        _ => panic!("default reached"),
                    };

                    println!("{} time: {}", in_file, timer.elapsed().as_millis());
                };
                closure();
                // pool.execute(closure);
            }
            // pool.join();
            println!("total time: {}", timer.elapsed().as_millis());
        }
        _ => panic!("pass either check or solve"),
    }

    Ok(())
}


fn solve(in_file: &str, out_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::open(in_file.to_string());
    let iter = std::io::BufReader::new(file.unwrap())
        .bytes()
        .map(Result::unwrap);
    let mut red = Red::new(iter);
    /*
        I need: fill each server with the best set of videos.
        What if I make a vec of "best latency for request"
        And also a vec "Possible savings for server for video"
        So it's a map savings_map server -> video -> savings 
        We fill it: for each request, for each server that the endpoint is connected to,
        we put server -> video += dc_latency - server_latency * n_requests

        then, we find the biggest saving, and put the video in the server

        obviously, after we put the video in server, potential savings change.
        For that server -> video, it's zero

        new map video_requests_map: video -> server -> requests for the video connected to the server
        for each request of that video that is connected to that server,
        do: savings_map[server][video] -= (current best latency - new best latency) * n_requests

        new map best_latency_request = dc for all

        // let's also make another map: cache_server -> video -> requests

    */

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

    // println!("Start");
    let start_time = Instant::now();
    // std::thread::sleep( std::time::Duration::from_secs(10));
    // println!("working on reading @ {}", start_time.elapsed().as_millis());
    let n_videos = red.read::<i32>();
    let n_endpoints = red.read::<i32>();
    let n_req_desc = red.read::<i32>();
    let n_cache = red.read::<i32>();  
    let server_capacity = red.read::<i32>();  

    let video_sizes = red.read_vec::<i32>(n_videos as usize);

    let mut server_endpoints = vec![vec![]; n_cache as usize];
    let mut endpoint_servers = vec![vec![]; n_endpoints as usize];
    let mut server_capacities = vec![server_capacity; n_cache as usize];

    const DC_ID: i32 = -1;

    let mut endp_lats = vec![HashMap::<i32, i32>::new(); n_endpoints as usize];

    for i in 0..n_endpoints {
        let latency_datacenter = red.read::<i32>();
        let n_cache_connected = red.read::<i32>();

        endp_lats[i as usize].insert(DC_ID, latency_datacenter);

        for _ in 0..n_cache_connected {
            let cache_id = red.read::<i32>();
            let latency_cache = red.read::<i32>();

            server_endpoints[cache_id as usize].push(i);
            endpoint_servers[i as usize].push(cache_id);

            endp_lats[i as usize].insert(cache_id, latency_cache);
        }
    }

    struct Req {
        vid_id: i32,
        endpoint_id: i32, 
        n_requests: i32,
    }

    // this data structure takes 800 MB
    // server -> video -> list of requests
    let mut serv_vid_reqs: Vec<HashMap<i32, Vec<i32>>> = vec![HashMap::new(); n_cache as usize];


    let mut reqs = Vec::with_capacity(n_req_desc as usize);
    let mut best_request_latency = vec![-1; n_req_desc as usize];
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

        best_request_latency[i as usize] = endp_lats[endpoint_id as usize][&DC_ID];
        endp_reqs[endpoint_id as usize].push(i);
        // takes 800 MB
        for &serv in &endpoint_servers[endpoint_id as usize] {
            serv_vid_reqs[serv as usize].entry(vid_id).or_default().push(i);
        }
    }

    // fn consider(server, video) -> how much latency we save
    // for each endpoint connected to the server
    //     for each request for that endpoint
    //         that is for the video
    //             sum(best_request_latency[request] - new latency * n_requests)

    let server_endpoints = server_endpoints;
    let endpoint_servers = endpoint_servers;

    let mut answer = vec![vec![]; n_cache as usize];


    let mut time_last_printed = Instant::now() - std::time::Duration::from_millis(1000);
    let mut total_score = 0i64;
    let mut videos_added = 0;

    let mut capacity_left = server_capacity as i64 * n_cache as i64;
    let capacity_initial = capacity_left;

    let mut serv_vid_score_table = vec![ vec![ 0i64; n_videos as usize ]; n_cache as usize ];
    for server in 0..n_cache {
        // println!("Server: {}", server);
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
    
    loop {
        let mut score_serv_vid = (0, -1, -1);

        for server in 0..n_cache {
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

        videos_added += 1;

        capacity_left -= video_sizes[video as usize] as i64;

        if time_last_printed.elapsed().as_millis() > 500 {
            print!("\x1B[2J\x1B[1;1H"); // clears the console
            println!("Total score: {}", total_score);
            println!("{} Capacity left: {} / {}", in_file, capacity_left, capacity_initial);
            println!("Videos added: {} / {}, time: {}", videos_added, n_videos, start_time.elapsed().as_secs());

            time_last_printed = Instant::now();
        }

        for server in 0..n_cache {
            // println!("Server: {}", server);
            // for video in 0..n_videos {
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
            // }
        }
    
    }

    // print!("\x1B[2J\x1B[1;1H"); // clears the console
    println!("{} Total score: {}", in_file, total_score);
    println!("{} Capacity left: {} / {}", in_file, capacity_left, capacity_initial);
    println!("{} Videos added: {} / {}", in_file, videos_added, n_videos);

    time_last_printed = Instant::now();

    // println!("finished working on reading @ {}", start_time.elapsed().as_millis());
    // std::thread::sleep( std::time::Duration::from_secs(50));
    // println!("finished @ {}", start_time.elapsed().as_millis());

    let mut max_space_left = -1;
    for server_id in 0..n_cache as usize {
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

    

    Ok(())
}

fn check(in_file: &str, out_file: &str) -> Result<usize, Box<dyn std::error::Error>> {
    Ok(0)
}
