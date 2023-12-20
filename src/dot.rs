//! Holds functions to render the Page Graph as a Dot graphiz format
use crate::{Link, Page, PageGraph, SpiderCrab};
use petgraph::dot::{Config, Dot};
use petgraph::graph::NodeIndex;

fn get_link_dot_attributes(
    _graph: &PageGraph,
    _edge_ref: petgraph::graph::EdgeReference<'_, Link>,
) -> String {
    "".to_string()
}

fn get_page_dot_attributes(_graph: &PageGraph, (_index, page): (NodeIndex, &Page)) -> String {
    let title: String = match (page.visited, page.title.clone()) {
        (false, _) => "???".to_string(),
        (true, None) => "NO TITLE".to_string(),
        (true, Some(t)) => t.trim().to_string(),
    };
    let color = match (page.visited, page.good) {
        (false, _) => "black",
        (true, Some(true)) => "green",
        (true, Some(false)) => "red",
        (true, None) => "orange",
    };

    format!(
        "label=\"{}\n{}\", color={}",
        title,
        page.url.as_str(),
        color
    )
}

impl SpiderCrab {
    pub fn get_dot_format(&self) -> String {
        format!(
            "{:?}",
            Dot::with_attr_getters(
                &self.graph,
                &[Config::EdgeNoLabel, Config::NodeNoLabel],
                &get_link_dot_attributes,
                &get_page_dot_attributes
            )
        )
    }
}
