// Some parts of this code are take from zola (https://github.com/getzola/zola).
// For example the regex and frontmatter parsing code. Hence this license/copyright notice.

/// The MIT License (MIT)

/// Copyright (c) 2017-2018 Vincent Prouillet

/// Permission is hereby granted, free of charge, to any person obtaining a copy
/// of this software and associated documentation files (the "Software"), to deal
/// in the Software without restriction, including without limitation the rights
/// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
/// copies of the Software, and to permit persons to whom the Software is
/// furnished to do so, subject to the following conditions:
///
/// The above copyright notice and this permission notice shall be included in all
/// copies or substantial portions of the Software.
///
/// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
/// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
/// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
/// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
/// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
/// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
/// SOFTWARE.

use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};
use regex::Regex;
use toml::Value;
use toml::value::Table;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug)]
pub(crate) struct Frontmatter {
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) image_path: PathBuf,
}

pub(crate) fn read_frontmatter(blog_root: &Path) -> Vec<Frontmatter> {
    let walker = WalkDir::new(blog_root).contents_first(true);
    let mut frontmatter_list = Vec::new();
    for entry in walker {
        match entry {
            Ok(entry) => {
                if is_md_file(&entry) {
                    let path = entry.path();
                    match create_frontmatter(path) {
                        Ok(frontmatter) => {
                            if let Some(frontmatter) = frontmatter {
                                frontmatter_list.push(frontmatter)
                            }
                        },
                        Err(e) => eprintln!("Error while creating frontmatter: {}", e),
                    }
                }
            }
            Err(e) => eprintln!("Failed to walk a directory entry. Error: {:?}", e)
        }
    }
    frontmatter_list
}

fn is_md_file(entry: &DirEntry) -> bool {
    entry.file_type().is_file() &&
        entry.file_name().to_str()
            .map(|s| s.to_lowercase().ends_with(".md"))
            .unwrap_or(false)
}

fn create_frontmatter(path: &Path) -> Result<Option<Frontmatter>, Cow<'static, str>> {
    let contents = fs::read_to_string(path).map_err(|e| format!("Error reading file: {}", e))?;
    let parsed_contents = parse_toml_frontmatter(&contents).map_err(|e| format!("Error parsing frontmatter: {}", e))?;
    let image_path = create_image_path(path).map_err(|e| format!("Error creating image path: {}", e))?;
    Ok(parsed_contents.map(|c| Frontmatter { title: c.0, description: c.1, image_path }))
}

fn create_image_path(path: &Path) -> Result<PathBuf, &'static str> {
    let image_path = path.parent().ok_or("Failed to find parent of path")?
        .join(&format!("{}.png", path.file_stem().expect("Failed to get file stem").to_str()
            .expect("Failed to convert from OsString")));
    Ok(image_path)
}

fn parse_toml_frontmatter(contents: &str) -> Result<Option<(String, String)>, Cow<'static, str>> {
    let toml_regex = Regex::new(r"^[[:space:]]*\+\+\+(\r?\n(?s).*?(?-s))\+\+\+[[:space:]]*(?:$|(?:\r?\n((?s).*(?-s))$))")
        .expect("Failed to parse toml regex");

    let captures = toml_regex.captures(contents).ok_or("Failed to find captures in TOML regex")?;
    let raw_frontmatter = captures.get(1).ok_or("Failed to get the first capture")?.as_str();
    let parsed_frontmatter = raw_frontmatter.parse::<Value>().map_err(|e| format!("Failed to parse frontmatter {}", e))?;
    let table = parsed_frontmatter.as_table().ok_or("Failed to get frontmatter table")?;
    if should_process_file(table) {
        let title = table.get("title").ok_or("Failed to get title")?
            .as_str().ok_or("Failed to convert title to string")?.to_string();
        let description = table.get("shortdesc").ok_or("Failed to get description")?
            .as_str().ok_or("Failed to convert description to string")?.to_string();
        Ok(Some((title, description)))
    } else {
        Ok(None)
    }
}

fn should_process_file(table: &Table) -> bool {
    !is_draft(table) && !is_blog_root(table)
}

fn is_blog_root(table: &Table) -> bool {
    match table.get("template") {
        Some(template) => {
            match template.as_str() {
                Some(template) => template == "blog.html",
                None => false,
            }
        }
        None => false,
    }
}

fn is_draft(table: &Table) -> bool {
    match table.get("draft") {
        Some(draft) => draft.as_bool().unwrap_or(true),
        None => false,
    }
}