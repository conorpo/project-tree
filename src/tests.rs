use std::env;
use tempdir::TempDir;

use super::*;

macro_rules! entries {
    () => {{
       HashSet::new()
    }};
    ($($x:expr),+) => {{
        let mut list: HashSet<PathBuf> = HashSet::new();
        $(
            list.insert(PathBuf::from($x));
        )*
        list
    }};
}

// Run with gitignore behaviour off to check old behaviour is preserved
fn run(
    temp_dir: &TempDir,
    ignore_list: HashSet<PathBuf>,
    stop_list: HashSet<PathBuf>,
    prioritise_dirs: bool,
    root: bool,
) -> String {
    run_with_gitignore(
        temp_dir,
        ignore_list,
        stop_list,
        prioritise_dirs,
        root,
        GitignoreOpt::GiOff,
    )
}

fn run_with_gitignore(
    temp_dir: &TempDir,
    ignore_list: HashSet<PathBuf>,
    stop_list: HashSet<PathBuf>,
    prioritise_dirs: bool,
    root: bool,
    gitignore: GitignoreOpt,
) -> String {
    let project_dir = temp_dir.path().join("project");
    env::set_current_dir(project_dir).unwrap();
    let mut result = ProjectTree::new(ignore_list, stop_list, prioritise_dirs, gitignore)
        .scan_folder(&PathBuf::from("./"), String::from(""), root)
        .unwrap()
        .join("\n");
    if root {
        result = format!("project\n{result}");
    }
    result
}

fn create_test_rust_project() -> TempDir {
    // Create a temp dir with a random suffix that will get deleted when the owner is dropped
    let temp_dir = TempDir::new("project-tree-test").unwrap();
    let project_dir = temp_dir.path().join("project");
    fs::create_dir(&project_dir).unwrap();
    // Populate it with a representative project structure
    let dirs = vec!["src", "target", "target/debug", "target/release"];
    let files = vec![
        ".gitignore",
        "Cargo.toml",
        "Cargo.lock",
        "README.md",
        "src/main.rs",
    ];
    for dir in &dirs {
        let path = project_dir.join(dir);
        fs::create_dir(path).unwrap();
    }
    for file in &files {
        let path = project_dir.join(file);
        fs::write(path, "test data").unwrap();
    }
    temp_dir
}

// Create a more advanced project with some .gitignore rules
fn create_test_rust_project_with_gitignore() -> TempDir {
    let temp_dir = create_test_rust_project();
    let project_dir = temp_dir.path().join("project");

    // Create a cache directory that we want to ignore
    fs::create_dir(project_dir.join("cache")).unwrap();
    fs::write(
        project_dir.join("cache").join("cache_file1.dat"),
        "junk data",
    )
    .unwrap();
    fs::write(
        project_dir.join("cache").join("cache_file2.dat"),
        "junk data",
    )
    .unwrap();

    // Create the .gitignore contents
    fs::write(
        project_dir.join(".gitignore"),
        "\
/target
cache
",
    )
    .unwrap();

    temp_dir
}

#[test]
fn test_basic_usage() {
    let temp_dir = create_test_rust_project();
    let prioritise_dirs = false;
    let root = false;
    let ignore_list = entries!();
    let stop_list = entries!();
    let tree = run(&temp_dir, ignore_list, stop_list, prioritise_dirs, root);
    assert_eq!(
        tree,
        "\
.gitignore
Cargo.lock
Cargo.toml
README.md
src/
│   └── main.rs
target/
    ├── debug/
    └── release/"
    );
}

#[test]
fn test_prioritise_dirs() {
    let temp_dir = create_test_rust_project();
    let prioritise_dirs = true;
    let root = false;
    let ignore_list = entries!();
    let stop_list = entries!();
    let tree = run(&temp_dir, ignore_list, stop_list, prioritise_dirs, root);
    assert_eq!(
        tree,
        "\
src/
│   └── main.rs
target/
│   ├── debug/
│   └── release/
.gitignore
Cargo.lock
Cargo.toml
README.md"
    );
}

#[test]
fn test_stop() {
    let temp_dir = create_test_rust_project();
    let prioritise_dirs = false;
    let root = false;
    let ignore_list = entries!();
    let stop_list = entries!("target");
    let tree = run(&temp_dir, ignore_list, stop_list, prioritise_dirs, root);
    assert_eq!(
        tree,
        "\
.gitignore
Cargo.lock
Cargo.toml
README.md
src/
│   └── main.rs
target/"
    );
}

#[test]
fn test_root() {
    let temp_dir = create_test_rust_project();
    let prioritise_dirs = false;
    let root = true;
    let ignore_list = entries!();
    let stop_list = entries!("target");
    let tree = run(&temp_dir, ignore_list, stop_list, prioritise_dirs, root);
    assert_eq!(
        tree,
        "\
project
├── .gitignore
├── Cargo.lock
├── Cargo.toml
├── README.md
├── src/
│   └── main.rs
└── target/"
    );
}

