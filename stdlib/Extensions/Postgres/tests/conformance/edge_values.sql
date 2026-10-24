-- Awkward values for the binary round trip (roundtrip.jai): the kinds of value the type tests of
-- pgx (pgtype), psycopg and npgsql exercise, written for this suite. Run in the schema
-- edge_values; works on PostgreSQL 15 and later.

drop schema if exists edge_values cascade;
create schema edge_values;
set search_path = edge_values, pg_catalog;

create table bools (v bool);
insert into bools values (true), (false), (null);

create table int2s (v int2);
insert into int2s values (-32768), (-256), (-1), (0), (1), (255), (256), (32767), (null);

create table int4s (v int4);
insert into int4s values (-2147483648), (-65536), (-1), (0), (1), (65535), (16777216), (2147483647), (null);

create table int8s (v int8);
insert into int8s values
    (-9223372036854775808), (-4294967296), (-2147483649), (-1), (0), (1), (2147483648),
    (4294967295), (4294967296), (9007199254740993), (9223372036854775807), (null);

create table oids (v oid);
insert into oids values (0), (1), (16384), (2147483647), (2147483648), (4294967295), (null);

create table chars (v "char");
insert into chars values ('a'), (' '), ('~'), ('\001'), ('\\'), (''), ('"'), (null);

create table names (v name);
insert into names values
    (''), ('a'), ('with "quotes" and \ backslash'), (repeat('n', 63)), (repeat('é', 40)),
    ('{braces,and,commas}'), (' padded '), (null);

create table texts (v text);
insert into texts values
    (''), (' '), ('  leading and trailing  '), ('héllo wörld'), (E'tab\there'), (E'new\nline\r\n'),
    ('back\slash and ''quote'' and "double"'), ('{1,2,3}'), ('NULL'), ('\x00ff'),
    (chr(1)), (chr(127)), (chr(128)), (chr(55295)), (chr(57344)), (chr(65533)), (chr(65535)),
    (chr(1114111)), ('日本語のテキスト'), ('😀 emoji'), (E'é combining'),
    (repeat('x', 100000)), (repeat('é', 40000)), (null);

create table varchars (v varchar(10));
insert into varchars values (''), ('abc'), ('0123456789'), ('éééééééééé'), (null);

create table bpchars (v char(6));
insert into bpchars values (''), ('a'), ('abc   '), ('éé'), ('123456'), (null);

create table byteas (v bytea);
insert into byteas values
    (''), ('\x00'), ('\xff'), ('\x00ff00'), ('\x5c'), ('\x27'),
    ((select decode(string_agg(lpad(to_hex(i), 2, '0'), ''), 'hex') from generate_series(0, 255) i)),
    ((select decode(string_agg(lpad(to_hex(i), 2, '0'), ''), 'hex') from generate_series(255, 0, -1) i)),
    (decode(repeat('ab', 70000), 'hex')), (null);

create table uuids (v uuid);
insert into uuids values
    ('00000000-0000-0000-0000-000000000000'), ('ffffffff-ffff-ffff-ffff-ffffffffffff'),
    ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11'), ('A0EEBC99-9C0B-4EF8-BB6D-6BB9BD380A11'),
    ('{a0eebc999c0b4ef8bb6d6bb9bd380a11}'), ('a0ee-bc99-9c0b-4ef8-bb6d-6bb9-bd38-0a11'),
    ('01234567-89ab-cdef-0123-456789abcdef'), (null);

create table float4s (v float4);
insert into float4s values
    (0), ('-0'), (1), (-1), (0.1), (1.5), (16777216), (16777217), (123456.79), (1e-10),
    (3.4028235e38), (-3.4028235e38), (1.17549435e-38), (1.4e-45), (-1.4e-45), (3.14159274),
    ('NaN'), ('Infinity'), ('-Infinity'), (null);

create table float8s (v float8);
insert into float8s values
    (0), ('-0'), (1), (-1), (0.1), (0.3), (1e23), (9007199254740992), (9007199254740993),
    (123456789012345.67), (2.5e-10), (1e308), (1.7976931348623157e308), (-1.7976931348623157e308),
    (2.2250738585072014e-308), (4.9e-324), (-4.9e-324), (8.41e21), (5e-324), (0.30000000000000004),
    ('NaN'), ('Infinity'), ('-Infinity'), (null);

