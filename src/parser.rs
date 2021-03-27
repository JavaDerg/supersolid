use html5ever::tendril::TendrilSink;
use html5ever::{parse_fragment, ParseOpts, Parser, QualName, ns, namespace_url, local_name};
use markup5ever_rcdom::{Handle, RcDom};

fn parse_snippet(snippet: &str) -> Handle {
    make_sub_parser().one(snippet).document
}

fn parse_document(doc: &str) -> Handle {
    make_doc_parser().one(doc).document
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
