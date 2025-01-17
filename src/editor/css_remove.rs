use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CssRemover(String);

impl CssRemover {
    fn parse(&self) -> anyhow::Result<scraper::Selector> {
        let scrape_selector = scraper::Selector::parse(&self.0)
            .map_err(|err| anyhow::anyhow!("css remover ({}) parse error: {err:?}", self.0))?;
        Ok(scrape_selector)
    }

    pub fn is_valid(&self) -> anyhow::Result<()> {
        self.parse()?;
        Ok(())
    }

    pub fn apply(&self, html: &str) -> anyhow::Result<String> {
        let mut parsed_html = scraper::Html::parse_document(html);
        let selected = parsed_html
            .select(&self.parse()?)
            .map(|o| o.id())
            .collect::<Vec<_>>();
        for selected in selected {
            if let Some(mut selected_mut) = parsed_html.tree.get_mut(selected) {
                selected_mut.detach();
            }
        }
        let html = parsed_html.root_element().html();
        Ok(html)
    }
}

impl std::str::FromStr for CssRemover {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = Self(s.to_string());
        s.parse()?;
        Ok(s)
    }
}

#[test]
fn valid() {
    let s = CssRemover("body".to_string());
    let result = s.is_valid();
    println!("{result:?}");
    assert!(result.is_ok());
}

#[test]
#[should_panic = "parse error"]
fn invalid() {
    CssRemover(".".to_string()).is_valid().unwrap();
}

#[cfg(test)]
const EXAMPLE_HTML: &str =
    r#"<html><head></head><body><div class="a"><p>A</p></div><div class="b">B</div></body></html>"#;

#[test]
fn removes_tag() {
    let html = CssRemover("p".to_string()).apply(EXAMPLE_HTML).unwrap();
    assert_eq!(
        html,
        r#"<html><head></head><body><div class="a"></div><div class="b">B</div></body></html>"#
    );
}

#[test]
fn remove_not_found() {
    let html = CssRemover("span".to_string()).apply(EXAMPLE_HTML).unwrap();
    assert_eq!(html, EXAMPLE_HTML);
}

#[test]
fn multiple_selectors_work() {
    let html = CssRemover(".b, p".to_string()).apply(EXAMPLE_HTML).unwrap();
    assert_eq!(
        html,
        r#"<html><head></head><body><div class="a"></div></body></html>"#
    );
}

#[test]
fn multiple_selectors_inside_each_other_work() {
    let html = CssRemover("p, .a".to_string()).apply(EXAMPLE_HTML).unwrap();
    assert_eq!(
        html,
        r#"<html><head></head><body><div class="b">B</div></body></html>"#
    );

    let html = CssRemover(".a, p".to_string()).apply(EXAMPLE_HTML).unwrap();
    assert_eq!(
        html,
        r#"<html><head></head><body><div class="b">B</div></body></html>"#
    );
}

#[test]
fn multiple_hits_only_remove_exact() {
    let html = CssRemover(".a p".to_string()).apply(r#"<html><head></head><body><div class="a"><p>TEST</p></div><div class="b"><p>TEST</p></div></body></html>"#).unwrap();
    assert_eq!(
        html,
        r#"<html><head></head><body><div class="a"></div><div class="b"><p>TEST</p></div></body></html>"#
    );
}
