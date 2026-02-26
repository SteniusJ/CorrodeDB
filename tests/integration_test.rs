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
    single_query(TEST_SCHEMA_PATH, "read[*] | where nosuchcol,=,10").expect_err("where returned ok when no such column exists");
    single_query(TEST_SCHEMA_PATH, "read[*] | where int,in,hello world").expect_err("where returned ok when given non matching operator value pair");
    // single_query(TEST_SCHEMA_PATH, "read[*] where int,=,hello world").expect_err("where returned ok when given non matching column value and comparison value");
    single_query(TEST_SCHEMA_PATH, "read[*] | where int,>").expect_err("where returned ok when given incorrect number of parameters");
    // Test random
    single_query(TEST_SCHEMA_PATH, "read[*] | random 1,2").expect_err("random returned ok when given incorrect number of parameters");
    single_query(TEST_SCHEMA_PATH, "read[*] | random abc").expect_err("random returned ok when given incorrect param");
    // Test write
    single_query(TEST_SCHEMA_PATH, "write[*] write abc,123,1.0").expect_err("write returned ok when given incorrect parameter types");
    single_query(TEST_SCHEMA_PATH, "write[*] write 1,2").expect_err("write returned ok when given incorrect number of parameters");
    single_query(TEST_SCHEMA_PATH, "write[*] write 1,1.2,hello world,extra thing").expect_err("write returned ok when given incorrect number of parameters");

    /*
     * Test functions
     */
    // Test read
    assert_eq!(single_query(TEST_SCHEMA_PATH, "read[*]").into_vec(), single_query(TEST_SCHEMA_PATH, "read[0..6]").into_vec());
    let result = single_query(TEST_SCHEMA_PATH, "read[1,4,6,3]").into_vec().unwrap();
    assert_eq!(result.len(), 4);
    assert_eq!(result[2].get("string").unwrap().as_string().unwrap(), String::from("j this is row 6"));
    let result = single_query(TEST_SCHEMA_PATH, "read[3..6]").into_vec().unwrap();
    assert_eq!(result.len(), 4);
    assert_eq!(result[0].get("int").unwrap().as_i64().unwrap(), 3);
    assert_eq!(single_query(TEST_SCHEMA_PATH, "read[*-1]").into_vec().unwrap()[0].get("int").unwrap().as_i64().unwrap(), 5);
    assert_eq!(single_query(TEST_SCHEMA_PATH, "read[0..*-3]").into_vec().unwrap()[3].get("int").unwrap().as_i64().unwrap(), 3);
    // Test read sort
    assert_eq!(single_query(TEST_SCHEMA_PATH, "read[*] | sort float,asc").into_vec().unwrap()[2].get("float").unwrap().as_f64().unwrap(), 2.1);
    assert_eq!(single_query(TEST_SCHEMA_PATH, "read[*] | sort int,dsc").into_vec().unwrap()[4].get("int").unwrap().as_i64().unwrap(), 2);
    // Test read with stacked sub functions
    let result = single_query(TEST_SCHEMA_PATH, "read[*] | sort int,dsc | where int,>,2 | random 3 | sort string,asc | random 1").into_vec().unwrap()[0].get("int").unwrap().as_i64().unwrap();
    assert!(result > 2, "result: {result} was less than 2 even when that shouldn't be possible");

    // Test write
    /*
     * setup
     */
    for _ in 0..6 {
        single_query(TEST_SCHEMA_PATH, "write[*] write 1,1.2,\"hello wolrd\"");
    }

    let write_data = format!("0x{:X}", rng.random::<u128>());
    single_query(TEST_SCHEMA_PATH, &format!("write[0] write 1,1.2,{write_data}")).into_tuple().unwrap();
    assert_eq!(single_query(TEST_SCHEMA_PATH, "write[0]").into_vec().unwrap()[0].get("string").unwrap().as_string().unwrap(), write_data);
    single_query(TEST_SCHEMA_PATH, "write[1,2,3] write 1,1.2,hello wolrd");
    single_query(TEST_SCHEMA_PATH, "write[1..5] write 1,1.2,hello wolrd");

    // Test random
    let result = single_query(TEST_SCHEMA_PATH, "read[2..6] | random 2").into_vec().unwrap();
    assert_eq!(result.len(), 2);

    // Test where
    assert_eq!(single_query(TEST_SCHEMA_PATH, "read[*] | where int,<,3").into_vec().unwrap()[2].get("int").unwrap().as_i64().unwrap(), 2);
    assert_eq!(single_query(TEST_SCHEMA_PATH, "read[*] | where string,=,\"d this is row 3\"").into_vec().unwrap()[0].get("int").unwrap().as_i64().unwrap(), 3);
    assert_eq!(single_query(TEST_SCHEMA_PATH, "read[*] | where string,in,j").into_vec().unwrap()[0].get("int").unwrap().as_i64().unwrap(), 6);

    // Test remove
    single_query(TEST_SCHEMA_PATH, "write[0] remove").into_tuple().unwrap();
    assert_eq!(single_query(TEST_SCHEMA_PATH, "write[0]").into_vec().unwrap().is_empty(), true);
    single_query(TEST_SCHEMA_PATH, "write[1..3] remove").into_tuple().unwrap();
    single_query(TEST_SCHEMA_PATH, "write[4,5] remove").into_tuple().unwrap();
    single_query(TEST_SCHEMA_PATH, "write[*] remove").into_tuple().unwrap();
}
