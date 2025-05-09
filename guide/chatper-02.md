# Byte Pair Encoding (BPE) Tokenization

Byte Pair Encoding (BPE) is a subword tokenization algorithm that has become widely popular in natural language processing, particularly in transformer-based models. It was originally introduced as a data compression technique but was later adapted for text tokenization.

## How BPE Works

BPE works by iteratively merging the most frequent pairs of characters or tokens in a text corpus. In this website [tiktokenizer](https://tiktokenizer.vercel.app/?model=gpt2) you can see the BPE tokenization process in action. Here's the step-by-step process:

1. **Initialization**: Start with a vocabulary of individual characters
2. **Frequency Counting**: Count the frequency of all adjacent character pairs
3. **Merging**: Merge the most frequent pair into a new token
4. **Iteration**: Repeat the process until reaching the desired vocabulary size

### Example

Let's walk through a simple example with the following text:
```
"low low low low low lower lowest"
```

1. **Initial Vocabulary**:
```
l, o, w, e, r, s, t, " ", "low", "er", "est"
```

2. **First Iteration**:
- Count pairs: "lo" appears 5 times, "ow" appears 5 times
- Merge "lo" into "lo"

3. **Second Iteration**:
- New pairs: "low" appears 5 times
- Merge "low" into a single token

4. **Final Vocabulary** might look like:
```
l, o, w, e, r, s, t, " ", "low", "er", "est", "lower", "lowest"
```

## Advantages of BPE

1. **Handles Unknown Words**: Can represent words not seen during training
2. **Balanced Vocabulary**: Creates a good balance between character and word-level representations
3. **Efficient**: Creates a compact vocabulary while maintaining semantic meaning
4. **Language Agnostic**: Works well across different languages


## Real-World Applications

BPE is used in many state-of-the-art language models:

1. **GPT Models**: Use BPE for tokenization
2. **BERT**: Uses WordPiece, a variant of BPE
3. **T5**: Uses SentencePiece, another BPE variant

## Best Practices

1. **Vocabulary Size**: Choose an appropriate vocabulary size based on your dataset
2. **Special Tokens**: Include special tokens for unknown words, padding, etc.
3. **Preprocessing**: Clean and normalize text before applying BPE
4. **Language-Specific**: Consider language-specific preprocessing for non-English text

## Common Challenges

1. **Subword Ambiguity**: Same subword might have different meanings
2. **Vocabulary Size**: Finding the optimal vocabulary size
3. **Memory Usage**: Large vocabularies can be memory-intensive
4. **Training Time**: BPE training can be time-consuming for large corpora

## Practical Implementation

In our implementation, we use the `tiktoken-rs` library, which provides Rust bindings for OpenAI's tiktoken tokenizer. Here's how we implement BPE tokenization in practice:

```rust
use tiktoken_rs::{r50k_base, CoreBPE};

// Encode text into tokens
fn encode_data(data: &str, bpe: &CoreBPE) -> Vec<u32> {
    bpe.encode_with_special_tokens(data).to_vec()
}

// Decode tokens back to text
fn decode_data(tokens: &[u32], bpe: &CoreBPE) -> String {
    bpe.decode(tokens.to_vec()).unwrap()
}
```

Our implementation demonstrates several key concepts:

1. **Token Generation**: We use the `r50k_base` tokenizer, which is the same vocabulary used by GPT-2
2. **Context Building**: We show how tokens can be used to build context windows
3. **Encoding/Decoding**: We implement both directions of the tokenization process
4. **Practical Usage**: We demonstrate how to process a text file token by token

### Dataset Preparation with GPTDataset

For training our language model, we've implemented a `GPTDataset` struct that handles the tokenization and preparation of text data:

```rust
pub struct GPTDataset {
    input_ids: Vec<Tensor>,
    target_ids: Vec<Tensor>,
}

impl GPTDataset {
    pub fn new(text: &str, max_length: usize, stride: usize) -> Result<Self> {
        let bpe = r50k_base()?;
        let token_ids = bpe.encode_with_special_tokens(text);

        let mut input_ids = Vec::new();
        let mut target_ids = Vec::new();

        for i in (0..token_ids.len() - max_length).step_by(stride) {
            let input_chunk: Vec<i64> = token_ids[i..i + max_length]
                .iter()
                .map(|&x| x as i64)
                .collect();
            let target_chunk: Vec<i64> = token_ids[i + 1..i + max_length + 1]
                .iter()
                .map(|&x| x as i64)
                .collect();

            input_ids.push(Tensor::from_slice(&input_chunk));
            target_ids.push(Tensor::from_slice(&target_chunk));
        }

        Ok(Self {
            input_ids,
            target_ids,
        })
    }
}
```

The `GPTDataset` implementation showcases several important aspects of working with tokenized text:

1. **Sliding Window**: Uses a sliding window approach with configurable stride to create training examples
2. **Input-Target Pairs**: Creates pairs of input and target sequences for next-token prediction
3. **Tensor Conversion**: Converts token IDs into PyTorch tensors for model training
4. **Memory Efficiency**: Processes text in chunks to handle large documents efficiently

This implementation allows us to:
- Process text files efficiently
- Build context windows for language modeling
- Understand the relationship between tokens and text
- See how BPE works in a real-world application
- Prepare data for training language models

## Conclusion

BPE provides an effective way to handle the vocabulary problem in NLP by breaking words into meaningful subword units. It strikes a good balance between character-level and word-level representations, making it suitable for various NLP tasks.
