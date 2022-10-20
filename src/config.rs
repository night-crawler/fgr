use std::path::PathBuf;

use clap::Parser;

use crate::{parse_root, ExpressionNode, GenericError};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = r###"fgr: find & grep program.

You can build complex query expressions in the form of:
    (FILTER1 op FILTER2) or FILTER3

FILTER syntax is:
    NAME eq_op VALUE

NAME can be any of: name, extension, mtime, atime, size, contains, depth, permissions, group, user, type.
NAME supports aliases. Run fgr with --syntax to get more information.
VALUE can be a number, a number with a qualifier (Mb, hour), or a PATTERN.
PATTERN can be either a glob (sample*) or regex: r"sample.+" or r'sample.+'.
PATTERN can be either a plain expression (*glob*) or it can be wtapped in quotes: ('*glob') or ("*glob*").

Examples:
    Find all files with name equal to sample under the current directory:
    fgr -e name=sample

    Find files with containing 's' and 777 permissions:
    fgr /home /bin -e 'name=*s* and perm=777'

    Find files under the /bin directory not owned by root:
    fgr /bin -e 'user > 0'

    Find files under the /bin directory having suid bit (but not limited to):
    fgr /bin -e 'perms>4000'

    Find recently accessed files (but not in future):
    fgr /home -e 'atime > now - 1h and atime < now - 0h'

    Find stuff in files:
    fgr /home -e 'type=text and contains=*stuff*'

    Other examples:
    fgr /home /bin -e 'name=*s* and perm=777 or (name=*rs and contains=r".+user.is_birthday.*")'
    fgr /home /bin -e 'name=*s* and perm=777 or (name=*rs and contains=*birth*)'
    fgr /home /bin -e 'ext=so and mtime >= now - 1d'
    fgr /home -e 'size>=1Mb and name != *.rs and type=vid'
"###
)]
pub struct Args {
    /// A list of directories where to search
    start_dirs: Option<Vec<String>>,

    /// Expression to evaluate on each file
    #[arg(short)]
    expression: String,

    /// Print expression tree graphviz schema and exit
    #[arg(short = 'q', long, default_value_t = false)]
    print_expression_tree: bool,

    #[arg(short, long, default_value_t = num_cpus::get())]
    threads: usize,

    /// Enable all standard filters (all filters below)
    #[arg(short, long, default_value_t = false)]
    all: bool,

    /// Ignore hidden files
    #[arg(long)]
    ignore_hidden: Option<bool>,

    /// Read ignore files from parent directories
    #[arg(long)]
    read_parents: Option<bool>,

    /// Read ignore files from parent directories
    #[arg(long)]
    read_ignore: Option<bool>,

    /// Read .gitignore
    #[arg(long)]
    read_git_ignore: Option<bool>,

    /// Read a global gitignore file
    #[arg(long)]
    read_git_global: Option<bool>,

    /// Read .git/info/exclude
    #[arg(long)]
    read_git_exclude: Option<bool>,

    /// Same filesystem
    #[arg(long)]
    same_filesystem: Option<bool>,
}

#[derive(Debug)]
pub struct Config {
    pub start_dirs: Vec<PathBuf>,
    pub root: ExpressionNode,

    pub threads: usize,

    pub standard_filters: bool,
    pub hidden: Option<bool>,
    pub parents: Option<bool>,
    pub ignore: Option<bool>,
    pub git_ignore: Option<bool>,
    pub git_global: Option<bool>,
    pub git_exclude: Option<bool>,

    pub same_filesystem: Option<bool>,

    pub print_expression_tree: bool,
}

impl Config {
    pub fn build() -> Result<Self, GenericError> {
        let args: Args = Args::parse();

        let start_dirs = if let Some(dirs) = args.start_dirs {
            dirs.into_iter().map(PathBuf::from).collect()
        } else {
            vec![std::env::current_dir()?]
        };

        let root = parse_root(&args.expression)?;

        Ok(Config {
            start_dirs,
            root,

            threads: args.threads,

            standard_filters: args.all,
            hidden: args.ignore_hidden,
            parents: args.read_parents,
            ignore: args.read_ignore,
            git_ignore: args.read_git_ignore,
            git_global: args.read_git_global,
            git_exclude: args.read_git_exclude,

            same_filesystem: args.same_filesystem,

            print_expression_tree: args.print_expression_tree,
        })
    }
}
