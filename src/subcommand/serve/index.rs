pub struct Index {
    pub(crate) backlinks: std::collections::BTreeMap<
        crate::page_id::PageId,
        std::collections::BTreeSet<crate::page_id::PageId>,
    >,
    config: crate::config::Config,
    pub(crate) page_metas:
        std::collections::BTreeMap<crate::page_id::PageId, crate::page_meta::PageMeta>,
    pub(crate) page_titles:
        std::collections::BTreeMap<String, std::collections::BTreeSet<crate::page_id::PageId>>,
}

impl Index {
    pub fn new(config: crate::config::Config) -> anyhow::Result<Self> {
        let page_ids = crate::page_io::PageIo::read_page_ids(&config)?;

        let mut page_titles = std::collections::BTreeMap::new();
        let mut page_metas = std::collections::BTreeMap::new();
        for page_id in &page_ids {
            let page_meta = crate::page_io::PageIo::read_page_meta(&config, page_id)?;
            match page_meta.title.as_deref() {
                None => {
                    // do nothing
                }
                Some(title) => {
                    page_titles
                        .entry(title.to_owned())
                        .or_insert_with(std::collections::BTreeSet::new)
                        .insert(page_id.clone());
                }
            }
            page_metas.insert(page_id.clone(), page_meta);
        }

        let mut backlinks = std::collections::BTreeMap::new();
        for (page_id, page_meta) in &page_metas {
            for linked_page_id in &page_meta.links {
                backlinks
                    .entry(linked_page_id.clone())
                    .or_insert_with(std::collections::BTreeSet::new)
                    .insert(page_id.clone());
            }
        }

        Ok(Self {
            backlinks,
            config,
            page_metas,
            page_titles,
        })
    }

    pub fn remove(&mut self, page_id: &crate::page_id::PageId) {
        let old_page_meta = self.page_metas.remove(page_id);
        match old_page_meta {
            Some(old_page_meta) => {
                // remove old links from backlinks
                for linked_page_id in &old_page_meta.links {
                    if let Some(set) = self.backlinks.get_mut(linked_page_id) {
                        set.remove(page_id);
                    }
                }

                // remove from page_titles
                if let Some(title) = &old_page_meta.title {
                    if let Some(set) = self.page_titles.get_mut(title) {
                        set.remove(page_id);
                        if set.is_empty() {
                            self.page_titles.remove(title);
                        }
                    }
                }
            }
            None => {
                // do nothing
            }
        }
    }

    pub fn update(&mut self, page_id: &crate::page_id::PageId) -> anyhow::Result<()> {
        let new_page_meta = crate::page_io::PageIo::read_page_meta(&self.config, page_id)?;

        let old_page_meta = self.page_metas.get(page_id).cloned();
        match old_page_meta {
            Some(old_page_meta) => {
                // remove old links from backlinks
                for linked_page_id in &old_page_meta.links {
                    if let Some(set) = self.backlinks.get_mut(linked_page_id) {
                        set.remove(page_id);
                    }
                }

                // remove old title from page_titles
                match old_page_meta.title.as_deref() {
                    None => {
                        // do nothing
                    }
                    Some(old_title) => {
                        self.page_titles
                            .entry(old_title.to_owned())
                            .and_modify(|set| {
                                set.remove(page_id);
                            });

                        match self.page_titles.get(&old_title.to_owned()) {
                            Some(set) if set.is_empty() => {
                                self.page_titles.remove(&old_title.to_owned());
                            }
                            _ => {
                                // do nothing
                            }
                        }
                    }
                }
            }
            None => {
                // do nothing
            }
        }

        for linked_page_id in &new_page_meta.links {
            self.backlinks
                .entry(linked_page_id.clone())
                .or_insert_with(std::collections::BTreeSet::new)
                .insert(page_id.clone());
        }

        self.page_metas
            .insert(page_id.clone(), new_page_meta.clone());
        match new_page_meta.title.as_deref() {
            None => {
                // do nothing
            }
            Some(new_title) => {
                self.page_titles
                    .entry(new_title.to_owned())
                    .or_insert_with(std::collections::BTreeSet::new)
                    .insert(page_id.clone());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_new() -> anyhow::Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let data_dir = temp_dir.path().join("data");
        std::fs::create_dir_all(&data_dir)?;

        let page1_id = crate::page_id::PageId::from_str("20251224T000000Z")?;
        let page1_content = r#"
# Test Page 1

This is a test page.
"#;
        std::fs::write(
            data_dir.join(page1_id.to_string()).with_extension("md"),
            page1_content,
        )?;

        let page2_id = crate::page_id::PageId::from_str("20251224T000001Z")?;
        let page2_content = r#"---
# Test Page 2

Link to [[20251224T000000Z]].
"#;
        std::fs::write(
            data_dir.join(page2_id.to_string()).with_extension("md"),
            page2_content,
        )?;

        let config_content = format!(
            r#"{{
    "data_dir": "{}"
}}"#,
            data_dir.display()
        );
        let config = <crate::config::Config as FromStr>::from_str(&config_content)?;

        let index = Index::new(config)?;

        assert_eq!(
            index.page_metas,
            [
                (
                    page1_id.clone(),
                    crate::page_meta::PageMeta {
                        title: Some("Test Page 1".to_owned()),
                        links: std::collections::BTreeSet::new(),
                    },
                ),
                (
                    page2_id.clone(),
                    crate::page_meta::PageMeta {
                        title: Some("Test Page 2".to_owned()),
                        links: [page1_id.clone()]
                            .into_iter()
                            .collect::<std::collections::BTreeSet<_>>(),
                    }
                ),
            ]
            .into_iter()
            .collect::<std::collections::BTreeMap<_, _>>()
        );

        assert_eq!(
            index.page_titles,
            [
                (
                    "Test Page 1".to_owned(),
                    [page1_id.clone()]
                        .into_iter()
                        .collect::<std::collections::BTreeSet<_>>(),
                ),
                (
                    "Test Page 2".to_owned(),
                    [page2_id.clone()]
                        .into_iter()
                        .collect::<std::collections::BTreeSet<_>>(),
                )
            ]
            .into_iter()
            .collect::<std::collections::BTreeMap<_, _>>()
        );

        assert_eq!(
            index.backlinks,
            [(
                page1_id.clone(),
                [page2_id.clone()]
                    .into_iter()
                    .collect::<std::collections::BTreeSet<_>>(),
            )]
            .into_iter()
            .collect::<std::collections::BTreeMap<_, _>>()
        );

        Ok(())
    }

