% Reasons to avoid Git submodules
%
% 4th Dec 2024

Git submodules let you embed one repo as a subdirectory inside another. There have been many debates about whether this is a good idea. Before using submodules extensively I was on the fence. Many of the reasons given for not using them didn't sound that compelling because the fundamental idea of composing repos sounds reasonable.

However after using them in anger the flaws in Git's specific *implementation* became very apparent. This is a list of those flaws so you don't have to discover them yourself.

## They break worktrees

Worktrees are an extremely useful feature that let you have multiple copies of a repo that share the same `.git` directory (so commits and branches stay synchronised). Unfortunately they don't work reliably with submodules.

Quoting [the man page](https://git-scm.com/docs/git-worktree):

> Multiple checkout in general is still experimental, and the support for submodules is incomplete. It is NOT recommended to make multiple checkouts of a superproject.

In my experience this warning is accurate.

## They break checkouts

When you switch branches you need to run `git submodule update --init --recursive` to update the submodules to point to wherever your target branch points them too. This is a bit of a pain, and Git provides a flag to do it automatically for you: `git switch --update-submodules`. It can be enabled by default so that all `git switch` commands automatically update submodules.

DO NOT USE THIS. It can seriously break your `.git` directory to the point that you may have to delete it and start from scratch. Common error messages are:

* `fatal: not a git repository: /some/path`
* `Failed to clone 'some/submodule'. Retry scheduled`
* `BUG: submodule considered for cloning, doesn't need cloning any more?`
* `fatal: could not get a repository handle for submodule 'some/submodule'`

[This message](https://lore.kernel.org/git/7C3FF354-F10E-4822-ACF8-31417B05C099@gmail.com/) contains some ideas for how to unbreak things if you get stuck in this mess.

Having to run `git submodule update --init --recursive` endlessly is rather tedious.

## They break branches

If you work with multiple branches that contain different sets of submodules be prepared for pain. When switching from a branch with a submodule to another without that submodule, Git will just leave the files on disk and they will become untracked files. You have to manually delete them.

Often you will be met with [`unable to rmdir` errors](https://stackoverflow.com/q/45215094/265521) or other similar where Git just can't quite deal with submodules changing when you switch branches.

This is similar to the behaviour when switching between branches with different `.gitignore`s, but much more annoying!

## They break mirroring

Without submodules, a Git repo is entirely self-contained and portable. It doesn't know where it is hosted; it can be freely forked, cloned and mirrored.

With submodules this is no longer the case - it adds a [`.gitmodules` file](https://github.com/pydrofoil/pydrofoil/blob/main/.gitmodules) which usually contains *absolute* URLs to the submodules. If you clone the repo the submodules will still point to the original URLs.

Even worse, the protocol (https or ssh) is often explicitly stated, which can even lead to the inability to clone the repo at all (for example if SSH URLs are used and you don't have appropriate access permissions).

It is possible to use [relative paths](https://www.damirscorner.com/blog/posts/20210423-ChangingUrlsOfGitSubmodules.html) but many people forget, and the exact format of these (number of `..`s etc.) depends on where you are hosting it.

Technically this is also a problem with any third party dependency, for example if your project uses NPM, Pypi or Cargo libraries. These tools provide ways to [replace dependencies](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html) if necessary. The problem arises when submodules are used for closely linked first party dependencies.

## Tools do not support them well

For example in VSCode by default if you click on the diff of a submodule, it will just show you what the hash changed from/to. Looking that up is very not fun.

There *is* a way to improve this via `git config --global diff.submodule log`, but like many of Git's UX fixes it is not enabled by default. Most developers are not going to know that option even exists.

In general you will find that submodules are an afterthought in most tools.

# What about the idea of submodules?

Ignoring the above points which are all things that are Git-specific and could *theoretically* be fixed (but don't hold your breath!), are submodules a good idea even in principle?

There are two situations where you might want to use submodules:

1. To import third party code that is already a separate project.
2. To split up a large project (e.g. your company codebase) into smaller pieces.

The first is generally reasonable in situations where a proper dependency manager (Cargo, vcpkg, etc.) is not available *and* you are not planning to edit that dependency.

The second is more debatable. You should consider the following:

## Every submodule creates a public interface

A key difference between code that lives in a submodule and code in a subdirectory is that code in a subdirectory has a bounded set of users (other code in the repo), whereas the submodule has an unbounded set (literally any other repo might use it). This means that your code suddenly gains a public API.

By putting the code in a submodule you can no longer trivially make breaking changes to the code because you don't know who might be using it.

## Testing becomes much harder

Even if you develop tooling to identify downstream users of your submodule, actually testing changes is extremely difficult. You can make a change and run CI for your submodule, but did you also bump the version of your submodule in all the other repos that include it and run *their* CI? Probably not. I have done this for some projects using Gitlab CI, which supports triggering downstream pipelines. It's better than nothing, but barely.

## Refactoring becomes much harder

Making any kind of breaking change (including refactoring) becomes very difficult to coordinate across repos. In a monorepo you can just do the refactor, send it to CI and call it a day. In a multi-repo setup this is much much harder. You cannot test the overall change in one go. You have to do it in stages and in a backwards compatible way. It can become especially complicated when two repos depend on each other (yes it can happen!).

## Transitive dependencies get duplicated

If you have a project that has two dependencies as submodules, and those dependencies both have a third dependency as a submodule (a diamond dependency graph), then you will have two copies of the third dependency in your project. This isn't theoretical; I've worked in a repo that heavily uses submodules and ends up with 6 copies of the same submodule.

## Git has poor support for monorepos

One alternative to submodules is a monorepo. Unfortunately Git has quite poor support for monorepos too. As far as I know, only Microsoft uses Git with a large monorepo. Google and Facebook both use custom VCS's.

Git can become *extremely* slow with large repos. Monorepos can become huge, and in that case you really need sparse checkouts and file monitoring.

Git has [*experimental* support for sparse checkouts](https://git-scm.com/docs/git-sparse-checkout), and [file monitoring](https://git-scm.com/docs/git-config#Documentation/git-config.txt-corefsmonitor) (not supported on Linux).

# What should you do instead?

If possible, use a package manager. E.g. for Rust use cargo, for C++ use vcpkg, etc.

For first party dependencies use a monorepo, and a build system which works well with monorepos (Bazel, Buck2, Pants, Please, etc).

For third party dependencies where you can't use a package manager you could consider `git subtree` (which is essentially vendoring the repo). This works fairly well and is especially good if you will be modifying the contents of the submodules. It does make pulling and pushing to the original repo a bit more of a pain though.

Another option is [`git subrepo`](https://github.com/ingydotnet/git-subrepo), which looks very similar to `git subtree` (I haven't used it).

Ultimately I don't think Git has a good answer to the question that leads people to submodules. I wonder if [Pijul](https://pijul.org/) or [Sapling](https://github.com/facebook/sapling) are any better...

# Bonus: better Git config

Git has a habit of fixing UX issues and then hiding the fix behind a flag that nobody notices. Here are a few I have discovered. Set them all!

* `git config --global diff.submodule log` - better submodule diffs
* `git config --global alias.pushf 'push --force-with-lease --force-if-includes'` - safe pushing; this checks you aren't clobbering someone else's commits accidentally
* `git config --global push.autoSetupRemote true` - never have to copy and paste `git push --set-upstream origin branchname` again!
* `git config --global merge.conflictStyle zdiff3` - better merge conflicts (though Git is still pretty shit at these in my experience)
* `git config --global fetch.prune true` - automatically delete your local copy of `origin/foo` when that branch is deleted on the remote (usually because the PR/MR merged)

In copy/pasteable form:

```
git config --global diff.submodule log
git config --global alias.pushf 'push --force-with-lease --force-if-includes'
git config --global push.autoSetupRemote true
git config --global merge.conflictStyle zdiff3
git config --global fetch.prune true
```
