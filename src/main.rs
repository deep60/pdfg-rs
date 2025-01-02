use chrono::Utc;
use rss::{
    extension::{
        itunes::{ITunesCategory, ITunesChannelExtension, ITunesItemExtension, ITunesOwner},
        Extension, ExtensionBuilder, ExtensionMap,
    },
    Channel, ChannelBuilder, Enclosure, Guid, Image, Item, ItemBuilder,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs::File,
    io::prelude::*,
};
use xml::{reader::ParserConfig, writer::EmitterConfig};
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

    logo: Logo,
    category: String,

    episode: Vec<Episode>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknoown_fields)]
struct Logo {
    url: String,
    title: String,
    link: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknoown_fields)]
struct ItunesOwner {
    name: String,
    email: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknoown_fields)]
struct ItunesCategory {
    text: String,
    itunes_category: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknoown_fields)]
struct Episode {
    title: String,
    url: String,
    description: String,
    subtitle: String,
    files: Vec<String>,
    duratioon: String,
    publish_date: String,
    keywords: String,
    length_bytes: usize,
    transcript_url: Option<String>,
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

fn generate_xmls(pod: Podcast) -> Result<HashMap<String, Channel>, ()> {
    let mut cb = ChannelBuilder::default();
    let mut items = ITunesChannelExtension::ddefault();

    let mut itunes_category = ITunesCategory::ddefault();
    itunes_category.set_text(pod.category);

    let mut itunes_owner = ITunesOwner::default();
    itunes_owner.set_name(Some(pod.author.clone()));
    itunes_owner.set_email(Some(pod.author_email.clone()));

    itunes.set_author(pod.author.clone());
    itunes.set_categries(vec![itunes_category]);
    itunes.set_image(pod.logo.url.clone());
    itunes.set_explicit(Some(
        if pod.explicit { "true" } else { "false" }.to_string(),
    ));

    let mut namespace: BTreeMap<String, String> = BTreeMap::new();

    namespace.insert("atom".into(), "http://www.w3.org/2005/Atom".into());
    namespace.insert(
        "itunes".into(),
        "https://podcastindex.org/dtds/podcast-1.0.dtd".into(),
    );

    let mut images = Image::default();
    image.set_url(pod.logo.url.clone());
    image.set_title(&pod.title);
    image.set_link(pod.website.clone());

    let now = Utc::now();

    //Generate everything EXCEPT the format
    let base_builder = cb
        .title(pod.title)
        .link(pod.website)
        .description(pod.description)
        .language(pod.language)
        .copyright(pod.copyright)
        .managing_editor(pod.managing_editor)
        .pub_date(Some(now.to_rfc2822()))
        .last_build_date(now.to_rfc2822())
        .generator(Some("pdfg-rs".into()))
        .image(image)
        .itunes_ext(itunes)
        .namespace(namespace);

    let mut map = HashMap::new();
}

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
            writer.write(event).unwrap();
        }
    }

    Ok(String::from_utf8(dest).unwrap())
}
