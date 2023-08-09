![logo](https://assets.gitlab-static.net/uploads/-/system/group/avatar/10553166/logo_white.ico?width=64)

# bp3d-os

Operating system tools for BlockProject 3D.

This crate provides various platform specific tools to use by most BlockProject 3D software.

## Features

- **assets** provides access to files bundled within the application (eg: AppImage for Linux and Application bundles 
  for Apple systems).
- **dirs** provides access to various system directories to place application data, logs, config, etc.
- **fs** provides a few utilities to extend the Rust standard library around the platform file system such as managing
  hidden files or getting the absolute path of a file or directory.
- **open** provides utilities to open URLs and files using the default registered application.

## Supported platforms

Currently, the following platforms are supported:
- macOS.
- macOS (with App sandbox).
- iOS.
- Linux.
- Windows.
- Other BSD like systems.

Support for Android is currently not planned due to drastic restrictions on file sizes and largely increased complexity
of interacting with the platform (No native C API, everything goes through JVM with some custom main function).

### Support for the *assets* feature

| Platform                 | Tested | Notes                    |
|--------------------------|--------|--------------------------|
| macOS                    | Yes    | Uses NSBundle API.       |
| macOS (with app sandbox) | Yes    | Uses NSBundle API.       |
| iOS                      | Yes    | Uses NSBundle API.       |
| Linux                    | Yes    | -                        |
| Windows                  | No     | Uses GetModuleFileNameW. |
| Other BSD like systems   | No     | -                        |

### Support for the *dirs* feature

| Platform                 | Tested | Notes                      |
|--------------------------|--------|----------------------------|
| macOS                    | Yes    | Uses NSFileManager API.    |
| macOS (with app sandbox) | Yes    | Uses NSFileManager API.    |
| iOS                      | Yes    | Uses NSFileManager API.    |
| Linux                    | Yes    | -                          |
| Windows                  | No     | Uses SHGetKnownFolderPath. |
| Other BSD like systems   | Yes    | -                          |

### Support for the *fs* feature

| Platform                 | Tested | Notes                   |
|--------------------------|--------|-------------------------|
| macOS                    | Yes    | Uses NSFileManager API. |
| macOS (with app sandbox) | Yes    | Uses NSFileManager API. |
| iOS                      | Yes    | Uses NSFileManager API. |
| Linux                    | Yes    | -                       |
| Windows                  | No     | -                       |
| Other BSD like systems   | Yes    | -                       |

### Support for the *open* feature

| Platform                 | Tested | Notes                        |
|--------------------------|--------|------------------------------|
| macOS                    | Yes    | Uses NSWorkspace API.        |
| macOS (with app sandbox) | Yes    | Uses NSWorkspace API.        |
| iOS                      | No     | Not yet implemented.         |
| Linux                    | Yes    | Uses dbus and xdg-open.      |
| Windows                  | Yes    | Uses ShellExecuteW.          |
| Other BSD like systems   | No     | Uses the same code as Linux. |

### Support for the *env* feature

No platform specific APIs are required for this feature except support for
environment variables which is usually missing from mobile platforms.

| Platform                 | Tested | Notes          |
|--------------------------|--------|----------------|
| macOS                    | Yes    | -              |
| macOS (with app sandbox) | Yes    | -              |
| iOS                      | No     | Not applicable |
| Linux                    | Yes    | -              |
| Windows                  | Yes    | -              |
| Other BSD like systems   | No     | -              |

### Support for the *cpu-info* feature

| Platform                 | Tested | Notes                                                         |
|--------------------------|--------|---------------------------------------------------------------|
| macOS                    | Yes    | Uses `machdep.cpu.core_count` and `machdep.cpu.brand_string`. |
| macOS (with app sandbox) | Yes    | Same as macOS.                                                |
| iOS                      | Yes    | Same as macOS.                                                |
| Linux                    | Yes    | Uses `raw_cpuid` on x86_64, None on other architectures.      |
| Windows                  | Yes    | Uses `raw_cpuid` on x86_64, None on other architectures.      |
| Other BSD like systems   | Yes    | Uses `hw.ncpu` and `hw.model`.                                |

### Support for the *time* feature

| Platform                 | Tested | Notes                                                  |
|--------------------------|--------|--------------------------------------------------------|
| macOS                    | Yes    | Reads and interprets the contents of `/etc/localtime`. |
| macOS (with app sandbox) | No     | Same as macOS.                                         |
| iOS                      | Yes    | Same as macOS.                                         |
| Linux                    | No     | Same as macOS.                                         |
| Windows                  | Yes    | Uses `GetTimeZoneInformation`.                         |
| Other BSD like systems   | No     | Same as macOS.                                         |
