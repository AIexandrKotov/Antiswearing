use std::io::Read;
use walkdir::WalkDir;

// fn download_file() -> String {
//     let url = "https://raw.githubusercontent.com/bars38/Russian_ban_words/master/words.txt";
//     reqwest::blocking::get(url).unwrap().text().unwrap()
// }

fn get_file(filename: &str) -> String {
    match std::fs::File::open(filename) {
        Ok(mut f) => {
            let mut buf = String::new();
            f.read_to_string(&mut buf).unwrap();
            buf
        }
        Err(_) => {
            // let mut f = std::fs::File::create(filename).unwrap();
            // let s = download_file();
            // f.write_all(s.as_bytes()).unwrap();
            // s
            println!("{} not found. Please, place it near .exe", filename);
            "blya".to_string()
        }
    }
}

trait Analizer {
    fn mark(&mut self, fname: &str, line: usize, badword: &str);
}

fn analize_file<T: Analizer>(analizer: &mut T, filename: &str, badwords: &Vec<String>) {
    let stream_result = std::fs::File::open(filename);
    for bw in badwords {
        if filename.to_lowercase().contains(bw.to_lowercase().as_str()){
            analizer.mark(filename, usize::MAX, bw);
        }
    }
    match stream_result {
        Ok(mut stream) => {
            let mut s = String::new();
            match std::fs::File::read_to_string(&mut stream, &mut s) {
                Ok(_) => {
                    for (i, x) in s.lines().enumerate() {
                        for bw in badwords {
                            if x.to_lowercase().contains(bw.to_lowercase().as_str()) {
                                analizer.mark(filename, i + 1, bw);
                            }
                        }
                    }
                }
                Err(_) => {
                    println!("Error while reading {}", filename);
                }
            }
        }
        Err(_) => println!("{} not available to open", filename),
    }
}

fn analize_dir<T: Analizer>(
    analizer: &mut T,
    dirname: &str,
    badwords: &Vec<String>,
    search_pattern: String,
) {
    for entry in WalkDir::new(dirname)
        .min_depth(1)
        .into_iter()
        .filter_entry(|x| x.path().is_file())
        .filter(|x| {
            x.as_ref()
                .unwrap()
                .path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .contains(search_pattern.as_str())
        })
    {
        analize_file(analizer, entry.unwrap().path().to_str().unwrap(), badwords);
    }
}

fn analize_any<T: Analizer>(
    analizer: &mut T,
    name: &str,
    badwords: &Vec<String>,
    search_pattern: String,
) {
    let md = std::fs::metadata(name).unwrap();
    if md.is_file() {
        analize_file(analizer, name, badwords);
    } else if md.is_dir() {
        analize_dir(analizer, name, badwords, search_pattern);
    }
}

use clap::Parser;

/// Antiswearing
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// extensions of searching files
    #[arg(short, long)]
    search_pattern: Option<String>,
    #[arg(default_value = "")]
    path: String,
}

struct Found {
    filename: String,
    line: usize,
    bw: String,
}

#[derive(Default)]
struct Founds {
    founds: Vec<Found>,
}

impl Analizer for Founds {
    fn mark(&mut self, fname: &str, line: usize, badword: &str) {
        self.founds.push(Found {
            filename: fname.to_string(),
            line,
            bw: badword.to_string(),
        });
    }
}

fn main() {
    let args = Arguments::parse();
    let bws = get_file(
        std::env::current_exe()
            .unwrap()
            .as_path()
            .parent()
            .unwrap()
            .join("antiswearing_badwords.txt")
            .to_str()
            .unwrap(),
    )
    .lines()
    .map(|x| x.to_string())
    .collect::<Vec<String>>();

    let mut analizer = Founds::default();
    analize_any(
        &mut analizer,
        &args.path,
        &bws,
        args.search_pattern.unwrap_or("".to_string()),
    );

    println!("---List of founded badwords: ");
    for f in analizer.founds {
        if f.line == usize::MAX {
            println!("Founded badword \"{}\" at filename {}", f.bw, f.filename)
        } else {
            println!(
                "Founded badword \"{}\" at {} in {}",
                f.bw, f.line, f.filename
            )
        }
    }
    println!("---end List");
}
