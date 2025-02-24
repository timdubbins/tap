mod error_view;
pub mod finder_view;
pub mod fuzzy_dir;
pub mod library;

pub use self::{
    error_view::ErrorView,
    finder_view::FinderView,
    fuzzy_dir::FuzzyDir,
    library::{Library, LibraryEvent, LibraryFilter},
};

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

pub const ID: &str = "finder";

// A struct that performs fuzzy matching on a collection of items.
#[derive(Clone)]
pub struct Finder {
    // The query string used for fuzzy matching.
    pub query: String,
    // The total count of items that match the fuzzy query.
    pub matches: usize,
    // The collection of items to perform fuzzy matching on.
    pub library: Library,
}

impl Finder {
    pub fn new(library: Library) -> Self {
        Self {
            query: String::new(),
            matches: library.len(),
            library,
        }
    }

    pub fn update(&mut self, query: String) {
        self.query = query;
        self.perform_fuzzy_matching();
    }

    fn perform_fuzzy_matching(&mut self) {
        if self.query.is_empty() {
            for i in 0..self.library.len() {
                self.library[i].match_weight = 1;
                self.library[i].match_indices.clear();
            }
            self.matches = self.library.len();
        } else {
            let mut match_count = 0;
            let matcher = Box::new(SkimMatcherV2::default().ignore_case());
            for i in 0..self.library.len() {
                let dir = &mut self.library[i];
                if let Some((weight, indices)) = matcher.fuzzy_indices(&dir.name, &self.query) {
                    dir.match_weight = weight;
                    dir.match_indices = indices;
                    match_count += 1;
                } else {
                    dir.match_weight = 0;
                    dir.match_indices.clear();
                }
            }
            self.matches = match_count;
            self.library
                .sort_by(|a, b| b.match_weight.cmp(&a.match_weight));
        }
    }
}
