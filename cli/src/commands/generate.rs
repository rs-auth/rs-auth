use std::fs;
use std::path::{Path, PathBuf};

pub fn run(name: &str) -> anyhow::Result<()> {
    let migrations_dir = migrations_dir();
    fs::create_dir_all(&migrations_dir)?;

    let next_prefix = next_prefix(&migrations_dir)?;
    let file_name = format!("{next_prefix:03}_{}.sql", slugify(name));
    let path = migrations_dir.join(file_name);
    fs::write(&path, "-- Write your migration here\n")?;

    println!("Created {}", path.display());
    Ok(())
}

fn migrations_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .join("pg")
        .join("migrations")
}

fn next_prefix(dir: &Path) -> anyhow::Result<u32> {
    let mut max_prefix = 0;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let Some(file_name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        let Some(prefix) = file_name.split('_').next() else {
            continue;
        };
        let Ok(value) = prefix.parse::<u32>() else {
            continue;
        };
        max_prefix = max_prefix.max(value);
    }

    Ok(max_prefix + 1)
}

fn slugify(name: &str) -> String {
    let mut slug = String::new();
    let mut last_was_separator = false;

    for character in name.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            last_was_separator = false;
        } else if !last_was_separator {
            slug.push('_');
            last_was_separator = true;
        }
    }

    slug.trim_matches('_').to_string()
}

#[cfg(test)]
mod tests {
    use super::slugify;

    #[test]
    fn slugifies_names() {
        assert_eq!(slugify("Add User Sessions"), "add_user_sessions");
        assert_eq!(slugify("hello---world"), "hello_world");
    }
}
