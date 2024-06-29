# helium
Fast static site generator

## Overview
- Fast rendering
- Effective static searching
- Overcome slow fetching speed of github pages
- Obsidian as your markdown text editor

Markdown and binary files(images) are linked in the way used by Obsidian program.

## Documentation
### Dependencies
#### required
- python, pip
    - kiwipiepy must be pre-installed through pip

```
pip3 install kiwipiepy
```

#### optional
- node, npm
    - webpack
- rust, cargo
    - wasm-pack

These depndencies are needed when `./wasm` or `./template` is modified.
You can just use `./wasm/dist_default/` and `./template/dist_default/` for rendering your website.

### How to build
```
cargo build --release
```
Now the program is placed in `./target/release` directory.

### How to run
First of all, prepare `config.yaml` file.

```yaml
nodes:
  - /Users/tyler/workspace/vault/main/2. Areas/blog-static/About.md
  - /Users/tyler/workspace/vault/main/2. Areas/blog-static/TØP 의 새 앨범 발매!.md
  - /Users/tyler/workspace/vault/main/2. Areas/blog-static/TWENTYONEPILOTS.svg
  - /Users/tyler/workspace/vault/main/2. Areas/blog-static/not todo list.md
  - /Users/tyler/workspace/vault/main/2. Areas/blog
  - /Users/tyler/workspace/vault/main/2. Areas/ps
  - /Users/tyler/workspace/vault/main/2. Areas/project

target_base: /Users/tyler/workspace/projects/argnmp.github.io

open_file_limit: 256

render:
  template: ./template/dist/template/*.html
  profile: ./profile.yaml
  collect_documents: true
  static:
    - ./wasm/dist/static/
    - ./template/dist/static/
    - /Users/tyler/workspace/vault/main/2. Areas/blog-static/profile image.png
  list_size: 10
```

- `nodes`, specify the root directory of markdown files, or a single markdown file. You can specify multiple directories or files to merge them into one static site.
- `target_base`, specify the path of directory where the generated static site will be placed.
- `open_file_limit`, specify the number of open files used concurrently. This value must be bigger than `50`. You can check your os limit by `ulimit` command.
- `render.template`, specify the path of template files. You don't need to change if you are using the default templates.
- `render.profile`, specify the path of profile yaml file. You don't need to change if you are using the default value. 
- `render.collect_documents`, specify whether to place documents(markdown files) in one directory(`/post`).
- `render.static`, specify the path of static files. These directories or files copied to the `/static` in your static site. You don't need to change `./wasm/dist/static/`, `./template/dist/static/` if you are using the default value. 
- `list_size`, specify the number of list elements that are shown in one page.

In summary, you only need to change `nodes` and target_base. The third path of `static` is used to copy a profile image which path is specified in `profile.yaml` below.

Second, prepare `profile.yaml` file which path is placed in `render.profile` in `config.yaml` file.

```yaml
page_title: blog.argnmp
name: argnmp
image: /static/profile image.png
descriptions:
  - 컴퓨터공학 관련 주제를 주로 다룹니다.
  - argnmp@gmail.com
links:
  - [github, https://github.com/argnmp]
footer: © 2023. Taehyeon Kim All rights reserved.
```

These values are all optional. You can fill your own values.

In `image`, specify the absolute path in terms of the rendered static page. By specifying `/Users/tyler/workspace/vault/main/2. Areas/blog-static/profile image.png` in `render.static` in `config.yaml`, `profile image` is copied to `/static` directory of static site. So it is possible to use the image in that directory.

To generate static site, execute the command below.
```
./target/release/helium --config config.yaml
```

### How to use
Upload the rendered files under the directory specified in `target_base` to git repository to deploy your static site using github pages.
