//! Holds functions to render the Page Graph as a Dot graphiz format
use petgraph::dot::{Dot, Config};
use petgraph::graph::NodeIndex;
use crate::{Link, Page, PageGraph, SpiderCrab};


fn get_link_dot_attributes<'a>(graph: &'a PageGraph, edge_ref: petgraph::graph::EdgeReference<'_, Link>) -> String {
    "".to_string()
}

fn get_page_dot_attributes<'a>(graph: &'a PageGraph, (index, page): (NodeIndex, &Page)) -> String {
    let title: String = match (page.checked, page.title.clone()) {
        (false, _) => "???".to_string(),
        (true, None) => "NO TITLE".to_string(),
        (true, Some(t)) => t.trim().to_string()
    };
    let color = match (page.checked, page.good) {
        (false, _) => "black",
        (true, Some(true)) => "green",
        (true, Some(false)) => "red",
        (true, None) => "orange"
    };

    return format!("label=\"{}\n{}\", color={}", title, page.url.as_str(), color);
}


impl SpiderCrab {
    pub fn get_dot_format(&self) -> String {
        format!("{:?}", Dot::with_attr_getters(&self.graph, &[Config::EdgeNoLabel, Config::NodeNoLabel], &get_link_dot_attributes, &get_page_dot_attributes))
    }
}