# CorrodeDB

CorrodeDB is a simple "vector like" database where queries are optimized for
getting data from index positions. CorrodeDB supports multiple ways of indexing
and a variety of data transformation functions, more information can be found
in the DOCS.md file.

# Compatability

CorrodeDB has been made and tested on mostly Linux machines and I cannot
guarantee that it will function on non unix systems. A small amount of testing
on Windows has led successfull results but I cannot guarantee any functionality
at this point.

# Building

`cargo build` / `cargo build --release`

# Application Init

Before starting up CorrodeDB you need to define a database schema along with
some general database settings.

A commented **example** for how such a **schema** should look like can be found
in the **"schema_exmaple.yaml"** file. And extra information about the schema
composition can be found in the DOCS.md file.

## Application parameters

CorrodeDB has support for application parameters which can be set by the
following flags. Order of definition doesn't matter.

### -s

flag for setting schema path

#### usage:

-s {schema path} <-- schema path may not include the hyphen character "-" or a space

#### example:

`-s ../another_folder/my_schema.yaml`

#### default:

CorrodeDB will default to **./schema.yaml** if no path is given

### -p

flag for setting port

#### usage:

-p {port}

#### example:

`-p 8008`

#### default:

CorrodeDB will default to port **4067** if no port is given

### -di

flag for running the program in data integrity check mode.
In this mode the program goes through all database data and makes sure it is in
the correct syntax. Writes out all noticed faults in the console during execution.

recommended to use if you have been touching around in the database files manually.

#### usage:

`-di true` has to be used with the value as `true`

#### default:

By default the data integrity check is false

### -cq

flag for running the program in console queries debug mode.
This mode allows the user to make db queries from the console interface.

#### usage:

`-cq true` hast to be used with the values as `true`

#### default:

By default the console queries is false
