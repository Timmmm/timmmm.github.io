% Cross-compiling Rust on Github Actions
%
% 23th Sep 2023

I saw [a recent article about cross-compiling Rust in Github actions](https://reemus.dev/tldr/rust-cross-compilation-github-actions), but they didn't actually cross-compile - instead building natively on different machines.

That's a good approach if you're just building separate installers for each platform, but sometimes you want binaries for all platforms in the same place.

You *can* use multiple CI stages that run on different OSes and pass artifacts between them, but that really complicates your CI setup and build system.

Cross-compiling is also better from a cost perspective. On Github a Windows machine is double the price of Linux, and Mac is 10 times the price!

To cross compile, you can theoretically use [cross](https://github.com/cross-rs/cross), but it can't cross-compile to Mac and it also uses Docker which can be problematic in CI (nested Docker is not trivial). Therefore I prefer "native" cross-compilation.

The following Gitlab CI steps *actually* cross compile from Linux to X86 Linux, Windows and Mac, and ARM Mac. Also I use Musl on Linux to avoid inevitable Glibc compatibility issue.

```yaml
jobs:
  build:
    runs-on: ubuntu-latest

    steps:
    - name: Install Linux and Windows Cross Compilers
      run: sudo apt-get install --yes --no-install-recommends musl-tools gcc-mingw-w64-x86-64-win32

    - name: Set up MacOS Cross Compiler
      uses: Timmmm/setup-osxcross@v2
      with:
        osx-version: "12.3"

    - name: Install Rustup targets
      run: rustup target add x86_64-unknown-linux-musl x86_64-pc-windows-gnu x86_64-apple-darwin aarch64-apple-darwin

    - name: Check out source code
      uses: actions/checkout@v3

    - name: Build
      run: ...
```

The first step installs Musl and Windows compilers. These are generally trivial to target from Linux. Mac is the problem child. To cross-compile to Mac you need a copy of the MacOS SDK which Apple doesn't freely provide. There's a repo on Github that keeps a legally dubious collection of them and Apple hasn't complained yet as far as I know.

The `setup-osxcross` action takes care of building a cross-compiler and installing the MacOS SDK for you. The one I linked contains an extra commit which adds ARM support. The first time it runs it builds Clang from source, which can take a while, but future runs are cached. Note that only *successful* builds write to the cache so I recommend pushing a CI config with only that step first so you don't have to wait for Clang builds to debug other issues.

Once built you can build just by specifying the Rust target. You can actually build multiple targets at once.

```shell
cargo build --target x86_64-unknown-linux-musl --target x86_64-pc-windows-gnu --target x86_64-apple-darwin --target aarch64-apple-darwin
```

I did have one issue building my program, which depends on [Chumsky](https://github.com/zesterer/chumsky) - it couldn't cross compile the [psm crate](https://crates.io/crates/psm), but fortunately that's an optional feature in Chumsky so I've just disabled it for now.

[A demo repo is available here.](https://github.com/Timmmm/rust_cross_compile_demo) For demonstration purposes the entire `target` directory is uploaded as an artifact, but you probably want to do something else. Also this doesn't cover signing, or making App bundles for Mac.

I hope that helps some people, thanks for reading!
