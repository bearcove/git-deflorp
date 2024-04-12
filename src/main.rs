use std::process::Command;

use camino::Utf8PathBuf;

fn main() {
    // make sure `git --version` runs
    match Command::new("git").arg("--version").output() {
        Ok(output) => {
            println!("Git version: {}", String::from_utf8(output.stdout).unwrap());
        }
        Err(e) => {
            println!("Could not run `git --version`: {}", e);
            std::process::exit(1);
        }
    }

    // find the nearest `.git` directory
    let repo = match Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
    {
        Ok(output) => Utf8PathBuf::from(std::str::from_utf8(&output.stdout).unwrap().trim()),
        Err(e) => {
            println!("Could not find nearest .git directory: {}", e);
            std::process::exit(1);
        }
    };
    println!("Operating on repo '{repo}'");

    let mut gh_token: Option<String> = None;

    // grab github token from GITHUB_TOKEN environment variable
    match std::env::var("GITHUB_TOKEN") {
        Ok(token) => {
            gh_token = Some(token);
            println!("Grabbed GitHub token from GITHUB_TOKEN environment variable");
        }
        Err(e) => {
            // do nothing
        }
    }

    // grab github token from the output of the command `gh auth token`
    if gh_token.is_none() {
        match Command::new("gh").arg("auth").arg("token").output() {
            Ok(output) => {
                gh_token = Some(
                    std::str::from_utf8(&output.stdout)
                        .unwrap()
                        .trim()
                        .to_owned(),
                );
                println!("Grabbed GitHub token from `gh auth token`");
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    if gh_token.is_none() {
        println!("Could not find GitHub token. Tried: GITHUB_TOKEN environment variable, `gh auth token` command.");
        std::process::exit(1);
    }

    // show first 4 and last 4 of github token
}
