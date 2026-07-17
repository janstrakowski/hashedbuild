mod cli;

use std::{env, fs, path::PathBuf};

use clap::Parser;
use libhashedbuild::{data::{Map, Value}, eval::eval, runtime::Runtime, tree_sitter::{parse_file, parse_raw}};

fn main() {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Eval { argument, file, source } => {
            let cache_dir_str = match env::var("HASHEDBUILD_CACHE") {
                Ok(val) => val,
                Err(env::VarError::NotPresent) => {
                    eprintln!("Error: HASHEDBUILD_CACHE environment variable is not set to the path of the cache.");
                    std::process::exit(1)
                },
                Err(env::VarError::NotUnicode(_)) => {
                    eprintln!("HASHEDBUILD_CACHE contains invalid unicode characters.");
                    std::process::exit(1)
                },
            };
            let cache_path = PathBuf::from(cache_dir_str);
            if let Err(err) = fs::create_dir_all(&cache_path) {
                eprintln!("Tried to create the cache directory (because it has not exited), but an IO error occured: {err}");
                std::process::exit(1);
            }

            let runtime = match Runtime::start(source, cache_path) {
                Ok(r) => r,
                Err(err) => {
                    eprintln!("Could not start the runtime: {err}.");
                    std::process::exit(1);
                }
            };
            let argument_val;
            if let Some(argument) = argument {
                let argument_ast = match parse_raw(argument.as_bytes(), "/dev/null") {
                    Ok(a) => a,
                    Err(err) => {
                        eprintln!("Could not parse the argument: {err}.");
                        std::process::exit(1);
                    }
                };
                println!("DEBUG: argument AST: {argument_ast:?}");
                argument_val = match eval(
                    &argument_ast,
                    &Map::new(),
                    &Value::Map(Map::new()),
                    &runtime
                ) {
                    Ok(v) => v,
                    Err(err) => {
                        eprintln!("ERROR while evaluating the argument: {err}.");
                        std::process::exit(1);
                    }
                };
            } else {
                argument_val = Value::Map(Map::new());
            }
            println!("DEBUG: argument: {argument_val:?}");

            let ast = match parse_file(file) {
                Ok(a) => a,
                Err(err) => {
                    eprintln!("Could not parse the source: {err}.");
                    std::process::exit(1);
                }
            };
            println!("DEBUG: AST: {ast:?}");
            let val = match eval(
                &ast,
                &Map::new(),
                &argument_val,
                &runtime,
            ) {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("ERROR: {err}.");
                    std::process::exit(1);
                }
            };
            println!("DEBUG: result: {val:?}");

            match val {
                Value::File(file) => {
                    println!("{}", file.path.to_string_lossy());
                },
                _ => {
                    eprintln!("The provided expression did not evaluate in the File type.");
                    std::process::exit(1);
                }
            }
        }
    }
}
