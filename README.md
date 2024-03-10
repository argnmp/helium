## helium
Fast static site generator, using nlp for effective searching.

## Documentation
### Dependencies
- python
    - kiwipiepy installed through pip
- node.js

#### install npm packages
```shell
cd template
npm install
```

#### install python package
```
pip3 install kiwipiepy
```

To check whether the dependencies have been installed properly, the command below must be executed without problem in the root directory of this project.
```
python3 main.py
```

### How to build
```
cargo build --release
```
Now the program is placed in `./target/release` directory.

### How to run
First of all, prepare `config.yaml` file

```yaml
target:
  - /Users/tyler/Library/Mobile Documents/iCloud~md~obsidian/Documents/docuvault/2. Areas/ps
  - /Users/tyler/Library/Mobile Documents/iCloud~md~obsidian/Documents/docuvault/4. Archives/blog/daily
  - /Users/tyler/Library/Mobile Documents/iCloud~md~obsidian/Documents/docuvault/4. Archives/blog/computer science/
  - /Users/tyler/Library/Mobile Documents/iCloud~md~obsidian/Documents/docuvault/3. Resources/cpp
  - /Users/tyler/Library/Mobile Documents/iCloud~md~obsidian/Documents/docuvault/3. Resources/rust
  - /Users/tyler/Library/Mobile Documents/iCloud~md~obsidian/Documents/docuvault/3. Resources/linux
  - /Users/tyler/Library/Mobile Documents/iCloud~md~obsidian/Documents/docuvault/3. Resources/node
  - /Users/tyler/Library/Mobile Documents/iCloud~md~obsidian/Documents/docuvault/4. Archives/project
  # - /Users/tyler/Library/Mobile Documents/iCloud~md~obsidian/Documents/docuvault/4. Archives/blog/test

base:
  /Users/tyler/workspace/projects/argnmp.github.io
  # ./post

static:
  - ./template/dist
  - ./render/dist

template:
  ./template/src/*.html
```
In `target`, specify the root directory of markdown files. You can specify multiple directories to merge them into one static site.
In `base`, specify directory where the generated static site will be placed.

Do not need to modify `static` and `template`. `static` directories are copied to result. `template` specifies the location of html files that template engine uses.

To generate static site, execute the command below.
```
./target/release/helium --config config.yaml
```

### How to use
Files under the directory specified in `base` could be uploaded to git repository to deploy using github pages.
