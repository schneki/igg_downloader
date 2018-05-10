use util;
use select::document::Document;
use select::predicate::{Attr, Name, Class};
use hyper::Uri;
use hyper::header::Cookie;

use std::sync::mpsc::Sender;
use gui::Status;


pub fn download(drive_url: &str, tx: &Sender<Status>) -> Result<String, String> {
    let (url, name, size) = try!(to_confirm_url(drive_url));
    util::download_file(&url, &name, size, &tx);
    Ok(name)
}

pub fn to_confirm_url(drive_url: &str) -> Result<(String, String, usize), String> {
    let (cookies, content) = util::get_url_content_https_with_cookies(drive_url, Cookie::new());
    let document = Document::from(content.as_str());
    let size: usize = match document.find(Class("uc-name-size")).next() {
        Some(size) => {
            let s = size.text();
            let s = s
                .split("(").collect::<Vec<&str>>()[1]
                .split(")").collect::<Vec<&str>>()[0];

            if s.contains("M") {
                let f: f32 = s.replace("M", "").replace(",", ".").parse().unwrap();
                f as usize
                
            }
            else if s.contains("G") {
                let f: f32 = s.replace("G", "").replace(",", ".").parse().unwrap();
                f as usize * 1000
            }
            else {
                0
            }
        }
        None => return Err("not found".into())
    };
    let name = document
        .find(Class("uc-name-size")).next().unwrap()
        .find(Name("a")).next().unwrap().text();

    let confirm_path = document
        .find(Attr("id", "uc-download-link")).next().unwrap()
        .attr("href").unwrap();


    let url: Uri = drive_url.parse().unwrap();
    
    let confirm_url = format!("https://{}{}", url.host().unwrap(), confirm_path);

    let (_, c) = util::get_url_content_https_with_cookies(&confirm_url, cookies);
    let final_url = c
        .split("A HREF=").collect::<Vec<&str>>()[1]
        .split(">here").collect::<Vec<&str>>()[0]
        .replace('"', "");

    Ok((final_url, name, size))

}
