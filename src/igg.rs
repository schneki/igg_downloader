use select::document::Document;
use select::predicate::{Class, Name};

use util;

pub struct Game {
    name: String,
    url: String
}

pub fn search_game(game: &str) -> Vec<Game> {
    let url = format!("{}{}","http://igg-games.com/?s=", game);
    let content = util::get_url_content(&url);
    let document = Document::from(content.as_str());
    let mut games = Vec::new();
    for node in document.find(Class("post-details")) {
        let name = node.find(Name("h2")).next().unwrap().text();
        let url = node.find(Name("h2")).next().unwrap().find(Name("a")).next().unwrap().attr("href").unwrap();
        games.push(Game{name:name, url:url.into()});
    };
    games
}

pub fn get_megaup_urls(game_url: &str) -> Vec<String> {
    let content = util::get_url_content(game_url);
    let document = Document::from(content.as_str());
    decrypt_urls(document.find(Name("b"))
        .filter(|n| n.text() == "Link MegaUp.net:")
        .next().unwrap()
        .parent().unwrap()
        .find(Name("a")).map(|a| a.attr("href").unwrap().into())
        .collect())

}

pub fn get_drive_urls(game_url: &str) -> Vec<String> {
    let content = util::get_url_content(game_url);
    let document = Document::from(content.as_str());
    decrypt_urls(document
        .find(Name("b"))
        .filter(|n| n.text() == "Link Google Drive:")
        .next().unwrap()
        .parent().unwrap()
        .find(Name("a")).map(|a| a.attr("href").unwrap().into())
        .collect())
}


fn decrypt_urls(urls: Vec<String>) -> Vec<String> {
    let prefix = "?xurl=s://";
    let end = "&export=download";
    urls.into_iter().map(|url| {
        format!("https://{}", 
                url.split(prefix).collect::<Vec<&str>>()[1].split(end).collect::<Vec<&str>>()[0])
    }).collect()
}


