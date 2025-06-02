# Snix

<img width="1200" src="./assets/snix_rust.png" alt="Snix Logo">

<sub><sub>z</sub></sub><sub>z</sub>z

Snix is a extremely fast code snippets & boilerplate manager (soon!) for your terminal. Inspired by Nap, Create and manage notebooks, snippets, and boilerplates with ease. Snix is designed to be fast, efficient, and user-friendly text-user interface, making it the perfect tool for developers who want to streamline their workflow in terminal.

<br />

<p align="center">
<img width="1000" src="./assets/showcase.gif" alt="snix showcase" />
</p>

<br />

## Text-based User Interface
Launch the interactive interface:

```bash
snix
```

<img width="1000" src="./assets/tui.gif" alt="snix tui showcase"/>

<br />

## Command Line Interface

List notebooks/snippets:

```bash
# List entire tree of notebooks/snippets
snix list / ls

# List snippets in current notebook
snix list <notebook_name>

# List all notebooks
snix list notebooks
```

<img width="600" src="./assets/snix_list_types.gif" alt="snix list"/>

Search for snippets:

```bash
# Search for snippets globally
snix search <query>
# Supports fuzzy search
```

<img width="600" src="./assets/snix_search.gif" alt="snix search"/>

List your favorite snippets / Preview a snippet:

```bash
snix list favorites / fav

# Preview a snippet using the snippet name
snix show / view <snippet_name>

# Preview a snippet using snippet ID
snix show / view  <snippet_id>
```
<img width="600" src="./assets/snix_fav.gif" alt="snix favorites"/>

## Installation

Build from source:

```bash
git clone https://github.com/parazeeknova/snix.git
cd snix
cargo run
```
or using make:

```bash
cd snix
make install
```

## Screenshot

<br />

<p align="center">
  <img
    width="1000"
    alt="image"
    src="./assets/showcase.png"
  />
</p>

<br />

## License

[MIT](https://github.com/parazeeknova/snix/blob/master/LICENSE)

## Feedback

I'd love to hear your feedback on improving `snix`.

Feel free to reach out via:
* [Twitter](https://x.com/hashcodes_)
* [GitHub issues](https://github.com/parazeeknova/snix/issues/new)

---

<sub><sub>z</sub></sub><sub>z</sub>z
