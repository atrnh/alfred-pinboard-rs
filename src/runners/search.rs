use super::*;
use alfred::{ItemBuilder, Item};

use rusty_pin::pinboard::SearchType;

pub fn run(cmd: SubCommand, config: Config, pinboard: Pinboard) {
    match cmd {
        SubCommand::Search {tags, title, url, query} => {
            let mut search_fields = vec![];
            if tags {
                search_fields.push(SearchType::TagOnly);
            }
            if title {
                search_fields.push(SearchType::TitleOnly);
            }
            if url {
                search_fields.push(SearchType::UrlOnly);
            }
            // If user is not asking explicitly for search fields, then search based on
            // configuration set by user
            if search_fields.is_empty() {
                if config.tag_only_search {
                    search_fields.push(SearchType::TagOnly);
                } else {
                    search_fields = vec![SearchType::TagOnly, SearchType::TitleOnly, SearchType::DescriptionOnly];
                }
            }

            process(query, &search_fields, config.pins_to_show, pinboard);
        },
        _ => unreachable!(),
    }
}

fn process(query: Vec<String>, search_fields: &[SearchType], pins_to_show: u8, pinboard: Pinboard) {
    let query = query.iter().map(|s| s.as_ref()).collect::<Vec<&str>>();
    match pinboard.search(&query, search_fields) {
        Err(e) => ::show_error_alfred(&e),
        Ok(r) => {
            let alfred_items = match r {
                None => vec![ItemBuilder::new("No bookmarks found!").icon_path("no_result.icns").into_item()],
                Some(pins) => pins.iter().take(pins_to_show as usize)
                    .map(|pin| {
                        ItemBuilder::new(pin.title.as_ref())
                            .arg(pin.url.as_ref())
                            .icon_path("bookmarks.icns")
                            .into_item()
                    }).collect::<Vec<Item>>(),
            };
            alfred::json::write_items(io::stdout(), alfred_items.as_ref());
        }
    }
}