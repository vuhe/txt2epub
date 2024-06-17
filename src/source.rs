use std::borrow::Cow;
use std::ffi::OsStr;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::{Parser, ValueEnum};
use epub_builder::{EpubBuilder, EpubVersion, ZipCommand};
use imageinfo::ImageInfo;
use indoc::formatdoc;
use lazy_regex::{regex, regex_captures, Regex};
use rand::{thread_rng, Rng};

use crate::convert::ContentConverter;
use crate::error::EyreToAnyhow;
use crate::orly_cover::gen_orly_cover;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum TitleAlign {
    Left,
    Right,
    Center,
}

/// Covert txt novel to epub.
#[derive(Parser)]
#[command(version, author, about, long_about = None)]
pub struct Args {
    /// Input txt file path, must end with '.txt'
    #[arg(short, long)]
    filename: PathBuf,
    /// Book name, if not set try parse filename
    #[arg(long)]
    book_name: Option<String>,
    /// Book author, default None
    #[arg(long)]
    author: Option<String>,
    /// Cover image path, use Orly image if None
    #[arg(long)]
    cover: Option<PathBuf>,
    /// Color idx (0-16) if cover use orly image, default random color
    #[arg(value_parser = clap::value_parser!(u8).range(1..17))]
    orly_color: Option<u8>,
    /// Image idx (0-41) if cover use orly image, default random image
    #[arg(value_parser = clap::value_parser!(u8).range(0..=41))]
    orly_idx: Option<u8>,
    /// Epub language, default 'zh'
    #[arg(long)]
    lang: Option<String>,
    /// Volume match regex for match volume title
    #[arg(long)]
    vol_regex: Option<Regex>,
    /// Chapter match regex for match chapter title
    #[arg(long)]
    chap_regex: Option<Regex>,
    /// Title alignment, default center
    #[arg(long, value_enum)]
    align: Option<TitleAlign>,
    /// Paragraph indent, default 2 char
    #[arg(long)]
    indent: Option<usize>,
    /// Paragraph spacing (units can be em, px), default 1em
    #[arg(long)]
    bottom: Option<String>,
}

impl Args {
    pub fn parse_filename(mut self) -> Result<Self> {
        if self.filename.extension() != Some(OsStr::new("txt")) {
            bail!("file must end with .txt");
        }

        let filename = self.filename.file_name();
        let filename = filename
            .map(|it| it.to_string_lossy())
            .unwrap_or(Cow::Borrowed(""));
        let filename = filename.as_ref();

        if let Some((_, book, author)) = regex_captures!("《(.*)》.*作者[：:](.*).txt", filename)
        {
            if self.book_name.is_none() && !book.is_empty() {
                self.book_name = Some(book.into());
            }
            if self.author.is_none() && !author.is_empty() {
                self.author = Some(author.into());
            }
        }

        Ok(self)
    }

    fn book_name(&self) -> Cow<str> {
        if let Some(book_name) = self.book_name.as_deref() {
            return book_name.into();
        }
        self.filename.file_stem().unwrap().to_string_lossy()
    }

    fn author(&self) -> Option<&str> {
        self.author.as_deref()
    }

    fn lang(&self) -> &str {
        self.lang.as_deref().unwrap_or("zh")
    }

    fn cover(&self) -> Result<Vec<u8>> {
        match self.cover.as_ref() {
            Some(path) => Ok(std::fs::read(path).context("read cover file fail.")?),
            None => {
                let title = self.book_name();
                let author = self.author().unwrap_or("unknown");
                let orly_color = self
                    .orly_color
                    .unwrap_or_else(|| thread_rng().gen_range(1..17));
                let orly_idx = self
                    .orly_idx
                    .unwrap_or_else(|| thread_rng().gen_range(0..=41));
                gen_orly_cover(&title, author, orly_color, orly_idx)
            }
        }
    }

    fn css_style(&self) -> String {
        let align = match self.align {
            Some(TitleAlign::Left) => "left",
            Some(TitleAlign::Right) => "right",
            _ => "center",
        };
        let bottom = self.bottom.as_ref().map(String::as_str).unwrap_or("1em");
        let indent = self.indent.unwrap_or(2);
        formatdoc! {"
          h3 {{
            text-align: {align}
          }}
          p {{
            margin-bottom: {bottom};
            margin-top: 0;
            text-indent: {indent}em;
          }}"}
    }

    pub fn into_converter(self) -> Result<ContentConverter> {
        let zip_type = ZipCommand::new().into_anyhow()?;
        let mut epub_builder = EpubBuilder::new(zip_type).into_anyhow()?;

        epub_builder.set_title(self.book_name());
        epub_builder.set_lang(self.lang());
        if let Some(author) = self.author() {
            epub_builder.add_author(author);
        }

        let cover_bytes = self.cover().context("get cover fail.")?;
        let cover_info =
            ImageInfo::from_raw_data(&cover_bytes).context("parse cover info fail.")?;
        let css_file = self.css_style();
        let css_file = css_file.as_bytes();
        let css_path = Path::new("css").join("page.css");

        epub_builder
            .epub_version(EpubVersion::V20)
            .add_cover_image(
                format!("cover.{}", cover_info.ext),
                cover_bytes.as_slice(),
                cover_info.mimetype,
            )
            .into_anyhow()
            .context("add cover fail.")?
            .add_resource(css_path, css_file, "text/css")
            .into_anyhow()
            .context("add page css fail.")?;

        let vol_regex = match self.vol_regex {
            Some(it) => Cow::Owned(it),
            None => Cow::Borrowed(regex!("^第[0-9一二三四五六七八九十零〇百千两 ]+[卷部]").deref()),
        };
        let chap_regex =  match self.chap_regex {
            Some(it) => Cow::Owned(it),
            None => Cow::Borrowed(regex!(r"^第[0-9一二三四五六七八九十零〇百千两 ]+[章回节集卷部]|^[Ss]ection.{1,20}$|^[Cc]hapter.{1,20}$|^[Pp]age.{1,20}$|^\d{1,4}$|^\d+、|^引子$|^楔子$|^章节目录|^章节|^序章").deref()),
        };

        ContentConverter::new(&self.filename, epub_builder, vol_regex, chap_regex)
    }
}
