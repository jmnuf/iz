use std::process::ExitCode;
use std::path::{Path, PathBuf};
use std::io;

// Bytes Units
const KILOBYTE: f64 = 1_000.0;
/// 1 MB = 10<sup>6</sup> bytes.
const MEGABYTE: f64 = 1_000_000.0;
/// 1 GB = 10<sup>9</sup> bytes.
const GIGABYTE: f64 = 1_000_000_000.0;
/// 1 TB = 10<sup>12</sup> bytes.
const TERABYTE: f64 = 1_000_000_000_0.0;

fn usage(program: &String) {
    println!("Usage:");
    println!("  {program} [OPTION] [DIR]");
    println!("    -a         Option to display entries that start with `.`");
    println!("    -i         Option to display only the information about a directory instead of its contents");
    println!("    DIR        Provide directory to list contents of");
    println!("    --help     Display this help message");
}

fn get_size<P: AsRef<Path>>(path: P) -> io::Result<u64> {
    let mut result = 0;
    if path.as_ref().is_dir() {
	for entry in std::fs::read_dir(&path)? {
	    let entry_path = entry?.path();
	    if entry_path.is_dir() {
		result += get_size(entry_path)?;
	    } else {
		result += entry_path.metadata()?.len();
	    }
	}
    } else {
	result = path.as_ref().metadata()?.len();
    }
    
    return Ok(result);
}

fn pretty_format_bytes(bytes: u64) -> String {
    let fbytes = bytes as f64;
    if fbytes >= TERABYTE {
	let tb = fbytes / TERABYTE;
	format!("{:.3}TB", tb)
    } else if fbytes >= GIGABYTE {
	let gb = fbytes / GIGABYTE;
	format!("{:.3}GB", gb)
    } else if fbytes >= MEGABYTE {
	let mb = fbytes / MEGABYTE;
	format!("{:.3}MB", mb)
    } else if fbytes >= KILOBYTE {
	let kb = fbytes / KILOBYTE;
	format!("{:.3}KB", kb)
    } else {
	format!("{bytes}B")
    }
}

fn display_info(path: &PathBuf, spacing: &'static str) -> io::Result<()> {
    let file_type = if path.is_dir() {
	"Dir"
    } else if path.is_file() {
	"File"
    } else if path.is_symlink() {
	"Sym"
    } else {
	"???"
    };
    print!("{spacing}Type: `{file_type}` - ");
    if path.is_dir() {
	print!("Entries Count: ");
	if let Ok(entries) = path.read_dir() {
	    let count = entries.count();
	    print!("{count} - ");
	} else {
	    print!("??? - ");
	}
    }
    let file_size = pretty_format_bytes(get_size(path)?);
    let metadata = path.symlink_metadata()?;
    let read_only = metadata.permissions().readonly();
    println!("Size: {file_size} - ReadOnly: {read_only}");
    Ok(())
}

fn display_dir(dir: &PathBuf, show_dots: bool, spacing: &'static str) -> io::Result<()> {
    let mut folders = Vec::new();
    let mut files = Vec::new();
    for entry in dir.read_dir()? {
	if let Ok(entry) = entry {
	    let file_name = entry.file_name().to_string_lossy().to_string();
	    if !show_dots && file_name.starts_with(".") {
		continue;
	    }
	    let file_type = entry.file_type()?;
	    if file_type.is_dir() {
		folders.push(entry.path());
	    } else {
		files.push(entry.path());
	    }
	}
    }
    for f in folders {
	println!("{spacing}\x1b[36m{}\x1b[0m", f.display());
    }
    for f in files {
	println!("{spacing}\x1b[39m{}\x1b[0m", f.display());
    }
    Ok(())
}

#[inline]
fn cur_dir() -> PathBuf {
    PathBuf::from(if cfg!(windows) { ".\\" } else { "./" })
}

fn run(program: &String, args: Vec<String>) -> Result<bool, String> {
    let mut show_dots = false;
    let mut only_info = false;
    let mut appd_info = false;
    let mut directories = Vec::new();
    for arg in args.iter() {
	if arg == "--help" {
	    usage(program);
	    return Ok(true);
	} else if arg == "-a" {
	    show_dots = true;
	    continue;
	} else if arg == "-i" {
	    if !appd_info {
		only_info = true;
	    }
	    continue;
	} else if arg == "-I" {
	    appd_info = true;
	    only_info = false;
	} else if arg.starts_with("-") {
	    let mut chars = arg.chars().skip(1);
	    while let Some(ch) = chars.next() {
		match ch {
		    'a' => {
			show_dots = true;
		    },
		    'i' => {
			if !appd_info {
			    only_info = true;
			}
		    },
		    'I' => {
			appd_info = true;
			only_info = false;
		    },
		    _ => return Err(format!("Unknown flag used. Don't recognize flag `{ch}` from `{arg}`")),
		};
	    }
	    continue;
	} else {
	    directories.push(PathBuf::from(arg));
	}
    }
    if directories.is_empty() {
	directories.push(cur_dir());
    }
    let directories = directories;
    let show_dots = show_dots;
    let only_info = only_info;
    if directories.len() == 1 {
	let path = &directories[0];
	if !path.exists() {
	    return Err("Item doesn't exist!".into());
	}

	return if !path.is_dir() || only_info {
	    print!("{}:", path.display());
	    match display_info(path, "") {
		Ok(_) => Ok(true),
		Err(e) => Err(format!("Failed to get metadata: {e}")),
	    }
	} else {
	    if appd_info {
		match display_info(path, "") {
		    Ok(_) => {},
		    Err(e) => eprintln!("\x1b[31;1mERROR\x1b[0m> Failed to get metadata: {e}"),
		};
	    }
	    match display_dir(path, show_dots, "") {
		Ok(_) => Ok(true),
		Err(e) => Err(format!("Problem happened while attempting to read directory: {e}"))
	    }
	};
    }

    let mut succeeded = 0;
    for path in directories.iter() {
	println!("{}:", path.display());
	if !path.exists() {
	    eprintln!("\x1b[31;1mERROR\x1b[0m> Item doesn't exist!");
	    continue;
	}
	let spacing = "  ";
	if !path.is_dir() || only_info {
	    match display_info(path, spacing) {
		Ok(_) => { succeeded += 1; },
		Err(e) => eprintln!("\x1b[31;1mERROR\x1b[0m> Failed to get metadata: {e}")
	    };
	    continue;
	}

	if appd_info {
	    match display_info(path, "") {
		Ok(_) => {},
		Err(e) => eprintln!("\x1b[31;1mERROR\x1b[0m> Failed to get metadata: {e}"),
	    };
	}
	
	match display_dir(path, show_dots, spacing) {
	    Ok(_) => { succeeded += 1; },
	    Err(e) => eprintln!("\x1b[31;1mERROR\x1b[0m> Problem happened while attempting to read directory: {e}")
	};
    }
    
    return Ok(succeeded > 0);
}

fn main() -> ExitCode {
    let mut args:Vec<String> = std::env::args().collect();
    let program = args.remove(0);
    match run(&program, args) {
        Ok(ok) => if ok { ExitCode::SUCCESS } else { ExitCode::FAILURE },
        Err(err) => {
            eprintln!("\x1b[31;1mERROR\x1b[0m> {err}");
	   usage(&program);
            ExitCode::FAILURE
        }
    }
}
