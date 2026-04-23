# C FFI Minimal Example

This folder demonstrates calling `eml-rs` from C via `cdylib`.

## 1) Build shared library

```bash
cargo build --release
```

Generated library path depends on platform:
- Windows: `target/release/eml_rs.dll` (+ import lib `eml_rs.dll.lib`)
- Linux: `target/release/libeml_rs.so`
- macOS: `target/release/libeml_rs.dylib`

## 2) Build C example

### Linux/macOS

```bash
cc examples/c_ffi_minimal/main.c -Iinclude -Ltarget/release -leml_rs -o eml_c_demo
```

### Windows (MSVC)

```powershell
cl /nologo /Iinclude examples\c_ffi_minimal\main.c /link /LIBPATH:target\release eml_rs.dll.lib /OUT:eml_c_demo.exe
```

If `cl.exe` is launched outside Developer Command Prompt and cannot find C headers,
use MinGW GCC with the generated import library:

```powershell
gcc examples\c_ffi_minimal\main.c -Iinclude -Ltarget\release -l:eml_rs.dll.lib -o eml_c_demo.exe
```

## 3) Run

Ensure runtime can locate the shared library:
- Linux/macOS: set `LD_LIBRARY_PATH` / `DYLD_LIBRARY_PATH`
- Windows: place `eml_rs.dll` next to `eml_c_demo.exe` or in `PATH`
