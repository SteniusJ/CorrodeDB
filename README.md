# CorrodeDB

CorrodeDB is a simple "array like" database where queries are optimized for getting data from index positions. It has support for four functions that add vital functionality and improve usability, more info about these functions can be found below in the **Queries** section.

## Compatability

CorrodeDB has been made and tested on mostly Linux machines and I cannot guarantee that it will function on non unix systems. A small amount of testing on Windows has led successfull results but I cannot guarantee any functionality at this point.

## Application Init

Before starting up CorrodeDB you need to define a database schema along with some general database settings.

A commented **example** for how such a **schema** should look like can be found in the **"schema_exmaple.yaml"** file.

### Database schema

The schema does have some restrictions which should be followed:

- **table names** can only have alpha numeric values (for example, these are not valid characters ",\_-/")
- **columns** have no such restrictions
- There are currently three **datatypes**: **NumberI** (Integer values), **NumberF** (Floating point values), **VarChar** (String values)

### General Settings

Currently there are two settings:

- **rows** this setting controls the amount of rows per database "container", a value of 50 will mean that the database will append 50 values into a "container" before creating a new one.
- **password** the database password required for data access. This password is given to the database as a url parameter called "password"

### Application parameters

CorrodeDB has support for application parameters which can be set by the following flags. Order of definition doesn't matter.

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

## Queries

CorrodeDB has a unique query structure and support for four functions write, remove, random, where.

### Data get

The syntax for getting data is:<br>
`{table name}[{indexes}]`

**Single index**<br>
`messages[20]`<br>
this query will get the data at the index **20** in the table called **messages**

**Multi index**<br>
`messages[1,5,10,20,68]`<br>
this query will get the data at the indexes **1,5,10,20,68** in the table called **messages**

**Index range**<br>
`messages[60..100]`<br>
this query will get the data at the indexes **60,61,62... 98,99,100** in the table called **messages**

**Wildcard**<br>
`messages[*]`<br>
this query will get the data at **all indexes** in the table called **messages**

### Data write

the syntax for writing data is:<br>
`{table name}[{index}] write {new data}`

!Data can only be written to one index at a time. The new data is given in a comma seperated string. Commas can be escaped in VarChar values by using the "\\" escape character.

**Writing to specific index**<br>
`messages[10] write hello\, world,100,12.239`<br>
this query will write the data **"hello, world" | 100 | 12.239** to the index **10**

**Appending to database**<br>
`messages[*] write hello\, world,100,12.239`<br>
this query will write the data **"hello, world" | 100 | 12.239** to the last available index.

### Data remove

the syntax for removing data is:<br>
`{table name}[{index}] remove`

!Data can only be removed from one index at a time.

**Example**<br>
`messages[10] remove`<br>
this query will remove the data at index **10**

### Random data get

The syntax for getting random data:<br>
`{table name}[{indexes}] random {number of random elements}`

The **random** function will get a given number of random elements from the select indexes. All the indexing types used in **data get** can be used with the **random** function.

**Example**<br>
`messages[*] random 5`<br>
this query will get **5** random values out of all messages.

### Get where

the syntax for getting data "where" is:<br>
`{table name}[{indexes}] where {column name},{operator},{matching value}`

The **where** function will get all data that matches the given conditions. All indexing types used in **data get** can be used with the **where** function

**= operator**<br>
`messages[*] where number,=,10`<br>
this query will get all the rows where the column **number** is 10

**> operator**<br>
`messages[10..50] where number,>,10`<br>
this query will get all the rows in the range **10 to (including) 50** where the column **number** is more than 10<br>
**note!** this operator can only be used with Number values

**< operator**<br>
`messages[1,20,36,37,49,90] where number,<,20`<br>
this query will get all the rows from the indexes **1,20,36,37,49,90** where the column **number** is less than 20<br>
**note!** this operator can only be used with Number values
