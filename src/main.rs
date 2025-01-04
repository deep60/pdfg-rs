use chrono::Utc;
use rss::{
    extension::{
        itunes::{self, ITunesCategory, ITunesChannelExtension, ITunesItemExtension, ITunesOwner},
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
    keywords: Vec<String>,
    explicit: bool,

    logo: Logo,
    category: String,

    episodes: Vec<Episode>,
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
    duration: String,
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

    itunes.set_owner(itunes_owner);
    itunes.set_subtitle(pod.subtitle);
    itunes.set_summary(pod.description.clone());
    itunes.set_keyword(pod.keywords.join(", "));

    let mut namespaces: BTreeMap<String, String> = BTreeMap::new();

    namespaces.insert("atom".into(), "http://www.w3.org/2005/Atom".into());
    namespaces.insert(
        "itunes".into(),
        "https://podcastindex.org/dtds/podcast-1.0.dtd".into(),
    );

    let mut image = Image::default();
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
        .namespace(namespaces);

    let mut map = HashMap::new();

    let mut item_map: HashMap<String, Vec<Item>> = HashMap::new();

    for episode in pod.episodes {
        let mut itunes_item = ITunesItemExtension::default();
        itunes_item.set_author(Some(pod.author.clone()));
        itunes_item.set_image(Some(pod.logo.url.clone()));
        itunes_item.set_duration(Some(episode.duration));
        itunes_item.set_explicit(Some(
            if pod.explicit { "true" } else { "false " }.to_string(),
        ));
        itunes_item.set_subtitle(episode.subtitle.clone());
        itunes_item.set_keywords(episode.keywords.join(", "));

        //Make "base" builder
        let mut base_item = ItemBuilder::default();

        base_item
            .title(episode.title.clone())
            .description(episode.description.clone())
            .author(pod.author_email.clone()) //email
            .pub_date(episode.publish_date.clone()) //RFC822
            .itunes_ext(itunes_item);

        let base_set: HashSet<_> = pod.formats.clone().drain(..).collect();

        let mut cur_set = base_set.clone();

        for file in episode.files {
            let ext = file.split('.').last().unwrap();
            let full_path = format!("{}/{}", pod.hosting_base_url, file);

            match (base_set.contains(ext), cur_set.contains(ext)) {
                (true, true) => {
                    let mut guid = Guid::default();
                    guid.set_value(full_path.clone());
                    guid.set_permalink(true);

                    let mut encl = Enclosure::default();
                    encl.set_url(full_path.clone());

                    let mine = match ext.to_lowercase().as_str() {
                        "mp3" => "audio/mpeg",
                        "m4a" => "audio/mp4",
                        "flac" => "audio/flac",
                        _ => "",
                    }
                    .to_string();

                    encl.set_mime_type(mime);
                    encl.set_length(episode.length_bytes.to_string());

                    let mut this_item = base_item.clone();
                    this_item.link(episode.url.clone());
                    this_item.enclosure(encl);

                    this_item.guid(Some(guid));
                    cur_set.remove(ext);

                    let mut item = this_item.build();

                    if let Some(transcript) = episode.transcript_url.clone() {
                        let xc_ext = transcript.split('.').last().unwrap();
                        let xcript_kind = match xc_ext {
                            "vtt" => "text/vtt",
                            "srt" => "application/srt",
                            "txt" => "text/plain",
                            _ => panic!("Unknown transcript extensin?"),
                        }
                        .to_string();

                        //Build an Extension....
                        let mut extension = ExtensionBuilder::ddefault();
                        extension.name("podcast:transcript");
                        let mut attrs = BTreeMap::new();
                        attrs.insert("url".to_string(), transcript);
                        attrs.insert("type".to_string(), xcript_kind);
                        extension.attrs(attrs);

                        let mut im = BTreeMap::<String, Vec<Extension>>::new();
                        im.insert("ext:name".to_string(), vec![extension.build()]);
                        let mut extension_map = ExtensionMap::default();
                        extension_map.insert("podcast".to_string(), im);

                        item.set_extensions(extension_map);
                    }

                    item_map.entry(ext.to_string()).or_default().push(item);
                }

                (true, false) => {
                    eprintln!("We've a file in the format '{}' for the episode '{}' has already been added. Skipping the file '{}' due to duplicate format.", ext, episode.title, file);
                }

                (false, _) => {
                    eprintln!("This podcast does not include '{}' among the available 'formats'! Skipping '{}' in the episode '{}'.", ext, file, episode.title);
                }
            }
        }
    }

    for (ext, items) in items_map.drain() {
        let mut this_builder = base_builder.clone();
        println!("{:#?}", items);
        this_builder.items(items);
        map.insert(ext.to_string(), this_builder.build());
    }

    Ok(map)
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
