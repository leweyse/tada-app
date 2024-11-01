# tada-app

Build your own templates, add your own collection of re-usable addons and generate your web apps based on your specific needs.

## Usage

TODO: Document the build steps (ref: [napi](https://napi.rs/)).

Create the `TADA_APP` environment variable in your system, and assign the location of this project as the value.

The existing projects under the `templates` directory will be the base for the new app, and the optional **addons** are under the `addons` directory (surprise).

To generate a new app just run the following command anywhere in your system:

```bash
npx create-tada-app
```

## What is the goal?

There are amazing projects that provide a great starting point for robust applications: [create-t3-app](https://github.com/t3-oss/create-t3-app), [create-jd-app](https://github.com/OrJDev/create-jd-app), [react-three-next](https://github.com/pmndrs/react-three-next), etc.

And [Vite](https://vite.dev/guide/#scaffolding-your-first-vite-project) will definitely be the best option to start a project from scratch.

But... What is the middle ground? How can you create a simple app with your favorite tools already included? A template in [CodeSandbox](https://codesandbox.io/)? No, there is something more important... Your own editor setup (I use Neovim btw).

I hope the following use cases can be covered using this CLI:

- Simple command to generate apps from my own templates and with my favorite (**optional**) tools: [tailwindcss](https://tailwindcss.com/), [biome](https://biomejs.dev/), [xr](https://github.com/pmndrs/xr), etc.
- The easiest option to experiment with new technologies or libraries without the hassle of setting up common tools.
- Simple way to reproduce issues and share repositories.

## Why Rust?

I just wanted to learn something new. Actually... I still have a lot to learn, but it's been fun so far.

## TODO

- Add **addons** to existing apps.
- Local test **addons** and their integration on different templates.
    - Possible solution: Create temporal files while running `dev` server.
- Document CLI installation and rules.
