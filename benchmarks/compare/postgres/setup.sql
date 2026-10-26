-- Tables of the Postgres comparison (benchmarks/compare/postgres/benches/postgres.jai). Run with
-- psql against a scratch database; tools/bench/library_compare.py does it before the benchmarks.
drop table if exists items, wide, sink;

-- Point selects: a 100,000-row table with a primary key.
create table items (
    id int primary key,
    name text not null,
    qty int not null,
    price float8 not null,
    active bool not null,
    note text not null
);

insert into items
select g, 'item-' || g, g % 100, g * 1.5, g % 2 = 0, repeat('x', 20)
from generate_series(1, 100000) as g;

-- Wide rows: 1,000 rows of 24 columns of the kinds both clients decode.
create table wide (
    id int primary key,
    i1 int, i2 int, i3 int,
    b1 bigint, b2 bigint, b3 bigint,
    r1 float4, r2 float4,
    d1 float8, d2 float8, d3 float8,
    f1 bool, f2 bool,
    t1 text, t2 text, t3 text, t4 text, t5 text, t6 text, t7 text, t8 text,
    n1 numeric(12, 2), n2 numeric(12, 2)
);

insert into wide
select g,
       g, g * 2, g * 3,
       g * 1000000007::bigint, g * 3, g * 5,
       g / 7.0, g / 11.0,
       g / 13.0, g / 17.0, g / 19.0,
       g % 2 = 0, g % 3 = 0,
       'alpha-' || g, 'beta-' || g, 'gamma-' || g, 'delta-' || g,
       repeat('e', 12), repeat('z', 24), 'a longer text value for row ' || g, md5(g::text),
       g * 1.25, g * 0.01
from generate_series(1, 1000) as g;

-- The target of the bulk inserts.
create table sink (
    id int not null,
    name text not null,
    qty int not null,
    price float8 not null
);

analyze;
