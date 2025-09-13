## To run tests

Note that you must run `pytest` inside the python `sample` folder,
otherwise the virtual env will not pick up `pyo3` module that was created.

```bash
cd sample
uv run pytest
```
