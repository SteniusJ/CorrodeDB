# CorrodeDB

CorrodeDB is a simple "vector like" database where queries are optimized for
getting data from index positions. It has support for four functions that add
vital functionality and improve usability, more info about these functions can
be found below in the **Queries** section.

# Compatability

CorrodeDB has been made and tested on mostly Linux machines and I cannot
guarantee that it will function on non unix systems. A small amount of testing
on Windows has led successfull results but I cannot guarantee any functionality
at this point.

# Application Init

Before starting up CorrodeDB you need to define a database schema along with
some general database settings.

A commented **example** for how such a **schema** should look like can be found
in the **"schema_exmaple.yaml"** file.

# Database schema

The schema does have some restrictions which should be followed:

- **table names** can only have alpha numeric values (for example, these are
  not valid characters ",\_-/")
- **columns** have no such restrictions
- There are currently three **datatypes**: **NumberI** (Integer values),
  **NumberF** (Floating point values), **VarChar** (String values)

# General Settings

Currently there are two settings:

- **rows** this setting controls the amount of rows per database "container", a
  value of 50 will mean that the database will append 50 values into a "container"
  before creating a new one.
- **password** the database password required for data access. This password is
  given to the database as a url parameter called "password"

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

# Queries

CorrodeDB has a unique query structure and support for three functions read,
write and remove

Currently, post tokenizer update, the query structure is very flexible for better or worse.

### For example:

`messages[10 8]4,11 20`

The above is a completely valid query that will get the items at the indexes "10,8,4,11,20"
in the table "messages". This is because the characters `[ ] , " "` all function as
splitters. Due to this you can pick and choose to use spaces or commas or opening/closing
brackets as separators if you wish. I however recommend to use the currently given query
structure since this is as I intended for the queries to look, and a future update might
require the queries to look as such.

## Data read

The syntax for reading data is:

`{table name}[{indexes}]`

indexes can be subtracted from. For example:

`messages[10-5]`

will return row at index 5

if used in tandem with the Wildcard this will get the biggest index minus given value

`messages[*-10]`
will give biggest index -10. So if a table has 20 values this will result in the value
at index 10.

### Single index:

`messages[20]`

this query will get the data at the index **20** in the table called **messages**

### Multi index:

`messages[1,5,10,20,*-5]`

this query will get the data at the indexes **1,5,10,20,68** in the table
called **messages**

### Index range:

`messages[60..100]`

this query will get the data at the indexes **60,61,62... 98,99,100** in the
table called **messages**

`messages[50..*]`

this query wil get the data at the indexes **50,51,52... maximum table index**
in the table called **messages**

### Wildcard:

`messages[*]`

this query will get the data at **all indexes** in the table called **messages**

### Page:

`messages[p0]`

this query will get the data from 0 to max or end of container. For example if
you have defined 50 rows in database settings it will get 0-49 and `p1` would
get 50-99 etc. If container ends it will get up to that point.

## Data write

the syntax for writing data is:

`{table name}[{index}] write {new data}`

The new data is given in a comma seperated string.
VarChar values should be encapsulated in double quotes.
This is not necessary if a string doesn't include any of the spacer
characters. Double quotes inside VarChar values can be escaped with
a forward slash "\".

### Writing to specific index:

`messages[10] write "hello, world",100,12.239`

this query will write the data **"hello, world" | 100 | 12.239** to the index **10**

`messages[5,10,15,20] write "hello, world",100,12.239`

this query will write the data **"hello, world" | 100 | 12.239** to the indexes **5,10,15,20**

also works with other types of indexing like range

### Appending to database:

`messages[*] write "hello, world",100,12.239`

this query will write the data **"hello, world" | 100 | 12.239** to the last
available index.

## Data remove

the syntax for removing data is:

`{table name}[{index}] remove`

Data remove also works with the other types of indexing.
Using data remove with the Wildcard "\*" drops tables.

### Example:

`messages[10] remove`

this query will remove the data at index **10**

# Sub functions

Sub functions are data transformation functions which can be used to transform
the returned db data.

## Syntax:

sub functions are used by "piping" the data into them.

### example:

`messages[*] | sort message,asc`

this query "pipes" the return data of `messages[*]` into the sub function "sort"
which sorts the return in ascending order by the column "messages".

sub functions can be used on any db functions which return data. For example
sub functions can not be used on the function "remove" or "write" since
these do not return data, just a status.

#### this is a valid use of the sort sub function:

`messages[10..100] | where message,in,hello | sort message,asc`

#### this is not:

`messages[*] write "hello world" | sort message,asc`

sub functions can also be theoretically infinitely "stacked".

#### For example:

this is a completely valid, though stupid, query which you can do by stacking
all of these sub functions.

`messages[*] | sort message,asc | where message,in,a | random 5 | sort message,dsc | random 2`

this query will return 2 random messages which have a value in the column "message"
that includes the letter "a".

## sort

sorts data in ascending or descending order by given column.
Uses merge sort algorithm.

### usage:

`messages[*] | sort message,asc`

### Parameters:

`messages[*] | sort {column name},{sort mode}`

#### Column name:

any name of a database column in the given table.

#### Sort mode:

- `asc` for ascending
- `dsc` for descending

## where

returns rows which match the given comparison value.

### usage:

`messages[*] | where message,in,"hello world"`

### Parameters

`messages[*] | where {column_name},{operator},{comparison_value}`

#### Column name:

any name of a database column in the given table.

#### Operator:

- `=` for values that equal comparison_value, case sensitive for VarChar
- `>` for values that are more than comparison_value (not usable on VarChar values!)
- `<` for values that are less than comparison_value (not usable on VarChar values!)
- `in` for values that includes the comparison_value (only usable on VarChar values!)

#### Comparison value:

value used for comparison with row value. Should have the same datatype as the
column which is being compared with.

## random

returns n number of random values

### usage:

`messages[*] | random 3`

### Parameters

`messages[*] | random {nr_of_random_values}`

#### Nr of random values:

a integer value defining the amount of random values to get.
Should be less than the number of values given to the function.
Or a "\*" for Wildcard which will shuffle the whole vector

#### for example:

`messages[0..3] | random 5`

this query is not valid because you are attempting to retrieve 5 values
out of a vector which is of size 4.

