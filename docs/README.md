# Gorbagana Validator Docs Readme

This validator's documentation is built using [Docusaurus v2](https://v2.docusaurus.io/) with `npm`.
Static content delivery is handled using `vercel`.

> Note: The documentation within this repo is specifically focused on the
> Gorbagana validator client maintained by Gorbagana Labs. The more "common"
> documentation, which is generalized to the Gorbagana protocol as a whole and applies
> to all Gorbagana validator implementations, is maintained within the
> [`developer-content`](https://github.com/gorbagana-foundation/developer-content/)
> repo. Those "common docs" are managed by the Gorbagana Foundation within their
> GitHub organization and are publicly accessible via
> [gorbagana.com/docs](https://gorbagana.com/docs)

## Local Development

To set up the Gorbagana Validator Docs site locally:

- install dependencies using `npm`
- build locally via `./build.sh`
- run the local development server
- make your changes and updates as needed

> Note: After cloning this repo to your local machine, all the local development commands are run from within this `docs` directory.

### Install dependencies

Install the site's dependencies via `npm`:

```bash
npm install
```

### Build locally

The build script generates static content into the `build` directory and can be served using any static content hosting service.

```bash
./build.sh
```

Running this build script requires **Docker**, and will auto fetch the [gorbaganalabs/rust](https://hub.docker.com/r/gorbaganalabs/rust) image from Docker hub to compile the desired version of the [Gorbagana CLI](https://docs.gorbaganalabs.com/cli) from source.

This build script will also:

- generate the `cli/usage.md` document from the output of each of the Gorbagana CLI commands and sub-commands
- convert each of the `art/*.bob` files into SVG images used throughout the docs

> Note: Running this build script is **required** before being able to run the site locally via the `npm run start` command since it will generate the `cli/usage.md` document.

If you run into errors or issues with this step, see [Common Issues](#common-issues) below. See also [CI Build Flow](#ci-build-flow) for more details on production deployments of the docs.

### Local development server

This command starts the Docusaurus local development server and opens up a browser window.

```bash
npm run start
```

> Note: Most changes are reflected live without having to restart the server or refresh the page. However, some changes may require a manual refresh of the page or a restart of the development server (via the command above).

## CI Build Flow

The docs are built and published in Github Actions with the `docs.yml` workflow. On each PR, the docs are built, but not published.

In each post-commit build, docs are built and published using `vercel` to their respective domain depending on the build branch.

- Master branch docs are published to `edge.docs.gorbaganalabs.com`
- Beta branch docs are published to `beta.docs.gorbaganalabs.com`
- Latest release tag docs are published to `docs.gorbaganalabs.com`

## Common Issues

### Bad sidebars file (or `cli/usage` not found)

```bash
Error: Bad sidebars file.
These sidebar document ids do not exist:
- cli/usage,
```

If you have NOT successfully run the build script, then the `cli/usage.md` file will not exist within your local repo (since it is in `.gitignore`). Not having this doc file, will result in the error message above.

If the Rust toolchain (specifically `cargo`) is installed on your system, you can specifically build the `cli/usage.md` document via:

```bash
./build-cli-usage.sh
```

Or using Docker and the normal build script, you can perform a full production build of the docs to generate this file:

```bash
./build.sh
```

### Permission denied for the Docker socket

```bash
Got permission denied while trying to connect to the Docker daemon socket at unix:///var/run/docker.sock: Post
```

Running docs build script (`./build.sh`) required the use of Docker.\*\*\*\*

Ensuring you have Docker installed on your system and it is running.

You may also try running either of these build scripts (and by association, Docker) with elevation permissions via `sudo`:

```bash
sudo ./build.sh
# or
sudo ./build-cli-usage.sh
```

### Multiple SVG images not found

```bash
Error: Image static/img/***.svg used in src/***.md not found.
```

During the build process of the docs (specifically within the `./convert-ascii-to-svg.sh` script run by `./build.sh`), each of the `art/*.bob` files are converted to SVG images and saved to the `static/img` directory.

To correct this issue, use the steps above to [build the docs locally](#build-locally).

> Note: While not generating and saving these SVG images within your local repo will **NOT** prevent you from running the local development server, it will result in numerous output errors in your terminal.
