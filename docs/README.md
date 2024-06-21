# /docs

Safehouse of information for the orbit project.

- `/drafts`: various notes, brainstorming, and random thoughts

- `/hdls`: language references, helpful paper resources

- `/pack`: documentation to provide in the deployed package

- `/papers`: in-depth ideas

- `/src`: orbit book pages for GitHub Pages through mdbook

- `/theme`: supportive mdbook coloring scheme themes for hdl code

The `manuals.toml` file contains the single source of truth regarding documentation for the various subcommands. This file is the input to a synchronization workflow to generate command docs for other places; see [mansync.py](./../tools/mansync.py).