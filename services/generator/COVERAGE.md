# Coverage Metrics

To generate and view test coverage metrics as an HTML page for this Rust project, we use `cargo-llvm-cov`. This is highly recommended for accurate source-based coverage.

## 1. Install cargo-llvm-cov

First, install the tool via Cargo:

```bash
cargo install cargo-llvm-cov
```

## 2. Generate HTML Coverage Report

To run your tests and generate an HTML coverage report, run the following command in root generator directory:

```bash
cargo llvm-cov --html
```

## 3. View the Report

Once the command finishes, it will print the location of the generated report to your console. 
By default, the main HTML file is located at `./target/llvm-cov/html/index.html`.

You can open it in your default web browser.