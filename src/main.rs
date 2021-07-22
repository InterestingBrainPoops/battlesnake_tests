use anyhow::Result;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::from_str;
use structopt::StructOpt;

use std::{fs::read_to_string, path::PathBuf, process};

use glob::glob;

use colored::*;

#[derive(Deserialize)]
struct TestCaseFile {
    state: serde_json::Value,
    expected: Vec<String>,
    description: Option<String>,
}

struct TestCase {
    state: serde_json::Value,
    expected: Vec<String>,
    description: Option<String>,
    path: PathBuf,
}

#[derive(Debug)]
enum TestResult {
    CorrectMove,
    /// Expected, Actual
    IncorrectMove(Vec<String>, String),
}

struct TestRun {
    test_case: TestCase,
    result: Result<(), TestFailure>,
}

#[derive(Debug)]
enum TestFailure {
    /// Expected, Actual
    IncorrectMove(Vec<String>, String),
    Error(anyhow::Error),
}

impl TestFailure {
    fn display_failure(&self, args: &Args) -> String {
        match self {
            TestFailure::IncorrectMove(expected, actual) => {
                if expected.len() == 1 {
                    let expected = expected.get(0).unwrap();
                    format!(
                        "Moved in the Wrong Direction: Should have moved \"{}\" but moved \"{}\"",
                        expected.color(args.expected_color),
                        actual.color(args.actual_color),
                    )
                } else {
                    let string_wrapped: Vec<_> =
                        expected.iter().map(|e| format!("\"{}\"", e)).collect();
                    format!(
                        "Moved in the Wrong Direction: Should have moved in one of [{}] but moved \"{}\"",
                        string_wrapped.join(", ").color(args.expected_color),
                        actual.color(args.actual_color),
                    )
                }
            }
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

fn run_test(test_case: &TestCase, client: &Client, url: &str) -> Result<TestResult> {
    let response_json: BattlesnakeMoveResponse = client
        .post(url)
        .body(test_case.state.to_string())
        .send()?
        .error_for_status()?
        .json()?;

    let result: TestResult = if test_case.expected.contains(&response_json.r#move) {
        TestResult::CorrectMove
    } else {
        TestResult::IncorrectMove(test_case.expected.clone(), response_json.r#move)
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

    #[structopt(short, long, parse(from_str), default_value = "yellow")]
    expected_color: Color,

    #[structopt(short, long, parse(from_str), default_value = "blue")]
    actual_color: Color,

    #[structopt(short, long, parse(from_str), default_value = "red")]
    failure_color: Color,
}

fn main() -> Result<()> {
    let args = Args::from_args();

    let client = Client::new();

    let mut results: Vec<TestRun> = vec![];

    for entry in glob(&format!("{}/**/*.json", args.test_directory))? {
        let path = entry?;
        let test_case_file: TestCaseFile = from_str(&read_to_string(&path)?)?;
        let test_case = TestCase {
            state: test_case_file.state,
            expected: test_case_file.expected,
            description: test_case_file.description,
            path,
        };
        let x = run_test(&test_case, &client, &args.url);
        let result = match x {
            Ok(TestResult::CorrectMove) => Ok(()),
            Ok(TestResult::IncorrectMove(e, a)) => Err(TestFailure::IncorrectMove(e, a)),
            Err(e) => Err(TestFailure::Error(e)),
        };
        let test_run = TestRun { test_case, result };
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
                "{}: {}\n{}Reason: {}\n\n",
                "Failure on test".color(args.failure_color),
                r.test_case.path.to_str().unwrap(),
                r.test_case
                    .description
                    .as_ref()
                    .map(|a| format!("Description: {} \n", a))
                    .unwrap_or_else(|| "".to_owned()),
                f.display_failure(&args)
            );
        }
    }

    if results.iter().any(|r| r.result.is_err()) {
        process::exit(1)
    }

    Ok(())
}
