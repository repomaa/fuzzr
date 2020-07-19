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
  item: JsValue,
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
  surround_matches_with: Option<(String, String)>,
  to_string: Option<js_sys::Function>,
}

impl TryFrom<&js_sys::Object> for Options {
  type Error = JsValue;

  fn try_from(object: &js_sys::Object) -> Result<Self, Self::Error> {
    let surround_matches = js_sys::Reflect::get(
        &object, &"surroundMatchesWith".into()
    )?;
    let to_string = js_sys::Reflect::get(&object, &"toString".into())?;

    const SURROUND_MATCHES_ERROR: &str = "surroundMatchesWith must be an array of exactly two strings";

    let surround_matches: Option<js_sys::Array> = match surround_matches.is_undefined() {
      true => Ok(None),
      false => match js_sys::Array::is_array(&surround_matches) {
        true => Ok(Some(surround_matches.into())),
        false => Err(SURROUND_MATCHES_ERROR),
      }
    }?;

    let surround_matches_with = match surround_matches {
      Some(array) => match array.length() {
        2 => {
          let begin = array.shift().as_string().ok_or(SURROUND_MATCHES_ERROR)?;
          let end = array.shift().as_string().ok_or(SURROUND_MATCHES_ERROR)?;
          Ok(Some((begin, end)))
        },
        _ => Err(SURROUND_MATCHES_ERROR)
      },
      None => Ok(None)
    }?;

    const TO_STRING_ERROR: &str = "toString must a function";

    let to_string: Option<js_sys::Function> = match to_string.is_undefined() {
      true => Ok(None),
      false => match to_string.is_function() {
        true => Ok(Some(to_string.into())),
        false => Err(TO_STRING_ERROR),
      },
    }?;

    Ok(Options { surround_matches_with, to_string })
  }

}

#[wasm_bindgen]
pub struct Fuzzr {
  items: JsValue,
  options: Options,
  matcher: SkimMatcherV2,
}

#[wasm_bindgen]
impl Fuzzr {
  #[wasm_bindgen(constructor)]
  pub fn new(items: JsValue, options: Option<js_sys::Object>) -> Result<Fuzzr, JsValue> {
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

  pub fn search(&mut self, query: String) -> Result<js_sys::Array, JsValue> {
    let iterator = js_sys::try_iter(&self.items)?.ok_or("Items must be iterable")?;

    let mut results: BTreeSet<SearchResultItem> = BTreeSet::new();

    for (index, item) in iterator.into_iter().enumerate() {
      let item = item?;
      let text: String = match &self.options.to_string {
        Some(to_string) => to_string.call1(&JsValue::null(), &item)?
          .as_string()
          .ok_or("toString function must return a string"),
        None => item.as_string().ok_or("no toString function provided and item is not a string")
      }?;

      match self.matcher.fuzzy_indices(text.as_str(), query.as_str()) {
        Some((score, indices)) => {
          results.insert(SearchResultItem {
            item,
            index: index,
            score,
            formatted: self.format(&text.as_str(), &indices).unwrap_or(text.into())
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

    Ok(js_array)
  }
}
