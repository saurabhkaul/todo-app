use std::borrow::Cow;
use std::fmt;
use std::fmt::Display;
use std::hash::Hash;
use std::ops::AddAssign;

use fxhash::{FxHashMap, FxHashSet};
use itertools::{EitherOrBoth, Itertools};
use nom::{AsChar};

use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Index(u64);

impl Index {
    pub fn new(i: u64) -> Index {
        Index(i)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Description(String);


impl Description {
    pub fn new(s: &str) -> Description {
        Description(s.to_owned())
    }

    pub fn value(&self) -> &str {
        &self.0
    }

    pub fn from_strings(ss: Vec<&str>) -> Vec<Description> {
        ss.clone().into_iter().map(|s| Description::new(s)).collect()
    }
    pub fn get_words(&self) -> Vec<String> {
        self.0.split_whitespace().map(|s| s.to_owned()).collect()
    }
}

impl fmt::Display for Description {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\"{}\"", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tag(String);


impl Tag {
    pub fn new(s: &str) -> Tag {
        Tag(s.to_owned())
    }

    pub fn value(&self) -> &str {
        &self.0
    }

    pub fn from_strings(ss: Vec<&str>) -> Vec<Tag> {
        ss.clone().into_iter().map(|s| Tag::new(s)).collect()
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoItem {
    pub index: Index,
    pub description: Description,
    pub tags: Vec<Tag>,
    pub done: bool,
}


fn subsequences<T: Display>(s: &T) -> Vec<String> {
    let chars: Vec<char> = s.to_string().chars().collect();
    let n = chars.len();
    let mut result = Vec::new();

    // Generate all possible combinations
    for i in 0..(1 << n) {
        let mut subsequence = String::new();
        for j in 0..n {
            if (i & (1 << j)) != 0 {
                subsequence.push(chars[j]);
            }
        }
        result.push(subsequence);
    }

    result
}

impl TodoItem {
    pub fn new(index: Index, description: Description, tags: Vec<Tag>, done: bool) -> TodoItem {
        TodoItem {
            index,
            description,
            tags,
            done,
        }
    }
}

impl Display for TodoItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tag_string = self.tags.iter().map(|t| format!("{t}")).join("\t");
        write!(f, "{} {} {}", self.index, self.description, tag_string)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoList {
    top_index: Index,
    items: Vec<TodoItem>,
    desc_map: DescriptionMap,
    tag_map: TagMap,
}

impl TodoList {
    pub fn search_word_and_tag(&self, w: Option<SearchWord>, t: Option<Tag>) -> Vec<&TodoItem> {
        let word_match_set = w.and_then(|w| self.desc_map.inner().get(&w.0));
        let tag_match_set = t.and_then(|t| self.tag_map.inner().get(&t.0));

        let final_set: Option<FxHashSet<Index>> = match (word_match_set, tag_match_set) {
            (Some(w), Some(t)) => Some(w.union(t).cloned().collect()),
            (Some(w), None) => Some(w.clone()),
            (None, Some(t)) => Some(t.clone()),
            (None, None) => None,
        };

        final_set
            .map(|set| set.iter().filter_map(|i| self.get_not_done_item_by_index(i)).collect())
            .unwrap_or_default()
    }


    pub fn get_not_done_item_by_index(&self, idx: &Index) -> Option<&TodoItem> {
        if let Some(item) = self.items.get((idx.value()) as usize) {
            if !item.done {
                Some(item)
            } else { None }
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DescriptionMap(FxHashMap<String, FxHashSet<Index>>);

impl DescriptionMap {
    pub fn inner(&self) -> &FxHashMap<String, FxHashSet<Index>> {
        &self.0
    }
    pub fn generate_subsequences_and_upsert(&mut self, idx: Index, s: String) {
        let subsequences = subsequences(&s);
        subsequences.into_iter().for_each(|s| {
            let _ = self.0
                .entry(s.clone()).and_modify(|set| { set.insert(idx); })
                .or_insert(FxHashSet::default());
            self
                .0
                .entry(s)
                .and_modify(|set| { set.insert(idx); });
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagMap(FxHashMap<String, FxHashSet<Index>>);

impl TagMap {
    pub fn inner(&self) -> &FxHashMap<String, FxHashSet<Index>> {
        &self.0
    }
    pub fn generate_subsequences_and_upsert(&mut self, idx: Index, s: String) {
        let subsequences = subsequences(&s);
        subsequences.into_iter().for_each(|s| {
            let _ = self.0
                .entry(s.clone()).and_modify(|set| { set.insert(idx); })
                .or_insert(FxHashSet::default());
            self
                .0
                .entry(s)
                .and_modify(|set| { set.insert(idx); });
        })
    }
}


impl AddAssign<u64> for Index {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs
    }
}

impl TodoList {
    pub fn new() -> TodoList {
        TodoList {
            top_index: Index::new(0),
            items: vec![],
            desc_map: DescriptionMap(FxHashMap::default()),
            tag_map: TagMap(FxHashMap::default()),
        }
    }

    pub fn push(&mut self, description: Description, tags: Vec<Tag>) -> TodoItem {
        let item = TodoItem::new(self.top_index, description.clone(), tags.clone(), false);
        self.items.push(item.clone());
        let _ = description.get_words().into_iter().for_each(|s| {
            self.desc_map.generate_subsequences_and_upsert(self.top_index, s);
        });
        tags.into_iter().for_each(|s| {
            self.tag_map.generate_subsequences_and_upsert(self.top_index, s.0);
        });
        self.top_index += 1;
        item
    }

    pub fn done_with_index(&mut self, idx: Index) -> Option<Index> {
        if let Some(item) = self.items.get_mut((idx.value()) as usize) {
            item.done = true;
            Some(item.index)
        } else {
            //index out of bounds
            None
        }
    }

    pub fn search<'a>(&'a self, sp: SearchParams) -> Vec<Cow<'a, TodoItem>> {
        let words = sp.words;
        let tags = sp.tags;
        let mut final_result = Vec::new();

        for pair in words.iter().zip_longest(tags.iter()) {
            let result = match pair {
                EitherOrBoth::Both(word, tag) => self.search_word_and_tag(Some(word.to_owned()), Some(tag.to_owned())),
                EitherOrBoth::Left(word) => self.search_word_and_tag(Some(word.to_owned()), None),
                EitherOrBoth::Right(tag) => self.search_word_and_tag(None, Some(tag.to_owned())),
            };
            final_result.extend(result.into_iter().map(Cow::Borrowed));
        }
        final_result.sort_by_key(|s| std::cmp::Reverse(s.index.value()));
        final_result
    }
}