    #[test]
    fn test_remove() -> anyhow::Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let data_dir = temp_dir.path().join("data");
        std::fs::create_dir_all(&data_dir)?;

        let page1_id = crate::page_id::PageId::from_str("20251224T000000Z")?;
        let page1_content = r#"
# Test Page 1

This is a test page.
"#;
        std::fs::write(
            data_dir.join(page1_id.to_string()).with_extension("md"),
            page1_content,
        )?;

        let page2_id = crate::page_id::PageId::from_str("20251224T000001Z")?;
        let page2_content = r#"---
# Test Page 2

Link to [[20251224T000000Z]].
"#;
        std::fs::write(
            data_dir.join(page2_id.to_string()).with_extension("md"),
            page2_content,
        )?;

        let page3_id = crate::page_id::PageId::from_str("20251224T000002Z")?;
        let page3_content = r#"---
# Test Page 3

Link to [[20251224T000000Z]].
"#;
        std::fs::write(
            data_dir.join(page3_id.to_string()).with_extension("md"),
            page3_content,
        )?;

        let config_content = format!(
            r#"{{
    "data_dir": "{}"
}}"#,
            data_dir.display()
        );
        let config = <crate::config::Config as FromStr>::from_str(&config_content)?;

        let mut index = Index::new(config)?;

        // verify initial state
        assert_eq!(index.page_metas.len(), 3);
        assert!(index.page_titles.contains_key("Test Page 1"));
        assert!(index.page_titles.contains_key("Test Page 2"));
        assert!(index.page_titles.contains_key("Test Page 3"));
        assert!(
            // page1
            // page2 -> page1
            // page3 -> page1
            index
                .backlinks
                .get(&page1_id)
                .map(|it| it.len() == 2 && it.contains(&page2_id) && it.contains(&page3_id))
                .unwrap_or(false)
        );

        index.remove(&page2_id);

        assert_eq!(index.page_metas.len(), 2);
        assert!(!index.page_metas.contains_key(&page2_id));
        assert!(!index.page_titles.contains_key("Test Page 2"));
        assert!(
            index
                .backlinks
                .get(&page1_id)
                .map(|it| it.len() == 1 && it.contains(&page3_id))
                .unwrap_or(false)
        );

        index.remove(&page1_id);

        assert_eq!(index.page_metas.len(), 1);
        assert!(!index.page_metas.contains_key(&page1_id));
        assert!(!index.page_titles.contains_key("Test Page 1"));

        // removing non-existent page should not cause any issues
        let non_existent_id = crate::page_id::PageId::from_str("20251225T000000Z")?;
        index.remove(&non_existent_id);

        Ok(())
    }

    #[tokio::test]
    async fn test_update() -> anyhow::Result<()> {
        // TODO: Add test for Index::remove
        Ok(())
    }
}
