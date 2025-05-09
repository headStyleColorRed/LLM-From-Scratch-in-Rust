use std::collections::HashMap;

pub struct NGramModel {
    unigram: HashMap<String, usize>,
    bigram: HashMap<(String, String), usize>,
    trigram: HashMap<(String, String, String), usize>,
    vocab_count: usize,
}

impl NGramModel {
    //──────────────────────────────────────────────────────────────────────────
    // Training: Building raw count tables
    //──────────────────────────────────────────────────────────────────────────
    pub fn train(text: &str) -> Self {
        // Tokenize the text, we are using a simple whitespace tokenizer for simplicity
        let tokens = Self::tokenize(text);
        // Initialize the hashmaps
        let mut unigram: HashMap<String, usize> = HashMap::new();
        let mut bigram: HashMap<(String, String), usize> = HashMap::new();
        let mut trigram: HashMap<(String, String, String), usize> = HashMap::new();

        // Walk through every token, keeping track of its index so we can refer to
        // the word that came immediately before it (needed for bigram counts).
        for (i, token) in tokens.iter().enumerate() {
            // ---------------------------- UNIGRAM ---------------------------------
            // Bump the occurrence count for the current word. If the word hasn't
            // been seen yet, `entry(...).or_insert(0)` inserts it with count 0,
            // then we increment it to 1.
            *unigram.entry(token.clone()).or_insert(0) += 1;

            // ----------------------------- BIGRAM ---------------------------------
            // Only once we're past the very first word do we have a valid "previous
            // token". At that point we form the pair (prev, current) and update
            // its frequency. This lets us answer questions like "after 'the mou'
            // how often does 'mountain' appear as the last word?"
            if i > 0 {
                let prev = tokens[i - 1].clone();
                *bigram.entry((prev, token.clone())).or_insert(0) += 1;
            }

            // ----------------------------- TRIGRAM ---------------------------------
            // Only once we're past the second word do we have a valid "previous
            // two words". At that point we form the pair (ante_prev, prev, current) and update
            // its frequency. This lets us answer questions like "in 'rock climbing mou'
            // how often does 'mountain' appear as the last word?"
            if i > 1 {
                let ante_prev = tokens[i - 2].clone();
                let prev = tokens[i - 1].clone();
                *trigram.entry((ante_prev, prev, token.clone())).or_insert(0) += 1;
            }
        }

        let vocab_count = unigram.keys().len();

        NGramModel {
            unigram,
            bigram,
            trigram,
            vocab_count
        }
    }

    //──────────────────────────────────────────────────────────────────────────
    // Tokeniser
    //──────────────────────────────────────────────────────────────────────────
    /// A simple whitespace tokenizer that also strips basic punctuation and lowercases tokens.
    fn tokenize(text: &str) -> Vec<String> {
        text.split_whitespace()
            .map(|t| {
                t.trim_matches(|c: char| !c.is_alphanumeric())
                    .to_lowercase()
            })
            .filter(|t| !t.is_empty())
            .collect()
    }

    //──────────────────────────────────────────────────────────────────────────
    // Laplace‑smoothed probability helpers
    //──────────────────────────────────────────────────────────────────────────
    /// Laplace smoothing for n-gram models
    ///
    fn smooth_with_laplace(&self, current: &str) -> f64 {
        // For bigram probability P(current|previous), we use:
        // (count(previous, current) + 1) / (count(previous) + V)
        // where V is the vocabulary size
        let mut total_probability = 0.0;

        for (prev_word, _) in &self.unigram {
            let bigram_count = self.bigram.get(&(prev_word.clone(), current.to_string())).unwrap_or(&0);
            let prev_word_count = self.unigram.get(prev_word).unwrap_or(&0);

            // Apply Laplace smoothing
            let smoothed_probability = ((*bigram_count as f64) + 1.0) /
                                    ((*prev_word_count as f64) + (self.vocab_count as f64));

            total_probability += smoothed_probability;
        }

        total_probability
    }
}

impl NGramModel {
    /// Suggest completions using unigram counts.
    pub fn suggest_unigram(&self, input: &str) -> (String, usize) {
        // 1. Lower‑case the input once so we don't repeat this inside the filter.
        let input: String = input
            .to_lowercase()
            .split_whitespace()
            .last()
            .unwrap_or("")
            .to_string();

        // 2‑4. Filter on prefix match, clone the key, copy the count; collect to Vec.
        let mut candidates: Vec<(String, usize)> = self
            .unigram
            .iter()
            .filter(|(word, _)| word.starts_with(&input))
            .map(|(word, count)| (word.clone(), *count))
            .collect();

        // 5. Sort by count descending
        candidates.sort_by(|a, b| b.1.cmp(&a.1));

        let best_candidate = candidates.first().cloned().unwrap_or((String::new(), 0));

        return best_candidate;
    }

    /// Suggest next word using bigram counts.
    pub fn suggest_bigram(&self, input: &str) -> (String, usize) {
        // 1. Tokenize input and ensure we have enough tokens
        let tokenized_input = Self::tokenize(input);

        if tokenized_input.len() < 1 {
            return (String::new(), 0);
        };

        // Extract current word to use as context
        let current_word = tokenized_input.last().unwrap();

        let mut candidates: Vec<(String, f64)> = Vec::new();

        // For each word in vocabulary, calculate its probability given the current word
        for word in self.unigram.keys() {
            let probability = self.smooth_with_laplace(word);
            candidates.push((word.clone(), probability));
        }

        // Sort by probability descending
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Convert probability back to a count-like number by scaling
        let best_candidate = candidates.first()
            .map(|(word, prob)| (word.clone(), (prob * 1000.0) as usize))
            .unwrap_or((String::new(), 0));

        return best_candidate;
    }

    /// Suggest next word using trigram counts
    pub fn suggest_trigram(&self, input: &str) -> (String, usize) {
        // 1. Tokenize input and ensure we have enough tokens
        let tokenized_input = Self::tokenize(input);

        if tokenized_input.len() < 2 {
            return (String::new(), 0);
        };

        // 2. Extract current and two previous words
        let current: String = tokenized_input[tokenized_input.len() - 1].clone();
        let previous: String = tokenized_input[tokenized_input.len() - 2].clone();

        // 3‑5. Filter on exact previous matches and current prefix, map to (word, count)
        let mut candidates: Vec<(String, usize)> = self
            .trigram
            .iter()
            .filter(|((ante_prev_word, prev_word, _), _)| {
                ante_prev_word == &previous && prev_word == &current
            })
            .map(|((_, _, current), count)| (current.clone(), *count))
            .collect();

        // 6. Sort by count descending and take the best match
        candidates.sort_by(|a, b| b.1.cmp(&a.1));

        let best_candidate = candidates.first().cloned().unwrap_or((String::new(), 0));

        return best_candidate;
    }
}
