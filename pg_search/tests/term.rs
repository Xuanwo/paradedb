// Copyright (c) 2023-2024 Retake, Inc.
//
// This file is part of ParadeDB - Postgres for Search and Analytics
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

mod fixtures;

use fixtures::*;
use pretty_assertions::assert_eq;
use rstest::*;
use sqlx::PgConnection;

#[rstest]
fn boolean_term(mut conn: PgConnection) {
    r#"
    CREATE TABLE test_table (
        id SERIAL PRIMARY KEY,
        value BOOLEAN
    );

    INSERT INTO test_table (value) VALUES (true), (false), (false), (true);
    "#
    .execute(&mut conn);

    r#"
    CALL paradedb.create_bm25(
        table_name => 'test_table',
        index_name => 'test_index',
        key_field => 'id',
        boolean_fields => '{"value": {}}'
    );
    "#
    .execute(&mut conn);

    let rows: Vec<(i32, bool)> = r#"
    SELECT * FROM test_index.search(
        query => paradedb.term(field => 'value', value => true),
        stable_sort => true
    );
    "#
    .fetch_collect(&mut conn);
    assert_eq!(rows, vec![(1, true), (4, true)]);
}

#[rstest]
fn integer_term(mut conn: PgConnection) {
    r#"
    CREATE TABLE test_table (
        id SERIAL PRIMARY KEY,
        value_int2 SMALLINT,
        value_int4 INTEGER,
        value_int8 BIGINT
    );

    INSERT INTO test_table (value_int2, value_int4, value_int8) VALUES 
        (-11, -1111, -11111111),
        (22, 2222, 22222222), 
        (33, 3333, 33333333), 
        (44, 4444, 44444444);
    "#
    .execute(&mut conn);

    r#"
    CALL paradedb.create_bm25(
        table_name => 'test_table',
        index_name => 'test_index',
        key_field => 'id',
        numeric_fields => '{"value_int2": {}, "value_int4": {}, "value_int8": {}}'
    );
    "#
    .execute(&mut conn);

    // INT2
    let rows: Vec<(i32, i16)> = r#"
    SELECT id, value_int2 FROM test_index.search(
        query => paradedb.term(field => 'value_int2', value => -11),
        stable_sort => true
    );
    "#
    .fetch_collect(&mut conn);
    assert_eq!(rows, vec![(1, -11)]);

    // INT4
    let rows: Vec<(i32, i32)> = r#"
    SELECT id, value_int4 FROM test_index.search(
        query => paradedb.term(field => 'value_int4', value => 2222),
        stable_sort => true
    );
    "#
    .fetch_collect(&mut conn);
    assert_eq!(rows, vec![(2, 2222)]);

    // INT8
    let rows: Vec<(i32, i64)> = r#"
    SELECT id, value_int8 FROM test_index.search(
        query => paradedb.term(field => 'value_int8', value => 33333333),
        stable_sort => true
    );
    "#
    .fetch_collect(&mut conn);
    assert_eq!(rows, vec![(3, 33333333)]);
}

#[rstest]
fn float_term(mut conn: PgConnection) {
    r#"
    CREATE TABLE test_table (
        id SERIAL PRIMARY KEY,
        value_float4 FLOAT4,
        value_float8 FLOAT8,
        value_numeric NUMERIC
    );

    INSERT INTO test_table (value_float4, value_float8, value_numeric) VALUES
        (-1.1, -1111.1111, -111.11111),
        (2.2, 2222.2222, 222.22222),
        (3.3, 3333.3333, 333.33333),
        (4.4, 4444.4444, 444.44444);
    "#
    .execute(&mut conn);

    r#"
    CALL paradedb.create_bm25(
        table_name => 'test_table',
        index_name => 'test_index',
        key_field => 'id',
        numeric_fields => '{"value_float4": {}, "value_float8": {}, "value_numeric": {}}'
    );
    "#
    .execute(&mut conn);

    // FLOAT4
    let rows: Vec<(i32, f32)> = r#"
    SELECT id, value_float4 FROM test_index.search(
        query => paradedb.term(field => 'value_float4', value => -1.1::float4),
        stable_sort => true
    );
    "#
    .fetch_collect(&mut conn);
    assert_eq!(rows, vec![(1, -1.1)]);

    // FLOAT8
    let rows: Vec<(i32, f64)> = r#"
    SELECT id, value_float8 FROM test_index.search(
        query => paradedb.term(field => 'value_float8', value => 4444.4444::float8),
        stable_sort => true
    );
    "#
    .fetch_collect(&mut conn);
    assert_eq!(rows, vec![(4, 4444.4444)]);

    // NUMERIC - no sqlx::Type for numerics, so just check id
    let rows: Vec<(i32,)> = r#"
    SELECT id FROM test_index.search(
        query => paradedb.term(field => 'value_numeric', value => 333.33333::numeric),
        stable_sort => true
    );
    "#
    .fetch_collect(&mut conn);
    assert_eq!(rows, vec![(3,)]);
}

