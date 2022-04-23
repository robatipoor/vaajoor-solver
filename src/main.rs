use clap::Parser;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

const WORLD_FILE_NAME: &str = "words.txt";
const URL_CHECK: &str = "https://www.vaajoor.com/api/check";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value_t = 1)]
    day: u32,
}

#[derive(Debug)]
struct Character {
    index: usize,
    color: Color,
    value: char,
}

#[derive(Debug, PartialEq, Eq)]
enum Color {
    Red,
    Yellow,
    Green,
}

impl TryFrom<char> for Color {
    type Error = String;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        use crate::Color::*;
        match value {
            'r' => Ok(Red),
            'g' => Ok(Green),
            'y' => Ok(Yellow),
            _ => Err("color not support !".to_string()),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct VaagoorResponse {
    #[serde(alias = "dictionaryError")]
    error: bool,
    #[serde(alias = "match")]
    matchs: [char; 5],
}

#[derive(Debug)]
struct Word {
    characters: [Character; 5],
}

impl Word {
    fn new(word: &str, matchs: [char; 5]) -> Word {
        Word {
            characters: matchs
                .into_iter()
                .enumerate()
                .zip(word.chars())
                .map(|((index, color), value)| Character {
                    index,
                    color: Color::try_from(color).unwrap(),
                    value,
                })
                .collect::<Vec<Character>>()
                .try_into()
                .unwrap(),
        }
    }

    fn is_solve(&self) -> bool {
        self.characters.iter().all(|c| c.color == Color::Green)
    }

    fn is_green(&self, index: usize) -> bool {
        self.characters
            .iter()
            .any(|c| c.color == Color::Green && c.index == index)
    }

    fn is_before_green(&self, charecter: char) -> bool {
        self.characters
            .iter()
            .any(|c| c.value == charecter && c.color == Color::Green)
    }
}

fn main() {
    let args = Args::parse();
    let result = read_words(WORLD_FILE_NAME)
        .and_then(|words| solve(words, &args.day.to_string()))
        .unwrap();
    println!("answer => {}", result)
}

fn solve(words: Vec<String>, day: &str) -> Result<String, Box<dyn std::error::Error + 'static>> {
    let rand_word = choose_rand_world(&words)?;
    let word = check(&rand_word, day).map(|resp| Word::new(&rand_word, resp.matchs))?;
    if word.is_solve() {
        Ok(rand_word)
    } else {
        solve(remove_items(words, word), day)
    }
}

fn remove_items(words: Vec<String>, word: Word) -> Vec<String> {
    fn decrease_list(words: Vec<String>, word: Word, index: usize) -> Vec<String> {
        if let Some(c) = word.characters.get(index) {
            match c.color {
                Color::Green => {
                    let words = words
                        .into_iter()
                        .filter(|w| w.chars().nth(c.index).unwrap() == c.value)
                        .collect();
                    decrease_list(words, word, index + 1)
                }
                Color::Yellow => {
                    let words = words
                        .into_iter()
                        .filter(|w| {
                            if w.contains(c.value) {
                                w.chars().nth(c.index).unwrap() != c.value
                                    && !word.is_green(c.index)
                            } else {
                                false
                            }
                        })
                        .collect();
                    decrease_list(words, word, index + 1)
                }
                Color::Red => {
                    let words = words
                        .into_iter()
                        .filter(|w| {
                            if w.contains(c.value) {
                                word.is_before_green(c.value)
                            } else {
                                true
                            }
                        })
                        .collect();
                    decrease_list(words, word, index + 1)
                }
            }
        } else {
            words
        }
    }
    decrease_list(words, word, 0)
}

fn choose_rand_world(words: &Vec<String>) -> Result<String, String> {
    words
        .choose(&mut rand::thread_rng())
        .map(String::to_string)
        .ok_or_else(|| "list is empty ".to_string())
}

fn read_words(path: &str) -> Result<Vec<String>, Box<dyn std::error::Error + 'static>> {
    Ok(std::fs::read_to_string(path)?
        .split('\n')
        .map(|w| w.trim().to_string())
        .collect::<Vec<String>>())
}

fn check(word: &str, day: &str) -> Result<VaagoorResponse, Box<dyn std::error::Error + 'static>> {
    reqwest::blocking::Client::new()
        .get(URL_CHECK)
        .query(&[("word", word), ("g", day)])
        .send()?
        .json()
        .map_err(|e| e.into())
}
