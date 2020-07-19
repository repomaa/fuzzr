extern crate wasm_bindgen;
extern crate js_sys;
extern crate web_sys;
extern crate fuzzy_matcher;

use std::convert::TryFrom;
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
    (&-self.score, &self.index).partial_cmp(&(&-other.score, &other.index))
  }
}

impl Ord for SearchResultItem {
  fn cmp(&self, other: &Self) -> Ordering {
    (&-self.score, &self.index).cmp(&(&-other.score, &other.index))
  }
}

#[derive(Default)]
pub struct Options {
  surround_matches_with: Option<(String, String)>
}

impl TryFrom<&js_sys::Object> for Options {
  type Error = JsValue;

  fn try_from(object: &js_sys::Object) -> Result<Self, Self::Error> {
    let js_value = js_sys::Reflect::get(
        &object, &"surroundMatchesWith".into()
    )?;

    const ERROR: &str = "surroundMatchesWith must be an array of exactly two strings";

    let js_array: js_sys::Array = match js_value.is_undefined() {
      true => return Ok(Self::default()),
      false => match js_sys::Array::is_array(&js_value) {
        true => js_value.into(),
        false => return Err(ERROR.into()),
      }
    };

    let surround_matches_with = match js_array.length() {
      2 => {
        let begin = js_array.shift().as_string().ok_or(ERROR)?;
        let end = js_array.shift().as_string().ok_or(ERROR)?;
        Ok((begin, end))
      },
      _ => Err(ERROR)
    }?;

    Ok(Options { surround_matches_with: Some(surround_matches_with) })
  }

}

#[wasm_bindgen]
pub struct Fuzzr {
  items: Vec<String>,
  options: Options,
  matcher: SkimMatcherV2,
}

#[wasm_bindgen]
impl Fuzzr {
  #[wasm_bindgen(constructor)]
  pub fn new(js_items: &JsValue, options: Option<js_sys::Object>) -> Result<Fuzzr, JsValue> {
    let iterator = js_sys::try_iter(js_items)?.ok_or("Items must be iterable")?;

    let mut items = Vec::<String>::new();

    for item in iterator {
      let item = item?;
      items.push(item.as_string().ok_or("Items must be strings")?)
    }

    let options = options.map_or(Ok(Options::default()), |v| Options::try_from(&v))?;

    let instance = Fuzzr {
      items,
      options,
      matcher: SkimMatcherV2::default(),
    };

    Ok(instance)
  }

  fn format(&self, item: &str, indices: &Vec<usize>) -> Option<String> {
    let (pre, post) = self.options.surround_matches_with.as_ref()?;
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

    Some(result)
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
            formatted: self.format(&item.as_str(), &indices).unwrap_or(item.into())
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