create table numerics (v numeric);
insert into numerics values
    (0), (0.0), (0.000), ('-0'), ('-0.00'), (1), (-1), (0.1), (0.5), (-0.5), (0.0001), (0.00001),
    (0.00000001), (0.000000000000000000000001), (1.10), (100.00), (10000), (9999.9999),
    (10000.0001), (99999999), (100000000), (100000001), (12345678901234567890.12345678901234567890),
    (-12345678901234567890.1234500), (1e-100), (1e100), (-1e100), (1.5e-100), (0.00009999),
    (123456789.987654321), (round(123456789::numeric, -3)), (trunc(98765.4321, -2)),
    (round(0.000123456, 4)), (1e131071), (('0.' || repeat('0', 16382) || '1')::numeric),
    (('1' || repeat('0', 4000) || '.' || repeat('9', 4000))::numeric),
    ('NaN'), ('Infinity'), ('-Infinity'), (null);

create table scaled_numerics (a numeric(10, 2), b numeric(5, -3), c numeric(3, 5), d numeric(1000, 500));
insert into scaled_numerics values
    (1.10, 12000, 0.00123, 1.5), (-99999999.99, -99999000, 0.00001, 0), (0, 0, 0, 0),
    (100, 1000, 0.0, -1e400), (null, null, null, null);

create table dates (v date);
insert into dates values
    ('infinity'), ('-infinity'), ('4713-01-01 BC'), ('0001-12-31 BC'), ('0001-01-01'),
    ('0044-03-15 BC'), ('1582-10-04'), ('1582-10-15'), ('1900-01-01'), ('1969-12-31'),
    ('1970-01-01'), ('1999-12-31'), ('2000-01-01'), ('2000-01-02'), ('2000-02-29'),
    ('2024-02-29'), ('2100-02-28'), ('9999-12-31'), ('10000-01-01'), ('294276-12-31'),
    ('5874897-12-31'), (null);

create table times (v time);
insert into times values
    ('00:00:00'), ('00:00:00.000001'), ('01:02:03'), ('04:05:06.789'), ('12:34:56.789012'),
    ('23:59:59'), ('23:59:59.999999'), ('24:00:00'), ('12:00:00.5'), (null);

create table timestamps (v timestamp);
insert into timestamps values
    ('infinity'), ('-infinity'), ('4713-01-01 00:00:00 BC'), ('0001-01-01 00:00:00'),
    ('0001-12-31 23:59:59.999999 BC'), ('0044-03-15 12:00:00 BC'), ('1582-10-04 12:00:00'),
    ('1969-12-31 23:59:59.999999'), ('1970-01-01 00:00:00'), ('1999-12-31 23:59:59.999999'),
    ('2000-01-01 00:00:00'), ('2000-01-01 00:00:00.000001'), ('2000-01-01 24:00:00'),
    ('2024-02-29 12:00:00.5'), ('2024-02-29 23:59:59.123456'), ('9999-12-31 23:59:59.999999'),
    ('10000-01-01 00:00:00'), ('294276-12-31 23:59:59.999999'), (null);

create table timestamptzs (v timestamptz);
insert into timestamptzs values
    ('infinity'), ('-infinity'), ('4713-01-01 00:00:00+00 BC'), ('0001-01-01 00:00:00+00'),
    ('0001-12-31 23:59:59.999999+00 BC'), ('0044-03-15 12:00:00+00 BC'),
    ('1969-12-31 23:59:59.999999+00'), ('1970-01-01 00:00:00+00'), ('1999-12-31 23:59:59.999999+00'),
    ('2000-01-01 00:00:00+00'), ('2000-01-01 00:00:00.000001+00'), ('2024-02-29 12:00:00.5+05:30'),
    ('2024-03-31 00:30:00+00'), ('2024-03-31 01:30:00+00'), ('2024-10-27 00:30:00+00'),
    ('2024-10-27 01:30:00+00'), ('2024-02-29 23:59:59.123456-08'), ('9999-12-31 23:59:59.999999+00'),
    ('10000-01-01 00:00:00+00'), ('294276-12-30 12:00:00+00'), (null);

