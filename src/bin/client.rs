use serde::Deserialize;
use std::io::{self, prelude::*};
use whatsong::UnwrapOrAbort;

#[derive(Deserialize)]
struct Response<'a> {
    url: &'a str,
}

trait ToU32 {
    fn to_u32(self) -> u32;
}

impl ToU32 for [u8; 4] {
    #[inline]
    fn to_u32(self) -> u32 {
        u32::from(self[0])
            + (u32::from(self[1]) << 8)
            + (u32::from(self[2]) << 16)
            + (u32::from(self[3]) << 24)
    }
}

fn read_data() -> Vec<u8> {
    let mut stdin = io::stdin();

    let mut size = [0u8; 4];
    stdin
        .read_exact(&mut size)
        .unwrap_or_abort("cannot read header");

    let mut buf = vec![0; size.to_u32() as usize];
    stdin
        .read_exact(&mut buf)
        .unwrap_or_abort("cannot read data");
    buf
}

fn find_address() -> String {
    let path = whatsong::get_port_file();
    std::fs::read_to_string(path)
        .unwrap_or_abort("no active whatsong daemon")
        .trim()
        .to_string()
}

fn post_youtube(response: Response, addr: &str) {
    attohttpc::post(format!("http://{}/youtube", addr))
        .json(&whatsong::Item {
            kind: whatsong::ItemKind::Youtube(response.url.to_string()),
            ts: whatsong::timestamp() as i64,
            version: 1,
        })
        .unwrap_or_abort("serializing youtube item")
        .send()
        .unwrap_or_abort("sending youtube item");
}

fn main() {
    // do this early to see if its running
    let address = find_address();

    let data = read_data();
    let response: Response = serde_json::from_slice(&data) //
        .unwrap_or_abort("cannot deserialize response from browser");

    // TODO support other sites
    post_youtube(response, &address);

    // used to signal everything went fine
    println!("okay");
}
