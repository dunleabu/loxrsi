//! Lexer throughput benchmark.
//!
//! ```text
//! cargo run --release --bin lexbench -- bench/
//! cargo run --release --bin lexbench -- bench/strings.lox --runs 50
//! cargo run --release --bin lexbench -- my.lox --target-mb 4 --rounds 5
//! ```
//!
//! Always build with `--release`; a debug build measures nothing useful.
//!
//! Small inputs are repeated until they reach `--target-mb` so the timing is
//! long enough to resolve. Repetition is fine for the lexer — it has no state
//! spanning the whole file — but it does mean the working set is the repeated
//! copy, so cache behaviour reflects that size rather than the original file's.
//!
//! Files are benchmarked in interleaved rounds rather than one file to
//! completion, so CPU frequency drift shows up as spread within a column
//! instead of as a fake difference between files. Compare the `best` column
//! across rounds before believing any gap smaller than the round-to-round
//! spread — on a laptop that is easily 5-10%.

use std::env;
use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::Instant;

use loxrsi::lexer::{Token, TokenContext, lex};

const DEFAULT_RUNS: usize = 25;
const DEFAULT_ROUNDS: usize = 3;
const DEFAULT_TARGET_MB: f64 = 1.0;

struct Config {
    paths: Vec<PathBuf>,
    runs: usize,
    rounds: usize,
    target_bytes: usize,
}

const USAGE: &str = "\
usage: lexbench <path>... [options]

  <path>            a .lox file, or a directory of them

options:
  --runs N          timed runs per round, the best is kept (default 25)
  --rounds N        interleaved rounds over all files (default 3)
  --target-mb F     repeat each input up to this size (default 1.0, 0 = off)
  -h, --help        this message";

fn parse_args() -> Result<Config, String> {
    let mut paths = Vec::new();
    let mut runs = DEFAULT_RUNS;
    let mut rounds = DEFAULT_ROUNDS;
    let mut target_mb = DEFAULT_TARGET_MB;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = |name: &str| -> Result<String, String> {
            args.next().ok_or(format!("{} needs a value", name))
        };
        match arg.as_str() {
            "-h" | "--help" => return Err(USAGE.to_string()),
            "--runs" => runs = parse_num(&value("--runs")?, "--runs")?,
            "--rounds" => rounds = parse_num(&value("--rounds")?, "--rounds")?,
            "--target-mb" => {
                let v = value("--target-mb")?;
                target_mb = v
                    .parse()
                    .map_err(|_| format!("--target-mb wants a number, got '{}'", v))?;
            }
            _ if arg.starts_with('-') => return Err(format!("unknown option '{}'", arg)),
            _ => paths.push(PathBuf::from(arg)),
        }
    }

    if paths.is_empty() {
        return Err(USAGE.to_string());
    }
    if runs == 0 || rounds == 0 {
        return Err("--runs and --rounds must be at least 1".to_string());
    }
    Ok(Config {
        paths,
        runs,
        rounds,
        target_bytes: (target_mb * 1024.0 * 1024.0) as usize,
    })
}

fn parse_num(s: &str, name: &str) -> Result<usize, String> {
    s.parse()
        .map_err(|_| format!("{} wants a whole number, got '{}'", name, s))
}

/// Expand directories into the `.lox` files they contain, sorted for a stable
/// report order.
fn collect_files(paths: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for path in paths {
        if path.is_dir() {
            let mut found: Vec<PathBuf> = fs::read_dir(path)
                .map_err(|e| format!("{}: {}", path.display(), e))?
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "lox"))
                .collect();
            if found.is_empty() {
                return Err(format!("{}: no .lox files", path.display()));
            }
            found.sort();
            files.append(&mut found);
        } else {
            files.push(path.clone());
        }
    }
    Ok(files)
}

struct Case {
    name: String,
    original_bytes: usize,
    copies: usize,
    source: String,
    /// Timings from every round, best-of-`runs` each. Seconds.
    times: Vec<f64>,
}

