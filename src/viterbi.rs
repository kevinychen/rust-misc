use home;
use ordered_float::OrderedFloat;
use std::collections::{HashMap, HashSet};
use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Tag {
    tag: String,
}

#[derive(Debug)]
struct TaggedWord {
    word: String,
    tag: Tag,
}

#[derive(Debug)]
struct TaggedSentence {
    words: Vec<TaggedWord>,
}

#[derive(Debug)]
struct TaggedCorpus {
    sentences: Vec<TaggedSentence>,
}

struct HMM<'a> {
    all_tags: HashSet<&'a Tag>,
    observation_likelihoods: HashMap<(&'a Tag, &'a str), f64>,
    transition_probabilities: HashMap<(&'a Tag, &'a Tag), f64>,
}

impl TaggedWord {
    fn new(word: &str) -> TaggedWord {
        let mut it = word.split('/');
        TaggedWord {
            word: it.next().unwrap().to_string(),
            tag: Tag {
                tag: it.next().unwrap().to_string(),
            },
        }
    }
}

impl TaggedSentence {
    fn new(line: &str) -> TaggedSentence {
        let words = line
            .split_ascii_whitespace()
            .map(|token| TaggedWord::new(&token))
            .collect();
        TaggedSentence { words }
    }
}

impl TaggedCorpus {
    fn get_hmm(&self) -> HMM {
        // let mut all_words: HashSet<&str> = HashSet::new();
        let mut all_tags = HashSet::new();
        let mut words_per_tag = HashMap::new();
        let mut next_tags_per_tag = HashMap::new();
        let mut observation_likelihoods: HashMap<(&Tag, &str), f64> = HashMap::new();
        let mut transition_probabilities: HashMap<(&Tag, &Tag), f64> = HashMap::new();
        for sentence in self.sentences.iter() {
            for word in sentence.words.iter() {
                *observation_likelihoods
                    .entry((&word.tag, &word.word))
                    .or_insert(0.) += 1.;
                words_per_tag
                    .entry(&word.tag)
                    .or_insert_with(HashSet::new)
                    .insert(&word.word);
                all_tags.insert(&word.tag);
            }
            for (word1, word2) in sentence.words.iter().zip(sentence.words.iter().skip(1)) {
                *transition_probabilities
                    .entry((&word1.tag, &word2.tag))
                    .or_insert(0.) += 1.;
                next_tags_per_tag
                    .entry(&word1.tag)
                    .or_insert_with(HashSet::new)
                    .insert(&word2.tag);
            }
        }
        for (tag, words) in words_per_tag.iter() {
            let sum: f64 = words
                .iter()
                .map(|word| observation_likelihoods.get(&(*tag, *word)).unwrap())
                .sum();
            for word in words {
                *observation_likelihoods.get_mut(&(*tag, *word)).unwrap() /= sum;
            }
        }
        for (tag, next_tags) in next_tags_per_tag.iter() {
            let sum: f64 = next_tags
                .iter()
                .map(|next_tag| transition_probabilities.get(&(*tag, *next_tag)).unwrap())
                .sum();
            for next_tag in next_tags {
                *transition_probabilities
                    .get_mut(&(*tag, *next_tag))
                    .unwrap() /= sum;
            }
        }
        HMM {
            all_tags,
            observation_likelihoods,
            transition_probabilities,
        }
    }
}

#[derive(Clone, Copy)]
struct ViterbiState {
    tag: usize,
    log_prob: f64,
    prev_tag: Option<usize>,
}

const NEG_INF: f64 = -1000.;

