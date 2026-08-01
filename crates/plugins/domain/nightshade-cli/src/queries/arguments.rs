use crate::data::Arguments;
use clap::{Args, Parser};

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub fn parse<T: Args + Default>() -> Arguments<T> {
    Arguments::<T>::try_parse_from(std::env::args()).unwrap_or_else(|error| error.exit())
}

#[cfg(target_os = "android")]
pub fn parse<T: Args + Default>() -> Arguments<T> {
    let mut arguments: Vec<String> = std::env::args().collect();
    if arguments.is_empty() {
        arguments.push(String::from("nightshade"));
    }
    Arguments::<T>::try_parse_from(arguments).unwrap_or_default()
}

#[cfg(target_arch = "wasm32")]
pub fn parse<T: Args + Default>() -> Arguments<T> {
    Arguments::<T>::try_parse_from(query_arguments(&search_string())).unwrap_or_default()
}

#[cfg(target_arch = "wasm32")]
fn search_string() -> String {
    let global = js_sys::global();
    let key = wasm_bindgen::JsValue::from_str("location");
    let Ok(location) = js_sys::Reflect::get(&global, &key) else {
        return String::new();
    };
    let key = wasm_bindgen::JsValue::from_str("search");
    let Ok(search) = js_sys::Reflect::get(&location, &key) else {
        return String::new();
    };
    search.as_string().unwrap_or_default()
}

#[cfg(target_arch = "wasm32")]
fn query_arguments(search: &str) -> Vec<String> {
    let mut arguments = vec![String::from("nightshade")];
    for pair in search
        .trim_start_matches('?')
        .split('&')
        .filter(|pair| !pair.is_empty())
    {
        let (key, value) = match pair.split_once('=') {
            Some((key, value)) => (key, Some(value)),
            None => (pair, None),
        };
        arguments.push(format!("--{}", decode(key)));
        if let Some(value) = value {
            arguments.push(decode(value));
        }
    }
    arguments
}

#[cfg(target_arch = "wasm32")]
fn decode(text: &str) -> String {
    let spaced = text.replace('+', " ");
    js_sys::decode_uri_component(&spaced)
        .map(String::from)
        .unwrap_or(spaced)
}
