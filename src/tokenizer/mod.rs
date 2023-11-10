use std::{fs::File, io::{BufWriter, Write}, collections::VecDeque, error::Error, sync::{Arc}, path::PathBuf, process::Stdio};

use tokio::{sync::{Mutex, broadcast::Sender}, process::{Command, Child}, io::{AsyncWriteExt, AsyncReadExt, AsyncBufRead, BufReader, AsyncBufReadExt}};
use serde::Deserialize;

static MAIN_PY: &str = 
r#"
import json
from kiwipiepy import Kiwi
kiwi = Kiwi()

def main():
    while True:
        try:
            sentence = input()
            result = []
            for token in kiwi.tokenize(sentence):
                if token.tag.startswith('NN') or token.tag == 'SL':
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
    signal: Sender<()>,
    ready: VecDeque<Child>,
}
pub struct Tokenizer {
    queues: Arc<Mutex<ModuleQueue>>,
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

        let (tx, _) = tokio::sync::broadcast::channel(1);

        Ok(Tokenizer {
            queues: Arc::new(Mutex::new(ModuleQueue { signal: tx, ready })),
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
                    let _ = queues.signal.send(());
                    drop(queues);

                    let token: Token = serde_json::from_str(&output)?;
                    
                    return Ok(token);
                },
                None => {
                    let mut rx = queues.signal.subscribe();
                    drop(queues);
                    rx.recv().await?;
                }
            }
        }
    }
}
