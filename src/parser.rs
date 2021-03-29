use html5ever::tendril::TendrilSink;
use html5ever::{local_name, namespace_url, ns, parse_fragment, ParseOpts, Parser, QualName};
use markup5ever_rcdom::{Handle, RcDom};

pub fn parse_snippet(snippet: &str) -> Handle {
    parse(make_sub_parser(), snippet)
}

pub fn parse_document(doc: &str) -> Handle {
    parse(make_doc_parser(), doc)
}

fn parse(parser: Parser<RcDom>, doc: &str) -> Handle {
    let doc = parser.one(doc);
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
        QualName::new(None, ns!(), local_name!("div")),
        vec![],
    )
}
