extern crate wasm_bindgen;
extern crate js_sys;
extern crate web_sys;
extern crate fuzzy_matcher;

use std::collections::BTreeSet;
use std::cmp::Ordering;
use wasm_bindgen::prelude::*;
use fuzzy_matcher::{
    FuzzyMatcher,
    skim::SkimMatcherV2,
};

pub struct SearchResultItem {
  item: String,
  index: usize,
  pub score: i64,
  formatted: String
}

impl PartialEq for SearchResultItem {
  fn eq(&self, other: &Self) -> bool {
    self.item == other.item
  }
}

impl Eq for SearchResultItem {}

impl PartialOrd for SearchResultItem {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    (&self.score, &self.index).partial_cmp(&(&other.score, &other.index))
  }
}

impl Ord for SearchResultItem {
  fn cmp(&self, other: &Self) -> Ordering {
    (&self.score, &self.index).cmp(&(&other.score, &other.index))
  }
}

pub struct FuzzrOptions {
  surround_matches_with: Option<(String, String)>
}

#[wasm_bindgen]
pub struct Fuzzr {
  items: Vec<String>,
  options: FuzzrOptions,
  matcher: SkimMatcherV2,
}

#[wasm_bindgen]
impl Fuzzr {
  pub fn matches(&self, string: &str, query: &str) -> bool {
    match self.matcher.fuzzy_match(string, query) {
      Some(_) => true,
      None => false
    }
  }

  #[wasm_bindgen(constructor)]
  pub fn new(js_items: js_sys::Array, options: Option<js_sys::Object>) -> Fuzzr {
    let mut items: Vec<String> = Vec::with_capacity(js_items.length() as usize);

    for item in &js_items.values() {
      let current_item = item.unwrap();
      items.push(current_item.as_string().unwrap())
    }

    let surround_matches_with = match &options {
      Some(object) => {
        match js_sys::Reflect::get(
          &object,
          &js_sys::JsString::from("surroundMatchesWith")
        ) {
          Ok(value) => {
            let array = js_sys::Array::from(&value);
            Some((
              array.shift().as_string().unwrap(),
              array.shift().as_string().unwrap()
            ))
          }
          Err(_) => None
        }
      },
      None => None
    };

    Fuzzr {
      items,
      options: FuzzrOptions {
        surround_matches_with
      },
      matcher: SkimMatcherV2::default(),
    }
  }

  fn format(&self, item: &str, indices: &Vec<usize>) -> String {
    match &self.options.surround_matches_with {
      Some((pre, post)) => {
        let mut result = String::with_capacity(item.len());
        let mut in_match = false;

        for (i, c) in item.chars().enumerate() {
          let matching = indices.contains(&i);

          if matching && !in_match {
            result.push_str(&pre);
          }

          if !matching && in_match {
            result.push_str(&post);
          }

          result.push(c);
          in_match = matching;
        }

        result
      },
      None => item.into()
    }
  }

  pub fn search(&mut self, query: String) -> js_sys::Array {
    let mut results: BTreeSet<SearchResultItem> = BTreeSet::new();

    for (index, item) in self.items.iter().enumerate() {
      match self.matcher.fuzzy_indices(item.as_str(), query.as_str()) {
        Some((score, indices)) => {
          results.insert(SearchResultItem {
            item: item.clone(),
            index: index,
            score,
            formatted: self.format(&item.as_str(), &indices)
          });
        },
        None => ()
      }
    }

    let js_array = js_sys::Array::new();

    for result in results {
      let item = js_sys::JsString::from(result.item);
      let index = js_sys::Number::from(result.index as u32);
      let score = js_sys::Number::from(result.score as u32);
      let formatted = js_sys::JsString::from(result.formatted);

      let object = js_sys::Object::new();
      js_sys::Reflect::set(&object, &js_sys::JsString::from("item"), &item).unwrap();
      js_sys::Reflect::set(&object, &js_sys::JsString::from("index"), &index).unwrap();
      js_sys::Reflect::set(&object, &js_sys::JsString::from("score"), &score).unwrap();
      js_sys::Reflect::set(&object, &js_sys::JsString::from("formatted"), &formatted).unwrap();

      js_array.push(&object);
    }

    js_array
  }
}
