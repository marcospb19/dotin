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

We'll explain how to use `dotin` to organize this into two folders inside `~/dotfiles`:

```ruby
~/dotfiles/
├── zsh/
│   └── ...
└── polybar/
    └── ...
```

The files will be organized into two groups, `zsh` and `polybar`.

## Importing the files

To import, first pass the group name, then provide the files to be imported.

```sh
dotin import zsh .zprofile
dotin import zsh .zshrc
# or
dotin import zsh .zprofile .zshrc
```

If they don't exist already, `dotin` will create both folders `~/dotfiles/` and `~/dotfiles/zsh/`, then move the files inside.

Here is what it looks like:

```ruby
~/dotfiles/
└── zsh/
    ├── .zprofile
    └── .zshrc
```

Even though your files were moved, everything works like before because the files were sym-linked to their original location, here is what's in your HOME now.

```ruby
~
├── .zprofile -> ~/dotfiles/zsh/.zprofile
└── .zshrc    -> ~/dotfiles/zsh/.zshrc
```

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

Like before, it's all linked and working, now, if you try to edit the files at your home, you'll actually end up editing the files inside of `~/dotfiles`.

## Sync With GitHub

With all configs living inside a single folder, we can easily turn it into a repository and back them up using `git` and `GitHub`:

```sh
# Just the usual GitHub repository setup
cd ~/dotfiles
git init
git commit -a -m "dotfiles repository setup"
# Now, inside of GitHub, create your repository without a README, and follow their instructions that look like these:
git remote add origin <REPOSITORY_URL>
git push -u origin HEAD
```

## Reapplying Configs In a New Machine

With `dotin` installed, you can re-apply all configs:

```sh
git clone URL
cd dotfiles
dotin link zsh
dotin link polybar
```

Done, files are linked to the correct locations (conflicts are reported, if any).

If you're in a hurry and don't want to install `dotin`, try using `stow` instead:

```sh
# Installation for Debian-based and Ubuntu-based
sudo apt install stow
# Installation for Arch-based
sudo pacman -S stow

# stow requires that you are inside your dotfiles (or, use some flag)
cd ~/dotfiles
stow polybar # same as `dotin link polybar`
```

# Differences from `stow`

`dotin` uses the same tree structure as `stow`, they are compatible.

Both tools are still similar, `dotin` is under development and there is a lot to be done, for now, here is how `dotin` differs from `stow`:

- `dotin` runs more checks before linking or moving.
- Simpler and more intuitive usage.
- Better checks and error messages.

# Known Issues

- `dotin` fails when dealing with exoteric file types.

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
