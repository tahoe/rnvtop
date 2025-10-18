# rnvtop
Rust Nvidia SMI top like program, as a playground for eventually writing a gnome extension for monitoring GPU usage in the toolbar...

## There are 3 different output types

* Tabular `rnvtop -tc`

  ![Table View](./artifacts/tabular.png "Table View")

* Json `rnvtop -j`

  ![Json View](./artifacts/json.png "Json View")

* Mutliline print `rnvtop -lc`

  ![Multiline View](./artifacts/multiline.png "Multiline View")

* Help (`-h`) output:
```
General Nvidia GPU monitoring

Usage: rnvtop [OPTIONS]

Options:
  -l, --loopit       
  -f, --freq <FREQ>  [default: 1]
  -c, --colorize     
  -j, --json         
  -t, --tabular      
  -h, --help         Print help
  -V, --version      Print version
```
