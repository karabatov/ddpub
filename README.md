# DDPub

## Features

* When exporting, reports on outbound linked notes not marked for publishing.

## Configuration file format

The website configuration file uses the [TOML](https://toml.io/en/) format.

A sample `website.toml`:
```toml
language = "en-US"

title = "NorikiTech"
subtitle = "Chaotic Software Engineering™"

theme = "default"

time_offset = -3600 # seconds from UTC

posts_url_prefix = "posts"

[notes]
id_format = "\d{12}"
id_link_format = "§\d{12}" # Format in Markdown links: [Link](§202212011301), [[§202212011301]]

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
