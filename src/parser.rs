use html5ever::tendril::TendrilSink;
use html5ever::{local_name, namespace_url, ns, parse_fragment, ParseOpts, Parser, QualName};
use markup5ever_rcdom::{Handle, RcDom};

pub fn parse_snippet(snippet: &str) -> Vec<Handle> {
    parse(make_sub_parser(), snippet)
        // remove Document tag
        .children
        .borrow()
        .first()
        .unwrap()
        // Remove html tag
        .children
        .take()
}

pub fn parse_document(doc: &str) -> Handle {
    parse(make_doc_parser(), doc)
}

fn parse(parser: Parser<RcDom>, src: &str) -> Handle {
    let doc = parser.one(src);
    for err in &doc.errors {
        tracing::warn!("Error while parsing document. Continuing...; error={}", err);
    }
    doc.document
}

fn make_doc_parser() -> Parser<RcDom> {
    html5ever::parse_document(RcDom::default(), ParseOpts::default())
}

fn make_sub_parser() -> Parser<RcDom> {
    parse_fragment(
        RcDom::default(),
        ParseOpts::default(),
        QualName::new(None, ns!(html), local_name!("div")),
        vec![],
    )
}

// FIXME: This is a hack im not proud of, it should be improved int he future
pub fn parse_markdown(md: &str) -> Vec<Handle> {
    let mut opt = pulldown_cmark::Options::empty();
    opt.insert(pulldown_cmark::Options::ENABLE_TABLES);
    opt.insert(pulldown_cmark::Options::ENABLE_FOOTNOTES);
    opt.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);

    let parser = pulldown_cmark::Parser::new_ext(md, opt);
    let mut snippet = String::new();
    pulldown_cmark::html::push_html(&mut snippet, parser);

    parse_snippet(&snippet)
}
