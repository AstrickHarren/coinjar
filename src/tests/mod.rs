use anyhow::{anyhow, bail, Context, Result};
use colored::Colorize;

use crate::transaction::Journal;

struct Test {
    name: String,
    directive: Result<(), String>,
}

fn test_directive(file: &str) -> Result<Test> {
    let input = std::fs::read_to_string(file)?;
    let directive = input.lines().next().ok_or_else(|| anyhow!("empty file"))?;
    let directive = directive.trim_start_matches(';');

    let (cmd, args) = directive
        .split_once(' ')
        .map(|(cmd, args)| (cmd, Err(args.trim().to_string())))
        .unwrap_or_else(|| (directive, Ok(())));

    let directive = match cmd {
        "ok" | "err" => args,
        _ => bail!("invalid directive {}", cmd),
    };

    Ok(Test {
        name: file.to_string(),
        directive,
    })
}

fn test_example(file: &str) -> Result<()> {
    let test = test_directive(file)?;
    let journal = Journal::from_file(&test.name);

    match journal {
        Ok(_) => test.directive.map_err(|e| anyhow!(e)),
        Err(err) => {
            let e = test
                .directive
                .err()
                .ok_or_else(|| anyhow!("{}: expected example failure", file))?;

            format!("{:#}", err)
                .contains(e.as_str())
                .then_some(())
                .ok_or_else(|| anyhow!("{}: expected error {}, got {:#}", file, e, err))
        }
    }
}

#[test]
fn test_examples() -> Result<()> {
    let dir = "./example/";
    let files = std::fs::read_dir(dir)
        .with_context(|| format!("failed to read example directory {}", dir))?;

    for file in files {
        let file = file?.path();
        let file = file.to_str().unwrap();
        test_example(file)?;
        println!("{} {}", "passed".green().bold(), file);
    }

    Ok(())
}
