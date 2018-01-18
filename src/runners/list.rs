use super::*;
use alfred::{Item, ItemBuilder};

pub fn run<'a>(cmd: SubCommand, config: Config, pinboard: Pinboard<'a>) {
    match cmd {
        SubCommand::List { tags, query } => process(config, pinboard, tags, query),
        _ => unreachable!(),
    }
}


// TODO: Do not show tags that are alread autocompleted or present in user's query
fn process<'a>(config: Config, pinboard: Pinboard<'a>, tags: bool, q: Option<String>) {
    if tags {
        // Search the tags using the last 'word' in 'q'
        let queries = q.unwrap_or(String::new());

        // Check if user has entered ';' which indicates they are providing a description.
        // So no need to search for tags!
        if queries.contains(';') {
            let pin_info = queries
                .splitn(2, ';')
                .map(|s| s.trim())
                .collect::<Vec<&str>>();
            let item = ItemBuilder::new("Hit Return to bookmark the page!")
                .icon_path("upload.png")
                .arg(queries.as_ref())
                .variable("tags", pin_info[0])
                .variable("description", pin_info[1])
                .into_item();
            ::write_to_alfred(vec![item], config);
            return;
        }

        let query_words: Vec<&str> = queries.split_whitespace().collect();

        let mut popular_tags = vec![];
        let mut alfred_items = vec![];

        let mut exec_counter = 1;
        // First try to get list of popular tags from Pinboard
        if config.suggest_tags {
            exec_counter = env::var("apr_execution_counter")
                .unwrap_or("1".to_string())
                .parse::<usize>()
                .unwrap_or(1);
            popular_tags = retrieve_popular_tags(&config, &pinboard, exec_counter);
        }

        match pinboard.search_list_of_tags(query_words.last().unwrap_or(&String::new().as_str())) {
            Err(e) => ::show_error_alfred(&e),
            Ok(results) => {
                alfred_items = match results {
                    None => {
                        assert!(!query_words.is_empty());
                        let last_query_word = *query_words.last().unwrap();
                        vec![
                            ItemBuilder::new(last_query_word)
                                .subtitle("NEW TAG")
                                .autocomplete(last_query_word)
                                .icon_path("tag.png")
                                .into_item(),
                        ]
                    }
                    Some(items) => {
                        let mut prev_tags: &str = "";
                        if query_words.len() > 1 {
                            // User has already searched for other tags, we should include those in the
                            // 'autocomplete' field of the AlfredItem
                            prev_tags = queries.get(0..queries.rfind(' ').unwrap() + 1).unwrap()
                        }
                        popular_tags
                            .iter()
                            .chain(items.into_iter().take(config.tags_to_show as usize))
                            .map(|tag| {
                                ItemBuilder::new(tag.0.as_ref())
                                    .subtitle(if tag.1 != 0 {
                                        tag.1.to_string()
                                    } else {
                                        String::from("Popular")
                                    })
                                    .autocomplete([prev_tags, &tag.0].concat())
                                    .valid(true)
                                    .arg(String::from(prev_tags) + &tag.0)
                                    .icon_path("tag.png")
                                    .into_item()
                            })
                            .collect::<Vec<Item>>()
                    }
                };
            }
        }
        ::write_to_alfred(alfred_items, config);
    } else {
        if q.is_some() && !q.unwrap().is_empty() {
            eprintln!("Ignoring search query, will spit out all bookmarks.")
        }
        let items = pinboard
            .list_bookmarks()
            .unwrap_or(vec![])
            .into_iter()
            .take(config.pins_to_show as usize)
            .map(|pin| {
                ItemBuilder::new(pin.title.as_ref())
                    .subtitle(pin.url.as_ref())
                    .arg(pin.url.as_ref())
                    .into_item()
            });
        ::write_to_alfred(items, config);
    }
}

/// Retrieves popular tags from a Web API call for first run and caches them for subsequent runs.
fn retrieve_popular_tags<'a>(config: &Config, pinboard: &Pinboard<'a>, exec_counter: usize) -> Vec<Tag> {
    use std::env;
    use std::fs;
    use std::io::{BufReader, BufWriter, BufRead};

    let ptags_fn = config.cache_dir().join("popular.tags.cache");
    let mut popular_tags = vec![];

    if exec_counter == 1 {
        eprintln!("Retrieving popular tags.");
        if let Ok(tab_info) = browser_info::get() {
            let tags = match pinboard.popular_tags(&tab_info.url) {
                Err(e) => vec![String::from("ERROR: fetching popular tags!")],
                Ok(tags) => tags,
            };
            fs::File::create(ptags_fn).and_then(|fp| {
                let mut writer = BufWriter::with_capacity(1024, fp);
                writer.write_all(&tags.join("\n").as_bytes())
            });
            popular_tags = tags.into_iter().map(|t| Tag(t, 0)).collect::<Vec<Tag>>();
        }
    } else {
        eprintln!("reading tags from cache file: {:?}", ptags_fn);
        let fp = fs::File::open(ptags_fn).unwrap();
        let reader = BufReader::with_capacity(1024, fp);
        popular_tags = reader.lines().map(|l| Tag(l.unwrap(), 0)).collect::<Vec<Tag>>();
    }
    popular_tags
}

pub struct MyItem<'a>(Item<'a>);
use std::iter::FromIterator;

//impl<'a> FromIterator<Pin> for MyItem<'a> {
//    fn from_iter(p: Pin) -> Self {
//        MyItem(alfred::ItemBuilder::new(p.title)
//            .subtitle(p.url.as_ref())
//            .into_item())
//    }
//}
