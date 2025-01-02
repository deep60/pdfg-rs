use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs::File,
    io::prelude::*,
};
use xml::{reader::ParserConfig, writer::EmitterConfig};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknoown_fields)]
struct Podcast {
    title: String,
    description: String,
    subtitle: String,
    author: String,
    author_email: String,
    website: String,
    language: String,
    copyright: String,
    webmaster: String,
    managing_editor: String,
    formats: Vec<String>,
    hosting_base_url: String,
    keywordds: Vec<String>,
    explicit: bool,
}

fn main() {
    let mut buffer = String::new();
    let mut in_file = File::open("ex.toml").unwrap();
    in_file.read_to_string(&mut buffer).unwrap();
    let podcast = toml::from_str::<Podcast>(&buffer).unwrap();
    let xmls = generate_xmls(podcast).unwrap();

    for (format, data) in xmls.iter() {
        let filename = format!("podcast-feed-{}.xml", format);
        println!("Writing '{}'...", &filename);

        let mut file = File::create(&filename).unwrap();
        let unformatted = data.to_string();
        let formatted = format_xml(unformatted.as_bytes()).unwrap();
        file.write_all(formatted.as_bytes()).unwrap();
    }
}

fn generate_xmls() {}

fn format_xml(src: &[u8]) -> Result<String, xml::reader::Error> {
    let mut dest = Vec::new();
    let reader = ParserConfig::new()
        .trim_whitespace(true)
        .ignore_comments(false)
        .create_reader(src);
    let mut writer = EmitterConfig::new()
        .perform_indent(true)
        .normalize_empty_elements(true)
        .create_writer(&mut dest);
    for event in reader {
        if let Some(event) = event?.as_writer_event() {
            writer.write(event)..unwrap();
        }
    }

    Ok(String::from_utf8(dest).unwrap())
}