#[test]
fn test_ignore_absolute() {
    let temp_dir = create_test_rust_project();
    let prioritise_dirs = false;
    let root = true;
    let ignore_list = entries!("./src/main.rs");
    let stop_list = entries!("./target");
    let tree = run(&temp_dir, ignore_list, stop_list, prioritise_dirs, root);
    assert_eq!(
        tree,
        "\
project
├── .gitignore
├── Cargo.lock
├── Cargo.toml
├── README.md
├── src/
└── target/"
    );
}

#[test]
fn test_ignore_absolute_no_dot() {
    let temp_dir = create_test_rust_project();
    let prioritise_dirs = false;
    let root = true;
    let ignore_list = entries!("src/main.rs");
    let stop_list = entries!("target");
    let tree = run(&temp_dir, ignore_list, stop_list, prioritise_dirs, root);
    assert_eq!(
        tree,
        "\
project
├── .gitignore
├── Cargo.lock
├── Cargo.toml
├── README.md
├── src/
└── target/"
    );
}

#[test]
fn test_ignore_relative() {
    let temp_dir = create_test_rust_project();
    let prioritise_dirs = false;
    let root = true;
    let ignore_list = entries!("main.rs", "Cargo.lock");
    let stop_list = entries!("target");
    let tree = run(&temp_dir, ignore_list, stop_list, prioritise_dirs, root);
    assert_eq!(
        tree,
        "\
project
├── .gitignore
├── Cargo.toml
├── README.md
├── src/
└── target/"
    );
}

// GitIgnore tests
#[test]
fn test_gitignore_dim_and_stop() {
    let temp_dir = create_test_rust_project_with_gitignore();
    let prioritise_dirs = false;
    let root = true;
    let ignore_list = entries!();
    let stop_list = entries!();
    let tree = run_with_gitignore(
        &temp_dir,
        ignore_list,
        stop_list,
        prioritise_dirs,
        root,
        GitignoreOpt::GiDimAndStop,
    );
    assert_eq!(
        tree,
        "\
project
├── .gitignore
├── \u{1b}[2mcache\u{1b}[0m/
├── Cargo.lock
├── Cargo.toml
├── README.md
├── src/
│   └── main.rs
└── \u{1b}[2mtarget\u{1b}[0m/"
    );
}

#[test]
fn test_gitignore_stop() {
    let temp_dir = create_test_rust_project_with_gitignore();
    let prioritise_dirs = false;
    let root = true;
    let ignore_list = entries!();
    let stop_list = entries!();
    let tree = run_with_gitignore(
        &temp_dir,
        ignore_list,
        stop_list,
        prioritise_dirs,
        root,
        GitignoreOpt::GiStop,
    );
    assert_eq!(
        tree,
        "\
project
├── .gitignore
├── cache/
├── Cargo.lock
├── Cargo.toml
├── README.md
├── src/
│   └── main.rs
└── target/"
    );
}

#[test]
fn test_gitignore_ignore() {
    let temp_dir = create_test_rust_project_with_gitignore();
    let prioritise_dirs = false;
    let root = true;
    let ignore_list = entries!();
    let stop_list = entries!();
    let tree = run_with_gitignore(
        &temp_dir,
        ignore_list,
        stop_list,
        prioritise_dirs,
        root,
        GitignoreOpt::GiIgnore,
    );
    assert_eq!(
        tree,
        "\
project
├── .gitignore
├── Cargo.lock
├── Cargo.toml
├── README.md
└── src/
    └── main.rs"
    );
}

#[test]
fn test_gitignore_dim() {
    let temp_dir = create_test_rust_project_with_gitignore();
    let prioritise_dirs = false;
    let root = true;
    let ignore_list = entries!();
    let stop_list = entries!();
    let tree = run_with_gitignore(
        &temp_dir,
        ignore_list,
        stop_list,
        prioritise_dirs,
        root,
        GitignoreOpt::GiDim,
    );
    assert_eq!(
        tree,
        "\
project
├── .gitignore
├── \u{1b}[2mcache\u{1b}[0m/
│   \u{1b}[2m├── cache_file1.dat\u{1b}[0m
│   \u{1b}[2m└── cache_file2.dat\u{1b}[0m
├── Cargo.lock
├── Cargo.toml
├── README.md
├── src/
│   └── main.rs
└── \u{1b}[2mtarget\u{1b}[0m/
    \u{1b}[2m├── debug/\u{1b}[0m
    \u{1b}[2m└── release/\u{1b}[0m"
    );
}

#[test]
fn test_all_together() {
    let temp_dir = create_test_rust_project_with_gitignore();
    let prioritise_dirs = true;
    let root = true;
    let ignore_list = entries!("cache", ".gitignore");
    let stop_list = entries!("./target");
    let tree = run_with_gitignore(
        &temp_dir,
        ignore_list,
        stop_list,
        prioritise_dirs,
        root,
        GitignoreOpt::GiDim,
    );
    assert_eq!(
        tree,
        "\
project
├── src/
│   └── main.rs
├── \u{1b}[2mtarget\u{1b}[0m/
├── Cargo.lock
├── Cargo.toml
└── README.md"
    );
}
