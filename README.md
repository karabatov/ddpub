# DDPub

## Features

* When exporting, reports on outbound linked notes not marked for publishing.

## Usage

### Check config

`website` is a directory containing `config.toml`.

```bash
$ ddpub check --config "~/notes/website"
```

### Serve from the local notes directory

```bash
$ ddpub serve --config "~/notes/website" --notes "~/notes" --port 33075
```

### Further development

Serving from an empty state and getting pushes is not essential to launching.

#### Start empty and expect a push

The `DDPUB_ADDRESS` environment variable is required when the `--config` parameter is missing.

Without the `DDPUB_TOKEN` environment variable the server cannot accept pushes.

```bash
$ DDPUB_TOKEN='ABCDEF' DDPUB_ADDRESS='ddpub.org' ddpub serve  --port 33075
```

#### Push a website to server

Performs a `/ddpub_push` POST request to a running DDPub server. Can use HTTP if required.

Both `DDPUB_TOKEN` environment variable and `address` variable in the config must match the values on the server, otherwise the push will be refused.

The server performs a diff and only requests the client to upload files it doesn't have, to conserve traffic. The diff checks for file names, modified dates and size.

```bash
$ DDPUB_TOKEN=ABCDEF ddpub push --config "~/notes/website" --notes "~/notes"
```

How to allow both running from `./notes` and not overwriting local bootstrapped notes?

## Configuration directory format

A website is a structured directory containing, at a minimum, the website configuration file `config.toml`. It may or may not be within the notes directory.

```bash
$ tree ~/notes/sites/website
website
└── config.toml
```

The website configuration file uses the [TOML](https://toml.io/en/) format.

A sample `config.toml`:
```toml
address = "norikitech.com"

language = "en-US"

title = "NorikiTech"
subtitle = "Chaotic Software Engineering™"

theme = "default"

time_offset = -3600 # seconds from UTC

posts_url_prefix = "posts"

[notes]
id_format = '\d{12}'
id_link_format = '§\d{12}' # Format in Markdown links: [Link](§202212011301), [[§202212011301]]

[homepage]
id = "202212011301" # or: builtin = "posts"

# Website menu entries

[[menu]]
builtin = "homepage"
title = "Home"

[[menu]]
builtin = "posts"
title = "Posts"

[[menu]]
id = "202212020102" # About

[[menu]]
title = "Mastodon"
url = "https://dat-a.com/@ykar"

[[menu]]
builtin = "spacer"

[[menu]]
builtin = "search"

# Tags for posts and previews

[publish_tags]
posts = "norikitech_posts"
preview = "norikitech_preview" # not linked, found under /preview/<id>
publish = "norikitech_publish" # not on feed, public if linked from posts

# Any tags not present in [[tags]] are stripped

[[tags]]
tag = "internal_tag"
published = "External Tag"
page_id = "202212030303"

```