create table intervals (v interval);
insert into intervals values
    ('0'), ('1 microsecond'), ('-1 microsecond'), ('1 year 2 mons 3 days 04:05:06.789012'),
    ('-1 year -2 mons -3 days -04:05:06.789012'), ('1 mon -1 day'), ('-1 mon 1 day'),
    ('1 mon -1 day +00:00:01'), ('1 day -00:00:00.000001'), ('-1 day 00:00:00.000001'),
    ('30 days'), ('1 mon'), ('12 mons'), ('100 years'), ('-100 years'),
    ('178956970 years 7 mons'), ('-178956970 years -8 mons'),
    ('2147483647 days'), ('-2147483648 days'),
    ('2562047788:00:54.775807'), ('-2562047788:00:54.775807'),
    ('2147483647 mons 2147483647 days 2562047788:00:54.775807'),
    ('-2147483648 mons -2147483648 days -2562047788:00:54.775807'),
    ('00:00:00.000001'), ('-00:00:00.000001'), ('23:59:59.999999'), ('100:00:00'), (null);

-- Interval infinities exist from PostgreSQL 17.
do $$
begin
    if current_setting('server_version_num')::int >= 170000 then
        insert into intervals values ('infinity'), ('-infinity');
    end if;
end
$$;

create table bool_arrays (v bool[]);
insert into bool_arrays values ('{}'), ('{t}'), ('{t,f,NULL}'), ('{NULL}'), (null);

create table int2_arrays (v int2[]);
insert into int2_arrays values ('{}'), ('{-32768,0,32767}'), ('{NULL,1}'), ('{{1,2},{3,4}}'), (null);

create table int4_arrays (v int4[]);
insert into int4_arrays values
    ('{}'), ('{1}'), ('{1,2,3}'), ('{NULL}'), ('{NULL,NULL}'), ('{1,NULL,3}'),
    ('{-2147483648,2147483647}'), ('[0:1]={1,2}'), ('[-1:1]={1,2,3}'), ('[5:5]={7}'),
    ('[2:3]={NULL,4}'), ('{{1,2},{3,4}}'), ('[0:1][0:1]={{1,2},{3,4}}'),
    ((select array_agg(i) from generate_series(1, 5000) i)), (null);

create table int8_arrays (v int8[]);
insert into int8_arrays values ('{}'), ('{-9223372036854775808,0,9223372036854775807}'), ('{NULL,1}'), (null);

create table oid_arrays (v oid[]);
insert into oid_arrays values ('{}'), ('{0,4294967295}'), ('{NULL,1}'), (null);

create table char_arrays (v "char"[]);
insert into char_arrays values ('{}'), ('{a,b,NULL}'), ('{" ","\""}'), (null);

create table name_arrays (v name[]);
insert into name_arrays values ('{}'), ('{a,"b c",NULL}'), ('{""}'), (null);

