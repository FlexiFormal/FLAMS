# FLAMS Foreign Function Interface (FFI)

The FLAMS ffi gives access to the sTeX annotations of files
using the quickparse functionality that is also used in the LSP.

It maintains a hash map of parsed files and their annotations.
If files are changed, they can be "unloaded" (i.e. removed from the hash map),
and then reloaded.
Note that neither dependencies nor dependents are automatically reloaded.

The data is encoded as JSON. Note that the strings returned by the ffi are
must be explicitly freed by the caller using `free_string`.

## Interface

```c
// A simple test function (prints to stdout)
void hello_world(size_t arg);

// Initializes the ffi (must be called before any other function)
void initialize();

// Frees a string allocated by the ffi
void free_string(char* s);

// Loads all files
void load_all_files();

// Returns a JSON array of the paths of all loaded files
char* list_of_loaded_files();

// Returns the JSON representation of the annotations of a file (requires the file to be loaded)
char* get_file_annotations(char* path);

// Unloads a file
void unload_file(char* s);

// Loads a file
void load_file(char* s);
```
