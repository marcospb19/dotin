# dotin

A Unix _dotfiles manager_ inspired by, and compatible with `stow`.

`dotin` concentrates your configs in a `~/dotfiles` folder in order to help with:

1. Backup/version control your configs (use `git`).
2. Easily re-apply configs in another installation/machine (run `dotin link`).
3. Maintain changes in sync between machines (use `git`).

You can roughly achieve what `dotin` does by creating a custom script with `mv` and `ln -s` commands, however, `dotin` has lots of checks for conflicts and corner cases, when possible, checks are done before any mutation is done, ensuring you don't end up with a partial update.

# Table of Contents

- Usage
  - Importing the files
  - Sync With GitHub
  - Reapplying Configs In a New Machine
- Differences from `stow`
- Known limitations
- Alternatives

# Usage

Say you were configuring `polybar` and `zsh`, and ended up creating these config files in your home:

```ruby
~
├── .zprofile          (new, zsh)
├── .zshrc             (new, zsh)
└── .config/
    └── polybar/       (new, polybar)
        ├── config.ini (new, polybar)
        └── launch.sh  (new, polybar)
```

We'll use `dotin` to organize this into two folders inside `~/dotfiles`:

```ruby
~/dotfiles/
├── zsh/
│   └── ...
└── polybar/
    └── ...
```

## Importing the files

To import the files into the `~/dotfiles/` we must provide the group name, the given files will go into `~/dotfiles/GROUP/`:

```sh
dotin import zsh .zprofile
dotin import zsh .zshrc
# or
dotin import zsh .zprofile .zshrc
```

If they don't exist, `dotin` will create both `~/dotfiles/` and `~/dotfiles/zsh/` before importing the files.

Here is the current `~/dotfiles` structure:

```ruby
~/dotfiles/
└── zsh/
    ├── .zprofile
    └── .zshrc
```

We moved the files, but `zsh` still works like before because `dotin` created _symlinks_ back at their original location:

```ruby
~
├── .zprofile -> ~/dotfiles/zsh/.zprofile
└── .zshrc    -> ~/dotfiles/zsh/.zshrc
```

Because we use symlinks, if you edit `~/.zprofile` or `~/.zshrc`, your editor will edit the real file inside `~/dotfiles/zsh/`.

Now, let's do the same thing for `polybar`:

```sh
dotin import polybar .config/polybar
```

So now we get:

```ruby
~/dotfiles/
├── zsh/
│   ├── .zprofile
│   └── .zshrc
└── polybar
    └── .config/
        └── polybar/
            ├── config.ini
            └── launch.sh
```

By organizing the files in this structure, `dotin` knows what their original location was.

## Sync With GitHub

With all configs inside the `~/dotfiles` folder, now we can turn it into a repository using `git`:

```sh
# Just the usual GitHub repository setup
cd ~/dotfiles
git init
git commit -a -m "dotfiles repository setup"
# Now, inside of GitHub, create your repository without a README, and follow their instructions that look like these:
git remote add origin <REPOSITORY_URL>
git push -u origin HEAD
```

Done, your configs are backed up.

## Reapplying Configs In a New Machine

Requirements: `dotin` and `git`:

```sh
git clone <REPOSITORY_URL>
dotin link zsh
dotin link polybar
```

Done, files are linked to the correct locations (conflicts are reported, if any).

Alternatively, you can link using `stow` instead of `dotin`:

```sh
# stow is more widely available
sudo apt install stow
# `stow` requires you to either be inside the folder or provide flags
cd ~/dotfiles
# same as `dotin link polybar`
stow polybar 
```

# Differences from `stow`

`dotin` uses the same tree structure as `stow`, they are compatible.

Both tools are still similar, `dotin` is under development and there is a lot to be done, for now, here is how `dotin` differs from `stow`:

- `dotin` runs more checks before linking or moving.
- Simpler and more intuitive usage.
- Better checks and error messages.

# Known Issues

- `dotin` fails when dealing with unusual file types.

# Non-goals

- Wrap `git` usage.
- Encryption.
- Secrets management.

# Alternatives

- `stow`.
  - Recommended, but overall a worse experience for dotfiles (in my personal opinion).
- Make your entire `$HOME` a repository and `.gitignore` everything.
  - Good, edit `.gitignore` to add or remove files.
  - If you like the idea, read [this](https://drewdevault.com/2019/12/30/dotfiles.html).
- Create your own script.
  - You'll likely waste time and end with worse ahead-of-time checks on conflicts and weird corner cases.
  - Go for it if it'll be fun.
- Use [`dotbot`](https://github.com/anishathalye/dotbot) instead.
- Use [`mackup`](https://github.com/lra/mackup) instead.
- Use [`chezmoi`](https://github.com/twpayne/chezmoi) instead.
- Use [some other tool](https://wiki.archlinux.org/title/Dotfiles#Tools).
- Use [some `git`-wrapping tool](https://wiki.archlinux.org/title/Dotfiles#Tools_wrapping_Git).