create table text_arrays (v text[]);
insert into text_arrays values
    ('{}'), ('{""}'), ('{"",""}'), ('{NULL}'), ('{"NULL"}'), ('{a,b,c}'), ('{"a,b","{}","\"q\"","back\\slash"}'),
    ('{" lead","trail "," "}'), ('{"héllo","😀","日本"}'), ('{NULL,"",NULL}'),
    ('[0:2]={a,b,c}'), ('{{a,b},{c,d}}'), (array[E'tab\t', E'nl\n', 'x y', '{', '}', ',', '\']),
    ((select array_agg(repeat('é', i)) from generate_series(1, 200) i)), (null);

create table bpchar_arrays (v char(4)[]);
insert into bpchar_arrays values ('{}'), ('{a,"b  ",NULL}'), (null);

create table varchar_arrays (v varchar(5)[]);
insert into varchar_arrays values ('{}'), ('{a,"",NULL}'), ('{"é","12345"}'), (null);

create table bytea_arrays (v bytea[]);
insert into bytea_arrays values
    ('{}'), ('{"\\x00","\\xff"}'), ('{NULL}'), ('{"\\x","\\x5c"}'),
    (array[decode('00ff', 'hex'), '\x'::bytea, null]), (null);

create table float4_arrays (v float4[]);
insert into float4_arrays values ('{}'), ('{NaN,Infinity,-Infinity,-0,1.5}'), ('{NULL}'), (null);

create table float8_arrays (v float8[]);
insert into float8_arrays values
    ('{}'), ('{NaN,Infinity,-Infinity,-0,1e308,5e-324}'), ('{NULL,0.1}'), ('{{1,2},{3,4}}'), (null);

create table numeric_arrays (v numeric[]);
insert into numeric_arrays values
    ('{}'), ('{0,0.0,-0.5,1.50,NaN,Infinity,-Infinity}'), ('{1e100,-1e-100,12345678901234567890.12345}'),
    ('{NULL,1}'), ('{10000,0.0001,9999.9999}'), (null);

create table uuid_arrays (v uuid[]);
insert into uuid_arrays values
    ('{}'), ('{00000000-0000-0000-0000-000000000000,ffffffff-ffff-ffff-ffff-ffffffffffff,NULL}'), (null);

create table date_arrays (v date[]);
insert into date_arrays values
    ('{}'), ('{infinity,-infinity,"4713-01-01 BC",2000-01-01,5874897-12-31}'), ('{NULL}'), (null);

create table time_arrays (v time[]);
insert into time_arrays values ('{}'), ('{00:00:00,24:00:00,23:59:59.999999,NULL}'), (null);

create table timestamp_arrays (v timestamp[]);
insert into timestamp_arrays values
    ('{}'), ('{infinity,-infinity,"4713-01-01 00:00:00 BC","2000-01-01 00:00:00.000001",NULL}'),
    ('{"294276-12-31 23:59:59.999999"}'), (null);

create table timestamptz_arrays (v timestamptz[]);
insert into timestamptz_arrays values
    ('{}'), ('{infinity,-infinity,"0001-01-01 00:00:00+00","2000-01-01 00:00:00+00",NULL}'), (null);

create table interval_arrays (v interval[]);
insert into interval_arrays values
    ('{}'), ('{"1 year 2 mons","-3 days","04:05:06.789012",NULL,"-178956970 years -8 mons"}'), (null);

create table json_scalars (v json);
insert into json_scalars values
    ('null'), ('true'), ('false'), ('0'), ('-0'), ('1.0'), ('1e5'), ('-1.5E-10'), ('12345678901234567890123'),
    ('""'), ('"a"'), ('"é😀"'), ('"\u0000"'), ('"\\ \/ \b\f\n\r\t"'), ('"é😀日本"'),
    ('[]'), ('{}'), (' [ 1 , 2 ] '), ('{"a":1,"a":2}'), ('{"":1}'), ('{"é":{"😀":[null,true]}}'),
    ((repeat('[', 200) || repeat(']', 200))::json), ('{"a":[1,{"b":[2,[3]]}],"c":null}'), (null);

create table jsonb_scalars (v jsonb);
insert into jsonb_scalars values
    ('null'), ('true'), ('false'), ('0'), ('-0'), ('1.0'), ('1e5'), ('-1.5E-10'), ('12345678901234567890123'),
    ('1e400'), ('""'), ('"a"'), ('"é😀"'), ('"\\ \/ \b\f\n\r\t"'), ('"é😀日本"'),
    ('[]'), ('{}'), (' [ 1 , 2 ] '), ('{"a":1,"a":2}'), ('{"":1}'), ('{"é":{"😀":[null,true]}}'),
    ((repeat('[', 200) || repeat(']', 200))::jsonb), ('{"a":[1,{"b":[2,[3]]}],"c":null}'),
    ((select jsonb_object_agg('k' || i, i) from generate_series(1, 500) i)), (null);

create table json_arrays (v json[]);
insert into json_arrays values ('{}'), (array['{"a": 1}'::json, 'null'::json, null]), (null);

create table jsonb_arrays (v jsonb[]);
insert into jsonb_arrays values ('{}'), (array['{"a": 1}'::jsonb, 'null'::jsonb, null]), (null);

create domain positive_int as int4 check (value > 0);
create domain label as varchar(8);
create type mood as enum ('sad', 'ok', 'happy');
create table domains (a positive_int, b label, c positive_int[], d mood);
insert into domains values (1, 'x', '{1,2}', 'ok'), (2147483647, '12345678', '{}', 'happy'), (null, null, null, null);
