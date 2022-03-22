// https://siciarz.net/24-days-rust-git2/

use git2::{Direction, Repository, Signature};
use std::path::Path;

fn push(repo: &Repository) -> Result<(), git2::Error> {
    let mut remote = repo.find_remote("origin").expect("Repository does not have remote 'origin' configured.");
    remote.connect(Direction::Push)?;
    remote.push(&["refs/heads/main:refs/heads/main"], None)
}

pub fn update(
    repo_path: &Path,
    target_repo: &str,
    checkout: &str,
    path: &str,
) -> Result<(), git2::Error> {
    let repo_handle = Repository::open(repo_path)?;
    let mut index = repo_handle.index()?;

    let pairs = vec![
        ("repository.txt", target_repo),
        ("checkout.txt", checkout),
        ("path.txt", path),
    ];

    println!("Writing output to repository...");

    for (path, value) in pairs {
        std::fs::write(repo_path.join(path), value).unwrap();
        index.add_path(Path::new(path))?;
    }

    println!("Done writing output.");

    let oid = index.write_tree()?;

    let signature = Signature::now("Contract Registry Bot", "contract-registry@stats.gallery")?;

    let parent_commit = repo_handle.head()?.resolve()?.peel_to_commit()?;

    let tree = repo_handle.find_tree(oid)?;

    let commit_oid = repo_handle.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &format!("Automated update\n\tTarget repository: {target_repo}\n\tCheckout: {checkout}\n\tPath: {path}"),
        &tree,
        &[&parent_commit],
    )?;

    println!("Commit hash: {commit_oid:?}");

    println!("Pushing...");

    push(&repo_handle)?;

    println!("Done.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::update;

    #[test]
    fn test() {
        println!("running...");
        println!(
            "{:?}",
            update(
                &Path::new("C:/Users/Jacob/Projects/contract-registry-ci-test"),
                "https://github.com/NEAR-Edu/stats.gallery-dapp.git",
                "main",
                "",
            )
        );
    }
}
