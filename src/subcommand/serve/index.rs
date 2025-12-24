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
        let old_page_meta = self.page_metas.get(page_id).cloned();
        match old_page_meta {
            Some(old_page_meta) => {
                // remove old links from backlinks
                for linked_page_id in &old_page_meta.links {
                    if let Some(set) = self.backlinks.get_mut(linked_page_id) {
                        set.remove(page_id);
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
