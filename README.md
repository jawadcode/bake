# bake

A simple build system for C/C++.

# Features:

* Supports projects with a flat file structure (a bit of a limitation but oh well)
* Performs incremental compilation, i.e. only changed .c or .cpp files are recompiled
* `bake run` builds the project and then runs the final executable