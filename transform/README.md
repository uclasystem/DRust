# DRust Code Transformation

DRust simplifies the process of converting a single-machine Rust program to be compatible with DRust by leveraging the [Structure Search and Replace (SSR)](https://rust-analyzer.github.io/manual.html#structural-search-and-replace) functionality of `rust-analyzer`. Below, we demonstrate how to use this feature with a simple example: GEMM (General Matrix Multiply). Additional examples and more rules will be released soon. 

In the `gemm`` folder, you'll find a runnable, single-machine version of a GEMM application. Follow these steps to transform it for DRust:

#### 1. Set Up Your Environment
- Open the project in VSCode.
- Install the `rust-analyzer` extension if you haven't already.

#### 2. Apply Structural Search and Replace (SSR)
For each file in the GEMM project:
- Select all the contents of the file.
- Open the command palette (`Ctrl+Shift+P` or `Cmd+Shift+P` on macOS) and choose the `rust-analyzer: Structural Search Replace` command.
- Apply the following SSR rules one at a time:
  ```text
  Vec<$a> ==>> DVec<$a>
  Vec::$a ==>> DVec::$a
  Box<$a> ==>> DBox<$a>
  Box::$a ==>> DBox::$a
  Mutex<$a> ==>> DMutex<$a>
  Mutex::$a ==>> Mutex::$a
  ```

#### 3. Adjust Imports

After applying the SSR rules, search for the import statement `use drust::prelude::*` and replace it with `use crate::prelude::*`.

#### 4. Organize Transformed Code

- Copy the transformed files to `drust/src/transformed/gemm/`.
- Rename `main.rs` to `mod.rs` and remove the main function to finalize the DRust-compatible GEMM application.


> Note: 
> - This automatic code transformation is still in its early stages and is meant primarily for demonstration purposes.
> - The performance of the transformed application may be suboptimal at this time.
