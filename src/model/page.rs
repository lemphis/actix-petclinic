pub struct Page {
    total_pages: u64,
    has_previous: bool,
    has_next: bool,
    page_range: Vec<u64>,
}

impl Page {
    pub fn new(page: u64, total_count: u64) -> Self {
        let total_pages = total_count.div_ceil(5);
        let start_page = page / 5 * 5 + 1;
        let end_page = (start_page + 4).min(total_pages);
        Page {
            total_pages,
            has_previous: page > 1,
            has_next: page < total_pages,
            page_range: (start_page..=end_page).collect(),
        }
    }
}

impl Page {
    pub fn total_pages(&self) -> u64 {
        self.total_pages
    }

    pub fn has_previous(&self) -> bool {
        self.has_previous
    }

    pub fn has_next(&self) -> bool {
        self.has_next
    }

    pub fn page_range(&self) -> &Vec<u64> {
        &self.page_range
    }
}
