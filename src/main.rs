//! # project-tree
//!
//! A simple ascii file tree generator.
//!
//! TODO:
//! Is HashMap<PathBuf> really the best way to do this?
//!

use clap::{Parser, ValueEnum};
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

    /// How to process entries specified in any .gitignore files
    #[arg(value_enum)]
    gitignore: Option<GitignoreOpt>,
}

#[derive(ValueEnum, Debug, Clone)]
enum GitignoreOpt {
    /// Do not use .gitignore file
    GiOff,
    /// Ignore all files and directories specified in .gitignore
    GiIgnore,
    /// Do not recurse into directories specified in .gitignore
    GiStop,
    /// Color .gitignore enties a dimmer shade of grey
    GiDim,
    /// A combination of both gi-dim and gi-stop options [Default]
    GiDimAndStop,
}
impl GitignoreOpt {
    fn is_enabled(&self) -> bool {
        match &self {
            GitignoreOpt::GiOff => false,
            _ => true,
        }
    }
    fn should_dim(&self) -> bool {
        match &self {
            GitignoreOpt::GiDim | GitignoreOpt::GiDimAndStop => true,
            _ => false,
        }
    }
    fn should_ignore(&self) -> bool {
        match &self {
            GitignoreOpt::GiIgnore => true,
            _ => false,
        }
    }
    fn should_stop(&self) -> bool {
        match &self {
            GitignoreOpt::GiStop | GitignoreOpt::GiDimAndStop => true,
            _ => false,
        }
    }
}

struct ProjectTree {
    ignore_list: HashSet<PathBuf>,
    stop_list: HashSet<PathBuf>,
    prioritize_dirs: bool,
    gitignore: Option<Gitignore>,
    gitignore_option: GitignoreOpt,
}

impl ProjectTree {
    fn new(
        ignore_list: HashSet<PathBuf>,
        stop_list: HashSet<PathBuf>,
        prioritize_dirs: bool,
        gitignore_option: GitignoreOpt,
    ) -> ProjectTree {
        ProjectTree {
            ignore_list,
            stop_list,
            prioritize_dirs,
            gitignore: None,
            gitignore_option,
        }
    }

    fn scan_folder(
        &mut self,
        cur_path: &PathBuf,
        cur_prefix: String,
        show_lines: bool,
    ) -> io::Result<Vec<String>> {
        let mut files: Vec<String> = Vec::new();

        // If this directory has a .gitignore file apply it for this and all subdirectories
        let mut prev_gitignore = None;
        let mut using_local_gitignore = false;
        if self.gitignore_option.is_enabled() {
            let gitignore_path = cur_path.join(".gitignore");
            if let Ok(true) = fs::exists(&gitignore_path) {
                prev_gitignore = self.gitignore.clone();
                self.gitignore = Some(Gitignore::new(gitignore_path).0);
                using_local_gitignore = true;
            }
        }

        let mut paths: Vec<PathBuf> = fs::read_dir(&cur_path)?
            .filter_map(|entry| {
                let entry: fs::DirEntry = entry.ok()?;
                let path: PathBuf = entry.path();
                if self.ignore_list.contains(&path)
                    || self
                        .ignore_list
                        .contains(path.strip_prefix("./").unwrap_or(&path))
                    || self.ignore_list.contains(&PathBuf::from(entry.file_name()))
                    || (self.gitignore_option.should_ignore()
                        && self.gitignore.is_some()
                        && self
                            .gitignore
                            .as_ref()
                            .unwrap()
                            .matched(&path, path.is_dir())
                            .is_ignore())
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
            if let Some(gitignore) = &self.gitignore {
                if let Match::Ignore(_) = gitignore.matched(path, is_dir) {
                    if self.gitignore_option.should_dim() {
                        colored_filename = filename.dimmed();
                    }
                    git_ignored = true;
                }
            }

            files.push(format!(
                "{cur_prefix}{affix}{colored_filename}{}",
                if is_dir { "/" } else { "" }
            ));

            if is_dir
                && !(git_ignored && self.gitignore_option.should_stop())
                && !self.stop_list.contains(path)
                && !self
                    .stop_list
                    .contains(path.strip_prefix("./").unwrap_or(&path))
                && !self.stop_list.contains(&PathBuf::from(filename))
            {
                let new_prefix = format!("{cur_prefix}{}", if is_last { "    " } else { "│   " });

                let mut sub_files: Vec<String> =
                    self.scan_folder(path, new_prefix.clone(), true)?;
                if git_ignored {
                    sub_files = sub_files
                        .iter()
                        .map(|s| s.strip_prefix(&new_prefix).unwrap_or(&s))
                        .map(|s| s.dimmed().to_string())
                        .map(|s| {
                            let mut s2 = new_prefix.clone();
                            s2.push_str(&s);
                            s2
                        })
                        .collect();
                }
                files.append(&mut sub_files);
            }
        }

        if using_local_gitignore {
            self.gitignore = prev_gitignore;
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

    let mut tree: String = ProjectTree::new(
        ignore_list,
        stop_list,
        args.dirs,
        args.gitignore.unwrap_or(GitignoreOpt::GiDimAndStop),
    )
    .scan_folder(&PathBuf::from("./"), String::from(""), args.root)
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
