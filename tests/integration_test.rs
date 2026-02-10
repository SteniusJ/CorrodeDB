use corrode_db::single_query;
use rand::prelude::*;

const TEST_SCHEMA_PATH: &str = "test_schema.yaml";

#[test]
fn test_queries() {
    let mut rng = rand::rng();
    /*
     * Test incorrect default inputs
     */
    // Test incorrect table name
    single_query(TEST_SCHEMA_PATH, "nosuchtable[*]").expect_err("returned ok when no such table exists");
    // Test incorrect index
    single_query(TEST_SCHEMA_PATH, "read[p]").expect_err("returned ok when index type was incorrect");
    single_query(TEST_SCHEMA_PATH, "read[-1]").expect_err("returned ok when index type was incorrect");
    single_query(TEST_SCHEMA_PATH, "read[10]").expect_err("returned ok when indexing outside table size");
    // Test incorrect function name
    single_query(TEST_SCHEMA_PATH, "read[*] nofunc").expect_err("returned ok when no such function exists");

    /*
     * Test incorrect function inputs
     */
    // Test where 
    single_query(TEST_SCHEMA_PATH, "read[*] where nosuchcol,=,10").expect_err("where returned ok when no such column exists");
    single_query(TEST_SCHEMA_PATH, "read[*] where int,in,hello world").expect_err("where returned ok when given non matching operator value pair");
    // single_query(TEST_SCHEMA_PATH, "read[*] where int,=,hello world").expect_err("where returned ok when given non matching column value and comparison value");
    single_query(TEST_SCHEMA_PATH, "read[*] where int,>").expect_err("where returned ok when given incorrect number of parameters");
    // Test random
    single_query(TEST_SCHEMA_PATH, "read[*] random 1,2").expect_err("random returned ok when given incorrect number of parameters");
    single_query(TEST_SCHEMA_PATH, "read[*] random abc").expect_err("random returned ok when given incorrect param");
    // Test write
    single_query(TEST_SCHEMA_PATH, "write[*] write abc,123,1.0").expect_err("write returned ok when given incorrect parameter types");
    single_query(TEST_SCHEMA_PATH, "write[*] write 1,2").expect_err("write returned ok when given incorrect number of parameters");
    single_query(TEST_SCHEMA_PATH, "write[*] write 1,1.2,hello world,extra thing").expect_err("write returned ok when given incorrect number of parameters");
    single_query(TEST_SCHEMA_PATH, "write[1,2,3] write 1,1.2,hello wolrd").expect_err("write returned ok when given incorrect index"); // multi index write
    single_query(TEST_SCHEMA_PATH, "write[1..5] write 1,1.2,hello wolrd").expect_err("write returned ok when given incorrect index"); // multi index write
    // Test remove
    single_query(TEST_SCHEMA_PATH, "write[*] remove").expect_err("remove returned ok when given incorrect index"); // multi index remove
    single_query(TEST_SCHEMA_PATH, "write[1,2,3] remove").expect_err("remove returned ok when given incorrect index"); // multi index remove
    single_query(TEST_SCHEMA_PATH, "write[1..5] remove").expect_err("remove returned ok when given incorrect index"); // multi index remove


    /*
     * Test functions
     */
    // Test read
    assert_eq!(single_query(TEST_SCHEMA_PATH, "read[*]").unwrap(), single_query(TEST_SCHEMA_PATH, "read[0..6]").unwrap());
    let result = single_query(TEST_SCHEMA_PATH, "read[1,4,6,3]").unwrap();
    assert_eq!(result.len(), 4);
    assert_eq!(result[2].get("string").unwrap().as_string().unwrap(), String::from("j this is row 6"));
    let result = single_query(TEST_SCHEMA_PATH, "read[3..6]").unwrap();
    assert_eq!(result.len(), 4);
    assert_eq!(result[0].get("int").unwrap().as_i64().unwrap(), 3);
    // Test read sort
    assert_eq!(single_query(TEST_SCHEMA_PATH, "read[*] | sort float,asc").unwrap()[2].get("float").unwrap().as_f64().unwrap(), 2.1);
    assert_eq!(single_query(TEST_SCHEMA_PATH, "read[*] | sort int,dsc").unwrap()[4].get("int").unwrap().as_i64().unwrap(), 2);

    // Test write
    let write_data = format!("0x{:X}", rng.random::<u128>());
    single_query(TEST_SCHEMA_PATH, &format!("write[0] write 1,1.2,{write_data}")).expect_err("not reachable");
    assert_eq!(single_query(TEST_SCHEMA_PATH, "write[0]").unwrap()[0].get("string").unwrap().as_string().unwrap(), write_data);

    // Test random
    let result = single_query(TEST_SCHEMA_PATH, "read[2..6] random 2").unwrap();
    assert_eq!(result.len(), 2);

    // Test where
    assert_eq!(single_query(TEST_SCHEMA_PATH, "read[*] where int,<,3").unwrap()[2].get("int").unwrap().as_i64().unwrap(), 2);
    assert_eq!(single_query(TEST_SCHEMA_PATH, "read[*] where string,=,d this is row 3").unwrap()[0].get("int").unwrap().as_i64().unwrap(), 3);
    assert_eq!(single_query(TEST_SCHEMA_PATH, "read[*] where string,in,j").unwrap()[0].get("int").unwrap().as_i64().unwrap(), 6);

    // Test remove
    single_query(TEST_SCHEMA_PATH, "write[0] remove").expect_err("unreachable");
    assert_eq!(single_query(TEST_SCHEMA_PATH, "write[0]").unwrap().is_empty(), true);
}
