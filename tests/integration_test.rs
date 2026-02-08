use corrode_db::single_query;

const TEST_SCHEMA_PATH: &str = "test_schema.yaml";

#[test]
fn test_queries() {
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
}
