# dotin

A Unix _dotfiles manager_ inspired by, and compatible with `stow`.

`dotin` organizes your config files in the `~/dotfiles` folder. It is great if you want to:

1. Backup and version your configs (with `git`).
2. Easily re-apply them in another machine (with `dotin link`).
3. Maintain changes in sync between machines (with `git`).
4. Use a simple tool.

# Table Of Contents

- Usage (with example)
    - Setup
    - Sync with GitHub
    - Reapplying changes in a new machine
- Differences from `stow`
- Known limitations
- Alternatives

# Usage (with example)

Say you configured a tool, like [`polybar`], and files are laid like this:

```ruby
~
├── .scripts/
│   ├── volumescript.sh
│   └── kb-layout.sh
└── .config/
    └── polybar/
        ├── config.ini
        └── launch.sh
```

After hours of configuring, I bet you don't want to lose those files.

## Setup

Create the group `polybar` at `~/dotfiles/polybar`, and structure it similarly:

```ruby
~/dotfiles/
└── polybar/
    ├── .scripts/
    │   ├── volumescript.sh
    │   └── kb-layout.sh
    └── .config/
        └── polybar/
            ├── config.ini
            └── launch.sh
```

You can move files manually, or just use `dotin import`:

```sh
dotin import polybar .config/polybar
# OR
dotin import polybar .config/polybar/*
```

Now your configs are missing, use `dotin link` to link them back to their original location:

```sh
dotin link polybar
```

Done! Your configs are in place and can be edited using the same path as before, but now, they can be saved in a repository.

## Sync with GitHub

The usual GitHub repository setup:

```sh
cd ~/dotfiles
# In GitHub, create a repository with no README, and follow their instructions or run these:
git init
git commit -a -m "dotfiles repository setup"
git remote add origin <REPOSITORY_URL>
git push -u origin HEAD
```

## Reapplying changes in a new machine

```sh
git clone URL
cd dotfiles
dotin link polybar
```

Done, files are linked to the correct locations (conflicts are reported, if any).

If installing `dotin` is too hard, use `stow` for linking the instead!

```sh
sudo apt install stow
cd ~/dotfiles
stow polybar
```

# Differences from `stow`

`dotin` uses the same tree structure as `stow`, you can use both in the same repository.

Here is how `dotin` differs from `stow`:

- The `import` subcommand.
- Better checks and error messages.
- Can be run from any directory.
    - Expects `"$HOME/dotfiles"`.
- Creates directories (like `mkdir`) when possible, while `stow` prefers to link.
    - In my experience, this avoids accidents.

# Limitations

- `dotin` can link files and directories, but other file types aren't supported yet.
- Can't change dotfiles folder path.

# Non-goals

- Wrap `git` usage.
- Templating.
- Encryption.
- Secrets management.

# Alternatives

- `stow`.
    - Recommended, but overall a worse experience, and easier to mess up.
- Create a script to link and copy everything.
    - Wastes your time, remember to cover corner cases if you don't want to mess up.
- Make your `$HOME` directory a repository and `.gitignore` everything.
    - Annoying to edit `.gitignore` for each file.
    - If you use `git` on a daily, especially with shell integration, it can be quite annoying, every folder you enter will be considered part of the repository.
    - If these don't bother you, read [this](https://drewdevault.com/2019/12/30/dotfiles.html).
- Use [`dotbot`](https://github.com/anishathalye/dotbot) instead.
- Use [`mackup`](https://github.com/lra/mackup) instead.
- Use [`chezmoi`](https://github.com/twpayne/chezmoi) instead.
- Use [some other tool](https://wiki.archlinux.org/title/Dotfiles#Tools).
- Use [some `git`-wrapping tool](https://wiki.archlinux.org/title/Dotfiles#Tools_wrapping_Git).

[`polybar`]: https://github.com/polybar/polybar
