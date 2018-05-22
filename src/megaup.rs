use util;
use select::document::Document;
use select::predicate::{Attr, Name, Class};

use gui::Status;
use std::sync::mpsc::Sender;
use hyper::header::Cookie;

pub fn download(megaup_url: &str, tx: &Sender<Status>) -> Result<String, String> {
    let (url, name, size, cookies) = try!(to_confirm_url(megaup_url));
    util::download_file_with_cookies(&url, &name, size, &tx, cookies);
    Ok(name)
}

fn to_confirm_url(megaup_url: &str) -> Result<(String, String, usize, Cookie), String> {
    let (mut cookies, content) = util::get_url_content_https_with_cookies(megaup_url, Cookie::new());
    let prev = "<a class='btn btn-default' href='";
    let prec = "'>download now</a>";
    
    let document = Document::from(content.as_str());

    
    let size: usize = match document.find(Class("responsiveInfoTable")).next() {
        Some(info) => { 
            let size = info.text();
            let s = size
                .split("Size: ").collect::<Vec<&str>>()[1]
                .split("</td>").collect::<Vec<&str>>()[0];

            if s.contains("MB") {
                let f: f32 = s.replace("MB", "").split(" ").collect::<Vec<&str>>()[0].parse().unwrap();
                f as usize
                
            }
            else if s.contains("GB") {
                let f: f32 = s.replace("GB", "").split(" ").collect::<Vec<&str>>()[0].parse().unwrap();
                f as usize * 1000
            }
            else {
                0
            } 
        },
        None => return Err("not found".into())
    };
    
    let name = document.find(Class("heading-1")).next().unwrap().text();

    let confirm_url = content
        .split(prev).collect::<Vec<&str>>()[1]
        .split(prec).collect::<Vec<&str>>()[0];

    let gtag = content
        .split("https://www.googletagmanager.com/gtag/js?id=").collect::<Vec<&str>>()[1]
        .split('"').collect::<Vec<&str>>()[0]
        .replace("-", "_");
    
    let gtag_cookie = format!("_gat_gtag_{}", gtag);

    cookies.append(gtag_cookie, "1");
    cookies.append("_ga", "GA1.2.1080630245.1526403795");
    cookies.append("_gid", "GA1.2.1088483899.1526403795");

    println!("{}", cookies);
    

   Ok((confirm_url.into(), name.into(), size, cookies))
}