fn viterbi<'a>(words: &Vec<&str>, hmm: &HMM<'a>) -> Vec<&'a Tag> {
    if hmm.all_tags.is_empty() {
        panic!("No tags");
    }
    if words.is_empty() {
        return vec![];
    }
    let all_tags = Vec::from_iter(hmm.all_tags.iter().map(|tag| *tag));
    let mut dp = vec![vec![None; all_tags.len()]; words.len()];
    for tag in 0..all_tags.len() {
        dp[0][tag] = Some(ViterbiState {
            tag,
            log_prob: hmm
                .observation_likelihoods
                .get(&(all_tags[tag], words[0]))
                .map_or(NEG_INF, |prob| prob.ln()),
            prev_tag: None,
        });
    }
    let transition_probabilities: Vec<Vec<f64>> = all_tags
        .iter()
        .map(|prev_tag| {
            all_tags
                .iter()
                .map(|tag| {
                    hmm.transition_probabilities
                        .get(&(prev_tag, tag))
                        .map_or(NEG_INF, |prob| prob.ln())
                })
                .collect()
        })
        .collect();
    let observation_likelihoods: Vec<Vec<f64>> = all_tags
        .iter()
        .map(|tag| {
            words
                .into_iter()
                .map(|word| {
                    hmm.observation_likelihoods
                        .get(&(tag, word))
                        .map_or(NEG_INF, |prob| prob.ln())
                })
                .collect()
        })
        .collect();
    for word in 1..words.len() {
        for tag in 0..all_tags.len() {
            let mut state = ViterbiState {
                tag,
                log_prob: -f64::INFINITY,
                prev_tag: None,
            };
            for prev_tag in 0..all_tags.len() {
                let log_prob = dp[word - 1][prev_tag].unwrap().log_prob
                    + transition_probabilities[prev_tag][tag]
                    + observation_likelihoods[tag][word];
                if log_prob >= state.log_prob {
                    state.log_prob = log_prob;
                    state.prev_tag = Some(prev_tag);
                }
            }
            dp[word][tag] = Some(state);
        }
    }
    let mut state = (0..all_tags.len())
        .map(|tag| dp[words.len() - 1][tag].unwrap())
        .max_by_key(|state| OrderedFloat(state.log_prob))
        .unwrap();
    let mut tags = vec![all_tags[state.tag]];
    for i in (0..words.len() - 1).rev() {
        let prev_tag = state.prev_tag.unwrap();
        state = dp[i][prev_tag].unwrap();
        tags.push(all_tags[prev_tag]);
    }
    tags.reverse();
    tags
}

pub fn run() {
    // http://korpus.uib.no/icame/brown/bcm.html
    let mut training_corpus = TaggedCorpus { sentences: vec![] };
    let mut test_corpus = TaggedCorpus { sentences: vec![] };
    fs::read_dir(home::home_dir().unwrap().join("nltk_data/corpora/brown"))
        .unwrap()
        .filter_map(|path| path.ok())
        .filter(|path| {
            let filename = path.file_name();
            let filename = filename.to_str().unwrap();
            filename.chars().next().unwrap() == 'c' && filename.len() == 4
        })
        .for_each(|entry| {
            let corpus = if entry.file_name().to_str().unwrap()[3..4]
                .parse::<u32>()
                .unwrap()
                % 2
                == 1
            {
                &mut training_corpus
            } else {
                &mut test_corpus
            };
            BufReader::new(File::open(entry.path()).unwrap())
                .lines()
                .filter_map(|line| line.ok())
                .filter(|line| !line.is_empty())
                .map(|line| TaggedSentence::new(&line))
                .for_each(|tagged_sentence| corpus.sentences.push(tagged_sentence));
        });
    let hmm = training_corpus.get_hmm();
    println!("num tags: {}", hmm.all_tags.len());
    println!("total # sentences: {}", test_corpus.sentences.len());
    let mut correct = 0;
    let mut total = 0;
    for sentence in test_corpus.sentences.iter().take(50) {
        let untagged_words: Vec<&str> = sentence
            .words
            .iter()
            .map(|word| AsRef::as_ref(&word.word))
            .collect();
        println!("{:?}", &untagged_words);
        let tags = viterbi(&untagged_words, &hmm);
        for (word, tag) in sentence.words.iter().zip(tags.iter()) {
            if word.tag == **tag {
                correct += 1;
            }
            total += 1;
        }
        let guessed_tags: Vec<&str> = tags.iter().map(|tag| AsRef::as_ref(&tag.tag)).collect();
        let actual_tags: Vec<&str> = sentence
            .words
            .iter()
            .map(|word| AsRef::as_ref(&word.tag.tag))
            .collect();
        println!("{:?}", guessed_tags);
        println!("{:?}", actual_tags);
        println!("Total: {}/{}\n", correct, total);
    }
}
