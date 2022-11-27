# Structure

Tool should auto-generate the following directory structure for new projects

```
project_name/
|- src/
|   |- main.c
|- bin/
|-
```

`bake build` should result in:

```
project_name/
|- src/
|   |- main.c
|- bin/
|   |- debug/
|   |   |- main.c.o
|   |   |- project_name
|- bake.toml
```

and `bake build --release`:

```
project_name/
|- src/
|   |- main.c
|- bin/
|   |- release
|   |   |- main.c.o
|   |   |- project_name
|- bake.toml
```

### Example project:

Given 

```
mandelbrot_c/
|- src/
|   |- common.h
|   |- hsv.c
|   |- hsv.h
|   |- main.c
|   |- mandelbrot.c
|   |- mandelbrot.h
|- bake.toml
```

`bake build` should give:

```
mandelbrot_c/
|- src/
|   |- common.h
|   |- hsv.c
|   |- hsv.h
|   |- main.c
|   |- mandelbrot.c
|   |- mandelbrot.h
|- bin/
|   |- debug/
|   |   |- hsv.c.o
|   |   |- main.c.o
|   |   |- mandelbrot.c.o
|   |   |- mandelbrot_c
|- bake.toml
```

## Incremental compilation:

Compare last-modified date of object files in bin/:
* If the object file is newer than the corresponding file(s) in src/ then nothing needs to be done
* If the object file is older than the corresponding file(s) in src/ then:
  * Recompile that particular source file
  * Run the linker again