impl Case {
    fn load(path: &Path, target_bytes: usize) -> Result<Case, String> {
        let text =
            fs::read_to_string(path).map_err(|e| format!("{}: {}", path.display(), e))?;
        if text.is_empty() {
            return Err(format!("{}: empty file", path.display()));
        }
        // Repeat up to the target size. Each copy ends in a newline so a
        // trailing line comment in the file can't swallow the next copy.
        let copies = if target_bytes == 0 {
            1
        } else {
            target_bytes.div_ceil(text.len() + 1).max(1)
        };
        let mut source = String::with_capacity((text.len() + 1) * copies);
        for _ in 0..copies {
            source.push_str(&text);
            if !source.ends_with('\n') {
                source.push('\n');
            }
        }
        Ok(Case {
            name: path
                .file_name()
                .map_or_else(|| path.display().to_string(), |n| n.to_string_lossy().into()),
            original_bytes: text.len(),
            copies,
            source,
            times: Vec::new(),
        })
    }

    fn mb(&self) -> f64 {
        self.source.len() as f64 / (1024.0 * 1024.0)
    }

    fn best_mbs(&self) -> f64 {
        self.mb() / self.times.iter().cloned().fold(f64::MAX, f64::min)
    }

    fn worst_mbs(&self) -> f64 {
        self.mb() / self.times.iter().cloned().fold(0.0, f64::max)
    }
}

/// Lex once, defeating dead-code elimination, and report token and error counts.
fn lex_once(source: &str) -> (usize, usize) {
    match lex(black_box(source)) {
        Ok(tokens) => {
            let n = black_box(&tokens).len();
            (n, 0)
        }
        Err(errors) => {
            let n = black_box(&errors).len();
            (0, n)
        }
    }
}

fn main() {
    let config = match parse_args() {
        Ok(c) => c,
        Err(msg) => {
            eprintln!("{}", msg);
            std::process::exit(2);
        }
    };

    let files = match collect_files(&config.paths) {
        Ok(f) => f,
        Err(msg) => {
            eprintln!("{}", msg);
            std::process::exit(1);
        }
    };

    let mut cases = Vec::new();
    for path in &files {
        match Case::load(path, config.target_bytes) {
            Ok(c) => cases.push(c),
            Err(msg) => {
                eprintln!("{}", msg);
                std::process::exit(1);
            }
        }
    }

    if cfg!(debug_assertions) {
        eprintln!("warning: debug build — rebuild with --release for meaningful numbers\n");
    }

    println!(
        "Token {} bytes, TokenContext {} bytes",
        std::mem::size_of::<Token>(),
        std::mem::size_of::<TokenContext>()
    );
    println!(
        "{} runs per round, {} interleaved rounds\n",
        config.runs, config.rounds
    );

    // Warm up: first touch of each buffer pays page faults that would
    // otherwise land on whichever case happens to go first.
    for case in &cases {
        lex_once(&case.source);
    }

    for _ in 0..config.rounds {
        for case in cases.iter_mut() {
            let mut best = f64::MAX;
            for _ in 0..config.runs {
                let start = Instant::now();
                lex_once(&case.source);
                best = best.min(start.elapsed().as_secs_f64());
            }
            case.times.push(best);
        }
    }

    println!(
        "{:<20} {:>8} {:>7} {:>9} {:>10} {:>9} {:>7}",
        "file", "bytes", "copies", "tokens", "best MB/s", "Mtok/s", "spread"
    );
    for case in &cases {
        let (tokens, errors) = lex_once(&case.source);
        let best = case.best_mbs();
        let spread = 100.0 * (best - case.worst_mbs()) / best;
        let mtoks = (tokens as f64 / 1e6) / (case.mb() / best);
        println!(
            "{:<20} {:>8} {:>7} {:>9} {:>10.1} {:>9.2} {:>6.1}%",
            case.name, case.original_bytes, case.copies, tokens, best, mtoks, spread
        );
        if errors > 0 {
            println!("  ^ lexed with {} errors — timing still valid, but this", errors);
            println!("    file exercises the error path, not the token path");
        }
    }
}
