use std::{collections::{BTreeSet, VecDeque}, error::Error, fs::File, io::{BufWriter, Write}, process::Stdio, sync::Arc};

use tokio::{sync::Mutex, process::{Command, Child}, io::{AsyncWriteExt, BufReader, AsyncBufReadExt}};
use serde::Deserialize;

use tokio::sync::OnceCell;

static TOKENIZER: OnceCell<Tokenizer> = OnceCell::const_new();
pub async fn get_tokenizer() -> Result<&'static Tokenizer, Box<dyn std::error::Error + Send + Sync>> {
    Ok(TOKENIZER.get_or_init(|| async {
        
        Tokenizer::new(5).unwrap()
    }).await)
}

static MAIN_PY: &str = r#"
import json
from kiwipiepy import Kiwi
from kiwipiepy.utils import Stopwords
kiwi = Kiwi()
stopwords = Stopwords()

def main():
    while True:
        try:
            sentence = input()
            result = []
            for token in kiwi.tokenize(sentence, stopwords=stopwords):
                if token.tag.startswith('NNG') or token.tag == 'SL':
                    result.append(token.form)
            print(json.dumps({
                "data": result,
                }))
        except EOFError as e:
            exit(0)

if __name__ == "__main__":
    main()
"#;

#[derive(Deserialize, Debug)]
pub struct Token {
    pub data: Vec<String>,
}

pub struct ModuleQueue{
    ready: VecDeque<Child>,
}
pub struct Tokenizer {
    queues: Arc<Mutex<ModuleQueue>>,
    pub all_tokens: Arc<Mutex<BTreeSet<String>>>,
}
impl Drop for Tokenizer {
    fn drop(&mut self) {
        let queues = self.queues.clone();
        tokio::spawn(async move {
            let mut queues = queues.lock().await;  
            while let Some(mut module) = queues.ready.pop_front() {
                module.kill().await.unwrap();
            }
        });
    }
}
impl Tokenizer {
    pub fn new(n: usize) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let f = File::options().write(true).create(true).truncate(true).open("./main.py")?;
        let mut writer = BufWriter::new(f);
        writer.write_all(MAIN_PY.as_bytes())?;
        writer.flush()?;

        let mut ready = VecDeque::new();
        for _ in 0..n {
            ready.push_back(
                Command::new("python3")
                .args(["./main.py"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?
                );
        }

        Ok(Tokenizer {
            queues: Arc::new(Mutex::new(ModuleQueue { ready })),
            all_tokens: Arc::new(Mutex::new(BTreeSet::new())),
        })
    }

    pub async fn tokenize(&self, s: &str) -> Result<Token, Box<dyn Error + Send + Sync>> {
        loop {
            let mut queues = self.queues.lock().await; 
            match queues.ready.pop_front() {
                Some(mut module) => {
                    drop(queues);
                    match &mut module.stdin {
                        Some(input) => {
                            input.write_all(format!("{}\n", s).as_bytes()).await?;
                            input.flush().await?;
                        },
                        None => {
                            dbg!("no stdin");
                        }
                    }
                    let mut output = String::new();
                    match &mut module.stdout {
                        Some(out) => {
                            let mut reader = BufReader::new(out);
                            reader.read_line(&mut output).await?;
                        },
                        None => {
                            dbg!("no stdout");
                        }
                    }

                    let mut queues = self.queues.lock().await; 
                    queues.ready.push_back(module);
                    drop(queues);

                    let token: Token = serde_json::from_str(&output)?;
                    
                    /* let mut all_tokens = self.all_tokens.lock().await;
                    all_tokens.extend(token.data.clone()); */
                    
                    return Ok(token);
                },
                None => {
                    drop(queues);
                }
            }
        }
    }
}

