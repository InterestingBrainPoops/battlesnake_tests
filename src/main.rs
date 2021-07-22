use anyhow::Result;
use reqwest::blocking::Client;
use serde::Deserialize;
use structopt::StructOpt;

use std::{
    fs::{read_to_string, File},
    path::{Path, PathBuf},
    process,
};

use glob::glob;

use colored::*;

#[derive(Debug)]
enum TestResult {
    CorrectMove,
    /// Expected, Actual
    IncorrectMove(String, String),
}

#[derive(Debug)]
struct TestRun {
    test_path: PathBuf,
    result: Result<(), TestFailure>,
}

#[derive(Debug)]
enum TestFailure {
    /// Expected, Actual
    IncorrectMove(String, String),
    Error(anyhow::Error),
}

impl TestFailure {
    fn display_failure(&self, colors: &ColorOptions) -> String {
        match self {
            TestFailure::IncorrectMove(expected, actual) => format!(
                "Moved in the Wrong Direction: Should have moved \"{}\" but moved \"{}\"",
                expected.color(colors.expected),
                actual.color(colors.actual),
            ),
            TestFailure::Error(e) => format!("Error {}", e),
        }
    }
}

#[derive(Deserialize, Debug)]
struct BattlesnakeMoveResponse {
    r#move: String,
    #[serde(default = "default_shout")]
    shout: Option<String>,
}

fn default_shout() -> Option<String> {
    None
}

fn run_test(input_path: &Path, client: &Client, url: &str) -> Result<TestResult> {
    let output_path = {
        let mut path = input_path.to_path_buf();
        path.set_file_name("output.json");
        path
    };

    let input = File::open(input_path)?;
    let output_contents = read_to_string(output_path)?;
    let output_json: BattlesnakeMoveResponse = serde_json::from_str(&output_contents)?;

    let response_json: BattlesnakeMoveResponse = client
        .post(url)
        .body(input)
        .send()?
        .error_for_status()?
        .json()?;

    let result: TestResult = if response_json.r#move == output_json.r#move {
        TestResult::CorrectMove
    } else {
        TestResult::IncorrectMove(output_json.r#move, response_json.r#move)
    };

    Ok(result)
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "battlesnake_tests",
    about = "A simple CLI that can run a set of Battlesnake Tests against a given URL"
)]
struct Args {
    #[structopt(short = "u", long = "url", name = "Battlesnake URL to test against")]
    url: String,

    #[structopt(
        short = "d",
        long = "dir",
        name = "Directory containing test cases",
        default_value = "./tests/"
    )]
    test_directory: String,

    #[structopt(short, long)]
    expected_color: Option<String>,

    #[structopt(short, long)]
    actual_color: Option<String>,

    #[structopt(short, long)]
    failure_color: Option<String>,
}

struct ColorOptions {
    expected: Color,
    actual: Color,
    failure: Color,
}

fn parse_color(input: &Option<String>, default: Color) -> Color {
    match input {
        Some(s) => s.parse().unwrap_or_else(|_| {
            eprintln!("Failed to parse color {}, falling back to default", s,);
            default
        }),
        None => default,
    }
}

fn main() -> Result<()> {
    let args = Args::from_args();
    let color_options = ColorOptions {
        expected: parse_color(&args.expected_color, Color::Yellow),
        actual: parse_color(&args.actual_color, Color::Blue),
        failure: parse_color(&args.failure_color, Color::Red),
    };

    let client = Client::new();

    let mut results: Vec<TestRun> = vec![];

    for entry in glob(&format!("{}/**/input.json", args.test_directory))? {
        let path = entry?;
        let x = run_test(&path, &client, &args.url);
        let result = match x {
            Ok(TestResult::CorrectMove) => Ok(()),
            Ok(TestResult::IncorrectMove(e, a)) => Err(TestFailure::IncorrectMove(e, a)),
            Err(e) => Err(TestFailure::Error(e)),
        };
        let test_run = TestRun {
            result,
            test_path: path,
        };
        results.push(test_run);
    }

    let successful_count = results.iter().filter(|x| x.result.is_ok()).count();

    let total_count = results.len();

    println!(
        "{} out of {} tests passed!\n\n",
        successful_count, total_count
    );

    for r in &results {
        if let Err(f) = &r.result {
            println!(
                "{}: {}\nReason: {}\n\n",
                "Failure on test".color(color_options.failure),
                r.test_path.to_str().unwrap(),
                f.display_failure(&color_options)
            );
        }
    }

    if results.iter().any(|r| r.result.is_err()) {
        process::exit(1)
    }

    Ok(())
}
