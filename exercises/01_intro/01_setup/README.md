```bash
$ uv python install 3.13
```

Copy the dynamic Python library libpython3.x.(dylib|so) to `rust-python-interoperability/target/debug`.

```bash
$ ln -s -f $HOME/.local/share/uv/python/cpython-3.13.2-linux-x86_64-gnu/lib/libpython3.13.so.1.0 $HOME/projects/rust-python-interoperability/target/debug/deps/
```

```
$ cd 01_setup
$ wr
```
