use anyhow::Result;
use reqwest::blocking::Client;

static COLORS: [&str; 17] = [
    "61005e", "70706d", "890029", "c4000e", "6d001d", "6a00bd", "f10000", "0071b1",
    "f9bc00", "2c0077", "ba009a", "009047", "009d9e", "222e85", "bd002e", "009d1a",
    "75a500",
];

pub fn gen_orly_cover(title: &str, author: &str, color: u8, img: u8) -> Result<Vec<u8>> {
    let color = color as usize;
    let img_id = img.to_string();

    let bytes = Client::new()
        .get("https://orly.nanmu.me/api/generate")
        .query(&[
            ("title", title),
            ("author", author),
            ("g_loc", "BR"),
            ("top_text", "txt2epub-rs generated"),
            ("g_text", ""),
            ("img_id", img_id.as_str()),
            ("color", COLORS[color]),
        ])
        .send()?
        .bytes()?;

    Ok(bytes.to_vec())
}
