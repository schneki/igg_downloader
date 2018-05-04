use hyper::{Uri, Client};
use futures::future::Future;
use futures::Stream;
use tokio_core::reactor::Core;
use hyper_tls::HttpsConnector;
use hyper::header::Headers;
use hyper::header::{SetCookie, Cookie, Raw};

pub fn get_url_content(url: &str) -> String {
    let uri = url.parse::<Uri>().unwrap();
    let mut core = Core::new().unwrap();
    let client = Client::new(&core.handle());
    let work = client.get(uri).map_err(|_err| ()).and_then(|resp| {
        resp.body().concat2().map_err(|_err| ()).map(|chunk| {
            let v = chunk.to_vec();
            String::from_utf8_lossy(&v).to_string()
        })
    });
    core.run(work).unwrap()
}

pub fn get_url_content_https(url: &str) -> String {
    let uri = url.parse::<Uri>().unwrap();
    let mut core = Core::new().unwrap();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &core.handle()).unwrap())
        .build(&core.handle());
    let work = client.get(uri).map_err(|_err| ()).and_then(|resp| {
        resp.body().concat2().map_err(|_err| ()).map(|chunk| {
            let v = chunk.to_vec();
            String::from_utf8_lossy(&v).to_string()
        })
    });
    core.run(work).unwrap()
}

use std::fs::File;
use std::io::Write;

pub fn download_file(url: &str, name: &str) {
    let uri = url.parse::<Uri>().unwrap();
    let mut core = Core::new().unwrap();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &core.handle()).unwrap())
        .build(&core.handle());

    let mut f = File::create(&name).unwrap();

    let work = client.get(uri).and_then(|resp| {
        resp.body().for_each(|chunk| {
            f.write_all(&chunk).unwrap();
            Ok(())
        })
    });
    core.run(work).unwrap();
}

pub fn get_url_content_https_with_cookies(url: &str, cookie: Cookie) -> (Cookie, String) {
    let uri = url.parse::<Uri>().unwrap();
    let mut core = Core::new().unwrap();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &core.handle()).unwrap())
        .build(&core.handle());
    
    let mut req = ::hyper::client::Request::new(::hyper::Method::Get, uri);

    req.headers_mut().set(cookie);
    
    let mut jar = Cookie::new();

    let work = client.request(req).map_err(|_err| ()).and_then(|resp| {
        if let Some(&SetCookie(ref content)) = resp.headers().get() {
            for set_cookie in content {
                let c = ::cookie::Cookie::parse(set_cookie.clone()).unwrap();
                let name = String::from(c.name());
                let value = String::from(c.value());
                jar.append(name,value);
            }
        }
        resp.body().concat2().map_err(|_err| ()).map(|chunk| {
            let v = chunk.to_vec();
            (jar, String::from_utf8_lossy(&v).to_string())
        })
    });
    core.run(work).unwrap()
}

fn get_cookies(resp: &::hyper::client::Response) -> Cookie {
    let cookies: Vec<(String, String)> 
     = if let Some(&SetCookie (ref cookies)) = resp.headers().get() {
        cookies.iter().map(|c| {
            let prev = c.split_at(c.find("=").unwrap());
            let aft = prev.1.split_at(prev.1.find(";").unwrap())
                .0.split_at(1);

            (prev.0.into(),aft.1.into())
        }).collect()
    } else { Vec::new() };
    
    let mut cookie_header = Cookie::new();
    for c in cookies {
        cookie_header.append(c.0, c.1);
    }
    cookie_header
}
