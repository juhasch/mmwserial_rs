# mmwserial

A Python package for data transfer between a TI radar board and a PC.
For the xWR68xx is uses a virtual serial port, for the AWR2544 it uses UDP.
The time critical part is implemented in Rust.

Meant to work with the [xwr68xxisk](https://github.com/juhasch/xwr68xxiskhttps://github.com/juhasch/xwr68xxisk) package.

## Installation

You can install the package directly from PyPI:

```pip install mmwserial```

## Development Installation

To install in development mode:

1. Clone this repository
2. Run `pip install maturin`
3. Run `maturin develop`
```

Now you can build and distribute your package in several ways:

1. For development:
```bash
maturin develop
```

2. To create a wheel for distribution:
```bash
maturin build
```

3. To publish to PyPI:
```bash
maturin publish
```

The key differences from your original setup:

1. We're using `pyproject.toml` as the main configuration file, which is the modern standard for Python packages
2. The build process is simplified since maturin handles most of the complexity
3. The package is now properly structured for PyPI distribution

To install the package, users can simply run:
```bash
pip install mmwserial
```

Make sure to update the `pyproject.toml` with your actual package information, dependencies, and other metadata. Also, if you have any Python dependencies, you can add them to the `project.dependencies` section in the `pyproject.toml` file.

