# gomori-py

## Setup

You'll probably want to use a virtualenv for the following steps. You can either use a pre-built wheel or build it from source yourself.

### Using pre-built wheel

Download a wheel for your platform from the [latest release](https://github.com/nnmm/gomori/releases).

```
pip install wheel
pip install <path to the wheel>
```

### Building from source

```
pip install maturin
maturin develop
```

## Usage

After setup is completed, you can just `from gomori import *`.

The API is pretty much the same as in Rust, so check out the documentation in that package.