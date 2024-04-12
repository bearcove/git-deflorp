use std::process::Command;

use camino::Utf8PathBuf;
use sha1::{Digest, Sha1};

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
        Err(_) => {
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

    let object_hash = git_hash::Kind::Sha1;

    // list all git packs. those are `.pack` files in the `.git/objects/pack` directory
    let pack_dir = repo.join(".git").join("objects").join("pack");
    for entry in pack_dir.read_dir().unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().unwrap() == "pack" {
            println!("Found pack: {}", path.display());

            let bundle = git_pack::Bundle::at(path, object_hash).unwrap();
            for idx_entry in bundle.index.iter() {
                println!("Entry: {:?}", idx_entry);

                let pack_entry = bundle.pack.entry(idx_entry.pack_offset);
                let mut out = vec![0u8; pack_entry.decompressed_size as usize];
                let n = bundle.pack.decompress_entry(&pack_entry, &mut out).unwrap();
                println!(
                    "Decompressed {} bytes (expected {})",
                    n, pack_entry.decompressed_size
                );

                // compute the crc32 of out
                let crc32 = crc32fast::hash(&out);
                // compare with the crc32 in the index
                println!("CRC32: {:x}, index {:x}", crc32, idx_entry.crc32.unwrap());
                // if let Some(idx_crc32) = idx_entry.crc32 {
                //     assert_eq!(crc32, idx_crc32, "CRC32 mismatch");
                // }

                // compute the sha-1 hash of out, using the right git_hash::Kind
                let mut hasher = Sha1::new();
                hasher.update(&out);
                let result = hasher.finalize();
                println!("SHA1: {:x}", result);
            }
        }
    }
}
