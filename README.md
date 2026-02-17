# CorrodeDB

CorrodeDB is a simple "vector like" database where queries are optimized for
getting data from index positions. It has support for four functions that add
vital functionality and improve usability, more info about these functions can
be found below in the **Queries** section.

## Compatability

CorrodeDB has been made and tested on mostly Linux machines and I cannot
guarantee that it will function on non unix systems. A small amount of testing
on Windows has led successfull results but I cannot guarantee any functionality
at this point.

## Application Init

Before starting up CorrodeDB you need to define a database schema along with
some general database settings.

A commented **example** for how such a **schema** should look like can be found
in the **"schema_exmaple.yaml"** file.

### Database schema

The schema does have some restrictions which should be followed:

- **table names** can only have alpha numeric values (for example, these are
  not valid characters ",\_-/")
- **columns** have no such restrictions
- There are currently three **datatypes**: **NumberI** (Integer values),
  **NumberF** (Floating point values), **VarChar** (String values)

### General Settings

Currently there are two settings:

- **rows** this setting controls the amount of rows per database "container", a
  value of 50 will mean that the database will append 50 values into a "container"
  before creating a new one.
- **password** the database password required for data access. This password is
  given to the database as a url parameter called "password"

### Application parameters

CorrodeDB has support for application parameters which can be set by the
following flags. Order of definition doesn't matter.

#### -s

flag for setting schema path

**usage:**<br>
-s {schema path} <-- schema path may not include the hyphen character "-" or a space<br>
**example:**<br>
`-s ../another_folder/my_schema.yaml`<br>
**default**<br>
CorrodeDB will default to **./schema.yaml** if no path is given

#### -p

flag for setting port

**usage:**<br>
-p {port}<br>
**example:**<br>
`-p 8008`<br>
**default**<br>
CorrodeDB will default to port **4067** if no port is given

#### -di

flag for running the program in data integrity check mode.
In this mode the program goes through all database data and makes sure it is in
the correct syntax. Writes out all noticed faults in the console during execution.

recommended to use if you have been touching around in the database files manually.

**usage:**<br>
`-di true` has to be used with the value as `true`<br>
**default**<br>
By default the data integrity check is false

#### -cq

flag for running the program in console queries debug mode.
This mode allows the user to make db queries from the console interface.

**usage:**<br>
`-cq true` hast to be used with the values as `true`<br>
**default**<br>
By default the console queries is false

## Queries

CorrodeDB has a unique query structure and support for three functions read,
write and remove

### Data read

The syntax for reading data is:<br>
`{table name}[{indexes}]`

**Single index**<br>
`messages[20]`<br>
this query will get the data at the index **20** in the table called **messages**

**Multi index**<br>
`messages[1,5,10,20,68]`<br>
this query will get the data at the indexes **1,5,10,20,68** in the table
called **messages**

**Index range**<br>
`messages[60..100]`<br>
this query will get the data at the indexes **60,61,62... 98,99,100** in the
table called **messages**<br>
`messages[50..*]`<br>
this query wil get the data at the indexes **50,51,52... maximum table index**
in the table called **messages**<br>
`messages[*-20..*]`<br>
this query will get the data at the indexes **maximum table index - 20... maximum table index**
in the table called **messages**

**Wildcard**<br>
`messages[*]`<br>
this query will get the data at **all indexes** in the table called **messages**

### Data write

the syntax for writing data is:<br>
`{table name}[{index}] write {new data}`

The new data is given in a comma seperated string.
Characters used in the query syntax can be escaped in VarChar
values by using the "\\" escape character. For example "," and "|" are both used in
the query syntax and they can be escaped by typing "\\," or "\\|".

**Writing to specific index**<br>
`messages[10] write hello\, world,100,12.239`<br>
this query will write the data **"hello, world" | 100 | 12.239** to the index **10**<br>
`messages[5,10,15,20] write hello\, world,100,12.239`<br>
this query will write the data **"hello, world" | 100 | 12.239** to the indexes **5,10,15,20**<br>
also works with other types of indexing like range

**Appending to database**<br>
`messages[*] write hello\, world,100,12.239`<br>
this query will write the data **"hello, world" | 100 | 12.239** to the last
available index.

### Data remove

the syntax for removing data is:<br>
`{table name}[{index}] remove`

Data remove also works with the other types of indexing.
Using data remove with the Wildcard "\*" drops tables.

**Example**<br>
`messages[10] remove`<br>
this query will remove the data at index **10**

## Sub functions

Sub functions are data transformation functions which can be used to transform
the returned db data.

**Syntax:**<br>
sub functions are used by "piping" the data into them.<br>
**example:**<br>
`messages[*] | sort message,asc`<br>
this query "pipes" the return data of `messages[*]` into the sub function "sort"
which sorts the return in ascending order by the column "messages".

sub functions can be used on any db functions which return data. For example
sub functions can not be used on the function "remove" or "write" since
these do not return data, just a status.

**this is a valid use of the sort sub function:**<br>
`messages[10..100] | where message,in,hello | sort message,asc`<br>
**this is not:**<br>
`messages[*] write hello world | sort message,asc`<br>

sub functions can also be theoretically infinitely "stacked".

**For example:**<br>
this is a completely valid, though stupid, query which you can do by stacking
all of these sub functions.<br>
`messages[*] | sort message,asc | where message,in,a | random 5 | sort message,dsc | random 2`<br>
this query will return 2 random messages which have a value in the column "message"
that includes the letter "a".

### sort

sorts data in ascending or descending order by given column.
Uses merge sort algorithm.

**usage:**<br>
`messages[*] | sort message,asc`<br>

#### Parameters

`messages[*] | sort {column name},{sort mode}`<br>
**Column name:**<br>
any name of a database column in the given table.<br>
**Sort mode:**<br>

- `asc` for ascending
- `dsc` for descending

### where

returns rows which match the given comparison value.

**usage:**<br>
`messages[*] | where message,in,hello world`<br>

#### Parameters

`messages[*] | where {column_name},{operator},{comparison_value}`<br>
**Column name:**<br>
any name of a database column in the given table.<br>
**Operator:**<br>

- `=` for values that equal comparison_value, case sensitive for VarChar
- `>` for values that are more than comparison_value (not usable on VarChar values!)
- `<` for values that are less than comparison_value (not usable on VarChar values!)
- `in` for values that includes the comparison_value (only usable on VarChar values!)

**Comparison value:**<br>
value used for comparison with row value. Should have the same datatype as the
column which is being compared with.

### random

returns n number of random values

**usage:**<br>
`messages[*] | random 3`<br>

#### Parameters

`messages[*] | random {nr_of_random_values}`<br>
**Nr of random values:**<br>
a integer value defining the amount of random values to get.
Should be less than the number of values given to the function.
Or a "\*" for Wildcard which will shuffle the whole vector<br>
**for example:**<br>
`messages[0..3] | random 5`<br>
this query is not valid because you are attempting to retrieve 5 values
out of a vector which is of size 4.
