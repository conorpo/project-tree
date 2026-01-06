//! # project-tree
//!
//! A simple ascii file tree generator.
//!
//! TODO:
//! Is HashMap<PathBuf> really the best way to do this?
//!

use clap::Parser;
use clipboard::{ClipboardContext, ClipboardProvider};
use colored::Colorize;
use ignore::gitignore::Gitignore;
use ignore::Match;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Files to ignore in the tree
    #[arg(short, long, value_name = "FILE")]
    ignore: Vec<String>,

    /// Files to not recurse into
    #[arg(short, long, value_name = "FILE")]
    stop: Vec<String>,

    /// Output file
    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,

    /// Show node_modules
    #[arg(long)]
    node_modules: bool,

    /// Show .git
    #[arg(long)]
    git: bool,

    /// Show .vscode
    #[arg(long)]
    vscode: bool,

    /// Recurse into target in Rust projects
    #[arg(long)]
    target: bool,

    /// Do not copy to clipboard
    #[arg(long)]
    noclip: bool,

    /// Include root
    #[arg(short, long)]
    root: bool,

    /// Prioritize directories
    #[arg(short, long)]
    dirs: bool,
}

struct ProjectTree {
    ignore_list: HashSet<PathBuf>,
    stop_list: HashSet<PathBuf>,
    prioritize_dirs: bool,
}

impl ProjectTree {
    fn new(
        ignore_list: HashSet<PathBuf>,
        stop_list: HashSet<PathBuf>,
        prioritize_dirs: bool,
    ) -> ProjectTree {
        ProjectTree {
            ignore_list,
            stop_list,
            prioritize_dirs,
        }
    }

    fn scan_folder(
        &self,
        cur_path: &PathBuf,
        cur_prefix: String,
        show_lines: bool,
        git_ignore: &Option<Gitignore>,
    ) -> io::Result<Vec<String>> {
        let mut files: Vec<String> = Vec::new();

        let mut paths: Vec<PathBuf> = fs::read_dir(&cur_path)?
            .filter_map(|entry| {
                let entry: fs::DirEntry = entry.ok()?;
                let path: PathBuf = entry.path();
                if self.ignore_list.contains(&path)
                    || self
                        .ignore_list
                        .contains(path.strip_prefix("./").unwrap_or(&path))
                    || self.ignore_list.contains(&PathBuf::from(entry.file_name()))
                {
                    None
                } else {
                    Some(path)
                }
            })
            .collect();

        if self.prioritize_dirs {
            paths.sort_by_key(|path| !path.is_dir());
        }

        for (i, path) in paths.iter().enumerate() {
            let is_dir: bool = path.is_dir();
            let is_last: bool = i == paths.len() - 1;

            let affix = match (show_lines, is_last) {
                (true, true) => "└── ",
                (true, false) => "├── ",
                (false, _) => "",
            };
            let filename: &std::ffi::OsStr = path.file_name().unwrap_or_default();
            let filename: &str = filename.to_str().unwrap_or_default();

            let mut colored_filename = filename.normal();
            let mut git_ignored = false;
            if let Some(gitignore) = git_ignore {
                if let Match::Ignore(_) = gitignore.matched(path, is_dir) {
                    colored_filename = filename.dimmed();
                    git_ignored = true;
                }
            }

            files.push(format!(
                "{cur_prefix}{affix}{colored_filename}{}",
                if is_dir { "/" } else { "" }
            ));

            if is_dir
                && !self.stop_list.contains(path)
                && !self
                    .stop_list
                    .contains(path.strip_prefix("./").unwrap_or(&path))
                && !self.stop_list.contains(&PathBuf::from(filename))
            {
                let new_prefix = format!("{cur_prefix}{}", if is_last { "    " } else { "│   " });

                let mut sub_files: Vec<String> =
                    self.scan_folder(path, new_prefix, true, &git_ignore)?;
                if git_ignored {
                    sub_files = sub_files.iter().map(|s| s.dimmed().to_string()).collect();
                }
                files.append(&mut sub_files);
            }
        }

        Ok(files)
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let mut clipboard: ClipboardContext = ClipboardProvider::new().unwrap();

    let mut ignore_list: HashSet<PathBuf> = HashSet::new();
    if !args.git {
        ignore_list.insert(PathBuf::from(".git"));
    }
    if !args.vscode {
        ignore_list.insert(PathBuf::from(".vscode"));
    }

    for ignore in args.ignore {
        ignore_list.insert(PathBuf::from(ignore));
    }

    let mut stop_list: HashSet<PathBuf> = HashSet::new();
    if !args.node_modules {
        stop_list.insert(PathBuf::from("node_modules"));
    }

    // If this is a Rust project stop at target dir unless target arg set
    if !args.target {
        if let Ok(true) = fs::exists("Cargo.toml") {
            stop_list.insert(PathBuf::from("target"));
        }
    }

    for stop in args.stop {
        stop_list.insert(PathBuf::from(stop));
    }

    // If this project has a .gitignore file, use it to colour ignored files
    let mut git_ignore: Option<Gitignore> = None;
    let gitignore_path = PathBuf::from(".gitignore");
    if let Ok(true) = fs::exists(&gitignore_path) {
        println!("Using .gitignore");
        git_ignore = Some(Gitignore::new(gitignore_path).0);
    }

    let mut tree: String = ProjectTree::new(ignore_list, stop_list, args.dirs)
        .scan_folder(
            &PathBuf::from("./"),
            String::from(""),
            args.root,
            &git_ignore,
        )
        .unwrap()
        .join("\n");

    //Get Root Dir Name
    if args.root {
        let root_dir: String = std::env::current_dir()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        tree = format!("{root_dir}\n{tree}");
    }

    println!("{tree}");
    if let Some(output_file) = args.output {
        fs::write(output_file, &tree)?;
    }

    if !args.noclip {
        clipboard.set_contents(tree).unwrap();
    }

    Ok(())
}