#[rstest]
fn text_term(mut conn: PgConnection) {
    r#"
    CREATE TABLE test_table (
        id SERIAL PRIMARY KEY,
        value_text TEXT,
        value_varchar VARCHAR(64)
    );

    INSERT INTO test_table (value_text, value_varchar) VALUES 
        ('abc', 'var abc'), 
        ('def', 'var def'), 
        ('ghi', 'var ghi'), 
        ('jkl', 'var jkl');
    "#
    .execute(&mut conn);

    r#"
    CALL paradedb.create_bm25(
        table_name => 'test_table',
        index_name => 'test_index',
        key_field => 'id',
        text_fields => '{"value_text": {}, "value_varchar": {}}'
    );
    "#
    .execute(&mut conn);

    // TEXT
    let rows: Vec<(i32, String)> = r#"
    SELECT id, value_text FROM test_index.search(
        query => paradedb.term(field => 'value_text', value => 'abc'),
        stable_sort => true
    );
    "#
    .fetch_collect(&mut conn);
    assert_eq!(rows, vec![(1, "abc".into())]);

    // VARCHAR
    let rows: Vec<(i32, String)> = r#"
    SELECT id, value_varchar FROM test_index.search(
        query => paradedb.term(field => 'value_varchar', value => 'ghi'),
        stable_sort => true
    );
    "#
    .fetch_collect(&mut conn);
    assert_eq!(rows, vec![(3, "var ghi".into())]);
}

#[rstest]
fn datetime_term(mut conn: PgConnection) {
    r#"
    CREATE TABLE test_table (
        id SERIAL PRIMARY KEY,
        value_date DATE,
        value_timestamp TIMESTAMP,
        value_timestamptz TIMESTAMP WITH TIME ZONE,
        value_time TIME,
        value_timetz TIME WITH TIME ZONE
    );

    INSERT INTO test_table (value_date, value_timestamp, value_timestamptz, value_time, value_timetz) VALUES 
        (DATE '2023-05-03', TIMESTAMP '2023-04-15 13:27:09', TIMESTAMP WITH TIME ZONE '2023-04-15 13:27:09 PST', TIME '08:09:10', TIME WITH TIME ZONE '08:09:10 PST'),
        (DATE '2021-06-28', TIMESTAMP '2019-08-02 07:52:43.123', TIMESTAMP WITH TIME ZONE '2019-08-02 07:52:43.123 EST', TIME '11:43:21.456', TIME WITH TIME ZONE '11:43:21.456 EST');
    "#
    .execute(&mut conn);

    r#"
    CALL paradedb.create_bm25(
        table_name => 'test_table',
        index_name => 'test_index',
        key_field => 'id',
        datetime_fields => '{"value_date": {}, "value_timestamp": {}, "value_timestamptz": {}, "value_time": {}, "value_timetz": {}}'
    );
    "#
    .execute(&mut conn);

    // DATE
    let rows: Vec<(i32,)> = r#"
    SELECT * FROM test_index.search(
        query => paradedb.term(field => 'value_date', value => DATE '2023-05-03')
    );
    "#
    .fetch_collect(&mut conn);
    assert_eq!(rows, vec![(1,)]);

    // TIMESTAMP
    let rows: Vec<(i32,)> = r#"
    SELECT * FROM test_index.search(
        query => paradedb.term(field => 'value_timestamp', value => TIMESTAMP '2019-08-02 07:52:43.123')
    );
    "#
    .fetch_collect(&mut conn);
    assert_eq!(rows, vec![(2,)]);

    // TIMESTAMP WITH TIME ZONE
    let rows: Vec<(i32,)> = r#"
    SELECT * FROM test_index.search(
        query => paradedb.term(field => 'value_timestamptz', value => TIMESTAMP WITH TIME ZONE '2023-04-15 13:27:09 PST')
    );
    "#
    .fetch_collect(&mut conn);
    assert_eq!(rows, vec![(1,)]);

    // TIMESTAMP WITH TIME ZONE: Change time zone in query
    let rows: Vec<(i32,)> = r#"
    SELECT * FROM test_index.search(
        query => paradedb.term(field => 'value_timestamptz', value => TIMESTAMP WITH TIME ZONE '2023-04-15 16:27:09 EST')
    );
    "#
    .fetch_collect(&mut conn);
    assert_eq!(rows, vec![(1,)]);

    // TIME
    let rows: Vec<(i32,)> = r#"
    SELECT * FROM test_index.search(
        query => paradedb.term(field => 'value_time', value => TIME '11:43:21.456')
    );
    "#
    .fetch_collect(&mut conn);
    assert_eq!(rows, vec![(2,)]);

    // TIME WITH TIME ZONE
    let rows: Vec<(i32,)> = r#"
    SELECT * FROM test_index.search(
        query => paradedb.term(field => 'value_timetz', value => TIME WITH TIME ZONE '11:43:21.456 EST')
    );
    "#
    .fetch_collect(&mut conn);
    assert_eq!(rows, vec![(2,)]);

    // TIME WITH TIME ZONE: Change time zone in query
    let rows: Vec<(i32,)> = r#"
    SELECT * FROM test_index.search(
        query => paradedb.term(field => 'value_timetz', value => TIME WITH TIME ZONE '08:43:21.456 PST')
    );
    "#
    .fetch_collect(&mut conn);
    assert_eq!(rows, vec![(2,)]);

    // TIMESTAMP WITH TIME ZONE: Query no time zone with time zone
    let rows: Vec<(i32,)> = r#"
    SELECT * FROM test_index.search(
        query => paradedb.term(field => 'value_timestamp', value => TIMESTAMP WITH TIME ZONE '2023-04-15 13:27:09 GMT')
    );
    "#
    .fetch_collect(&mut conn);
    assert_eq!(rows, vec![(1,)]);

    // TIMESTAMP: Query time zone with no time zone (GMT = EST + 5)
    let rows: Vec<(i32,)> = r#"
    SELECT * FROM test_index.search(
        query => paradedb.term(field => 'value_timestamptz', value => TIMESTAMP '2019-08-02 12:52:43.123')
    );
    "#
    .fetch_collect(&mut conn);
    assert_eq!(rows, vec![(2,)]);
}
