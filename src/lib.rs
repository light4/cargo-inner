use anyhow::Result;
use cargo_metadata::MetadataCommand;
use std::collections::HashSet;
use std::process::Command;

pub fn run() {
    let mut args = std::env::args().skip_while(|val| !val.starts_with("--manifest-path"));

    let mut cmd = MetadataCommand::new();
    let _manifest_path = match args.next() {
        Some(ref p) if p == "--manifest-path" => {
            cmd.manifest_path(args.next().unwrap());
        }
        Some(p) => {
            cmd.manifest_path(p.trim_start_matches("--manifest-path="));
        }
        None => {}
    };

    let metadata = cmd.exec().unwrap();

    let root_package = metadata.root_package().unwrap();
    let mut root_deps = HashSet::new();
    for dep in &root_package.dependencies {
        // println!("        dep {}: {:?}", dep.name, dep);
        root_deps.insert(dep.name.to_owned());
    }

    let mut all_other_deps = HashSet::new();
    for pkg in &metadata.packages {
        println!(
            "name: {}, version: {}, edition: {}",
            pkg.name, pkg.version, pkg.edition
        );
        for target in &pkg.targets {
            println!(
                "    name: {}, kind: {:?}, crate_types: {:?}",
                target.name, target.kind, target.crate_types
            );
        }
        if pkg.name != root_package.name {
            for dep in &pkg.dependencies {
                // println!("        dep {}: {:?}", dep.name, dep);
                if all_other_deps.get(&dep.name).is_none() {
                    all_other_deps.insert(dep.name.to_owned());
                }
            }
        }
    }

    let dup_crates = root_deps.intersection(&all_other_deps);

    println!("all_crates: {:?}", all_other_deps);
    println!("root_deps: {:?}", root_deps);
    println!("dup_crates: {:?}", dup_crates);

    let mut maybe_unused = HashSet::new();
    for c in dup_crates {
        if let Ok(true) = find_usage(&c) {
            continue;
        } else {
            maybe_unused.insert(c);
        }
    }
    println!("crates {:?} maybe unused", maybe_unused);
}

fn find_usage(c: &str) -> Result<bool> {
    let pkg_name = if c.contains('-') {
        c.replace('-', "_")
    } else {
        c.to_string()
    };

    let use_output = Command::new("rg")
        .arg("--fixed-strings")
        .arg(format!("use {}", pkg_name))
        .output()?;

    let direct_use_output = Command::new("rg")
        .arg("--fixed-strings")
        .arg(format!("{}::", pkg_name))
        .output()?;

    let ok = use_output.status.success() || direct_use_output.status.success();

    Ok(ok)
}
