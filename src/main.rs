use dashmap::DashMap;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use std::fs::File;
use std::io::prelude::*;
use std::str;
use std::sync::Arc;
use std::thread;

fn remove_suffix<'a>(s: &'a str, p: &str) -> &'a str {
    match s.find(p) {
        Some(index) => &s[..index],
        None => &s,
    }
}

fn main() {
    let mut file = File::open(r"F:\namuwiki210301\namuwiki_20210301.json").unwrap();
    let mut buf = Arc::new(Vec::new());
    file.read_to_end(Arc::get_mut(&mut buf).unwrap()).unwrap();

    let pb = Arc::new(ProgressBar::new(file.metadata().unwrap().len()));

    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:60.cyan/blue} {pos:>7}/{len:7} {bytes_per_sec} {eta} {msg}",
        )
        .unwrap()
        .progress_chars("##-"),
    );

    let counter_d = DashMap::new();
    let counter: Arc<DashMap<String, u32>> = Arc::new(counter_d);

    let thread_count = num_cpus::get() as u64;
    let each_size = file.metadata().unwrap().len() / thread_count;

    for i in 0..thread_count {
        let counter = counter.clone();
        let pb = pb.clone();
        let mut index = 0;
        let buf = buf.clone();
        let current_job = thread::spawn(move || {
            let mut record = vec![];
            let mut do_record = false;
            let mut is_title = false;
            loop {
                pb.inc(1);

                if (each_size * i + index + 7) as usize >= buf.len()
                    || each_size * i + index + 7 >= each_size * (i + 1)
                {
                    break;
                }

                let buf =
                    &buf[(each_size * i + index) as usize..=(each_size * i + index + 7) as usize];
                index += 1;

                if buf[0] == b'[' && buf[1] == b'[' {
                    do_record = true;
                } else if buf[0] == b']' && buf[1] == b']' {
                    do_record = false;
                } else if buf == b"\"title\":" {
                    is_title = true;
                    do_record = true;
                } else if buf == b",\"text\":" {
                    do_record = false;
                }

                if do_record {
                    record.push(buf[1]);
                } else if record.len() > 0 {
                    if is_title {
                        let mut title = str::from_utf8(&record).unwrap();
                        title = &title[8..title.len() - 2];
                        is_title = false;
                        pb.set_message(title.to_string());

                        record.clear();
                    } else {
                        let mut href = str::from_utf8(&record).unwrap();
                        href = &href[1..href.len() - 1];
                        if href.starts_with("파일:")
                            || href.starts_with("분류:")
                            || href.starts_with("틀:")
                            || href.starts_with("http")
                        {
                            record.clear();
                            continue;
                        }

                        href = remove_suffix(remove_suffix(&href, "|"), "#");
                        *counter.entry(String::from(href)).or_insert(0) += 1;
                        record.clear();
                    }
                }
            }
        });

        if i == thread_count - 1 {
            current_job.join().unwrap();
        }
    }
    pb.finish();
    let mut hash_vec: Vec<_> = counter.iter().collect();
    hash_vec.sort_by(|a, b| b.cmp(a));
    for line in hash_vec.into_iter() {
        println!("\"{}\",{}", line.key(), line.value());
    }
}
