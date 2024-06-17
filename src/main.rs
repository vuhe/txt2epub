mod content;
mod convert;
mod orly_cover;
mod source;
mod error;

fn main() -> anyhow::Result<()> {
    use clap::Parser;

    source::Args::try_parse()?
        .parse_filename()?
        .into_converter()?
        .covert_content_into_epub()
}
