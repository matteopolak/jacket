-- The schema the typed-query tests describe their statements against (run by run.sh).
drop schema public cascade;
create schema public;

create type mood as enum ('sad', 'ok', 'happy');

create table users (
    id bigserial primary key,
    email text not null,
    display_name text,
    created_at timestamptz not null default now(),
    settings jsonb,
    mood mood not null default 'ok',
    score int4 not null default 0,
    tags text[] not null default '{}'
);

create table posts (
    id bigserial primary key,
    author_id bigint not null references users (id),
    title text not null,
    body text,
    created_at timestamptz not null default now()
);

-- A key on two columns, for loading relations that join on both.
create table teams (
    org text not null,
    code int4 not null,
    name text not null,
    primary key (org, code)
);

create table members (
    id bigserial primary key,
    org text not null,
    team_code int4,
    name text not null,
    foreign key (org, team_code) references teams (org, code)
);

create view user_posts as
    select u.id as user_id, u.email, p.id as post_id
    from users u join posts p on p.author_id = u.id;

insert into users (email, display_name, settings, tags, mood)
values ('a@example.com', 'Alice', '{"theme": "dark", "font_size": 16}', '{x,y}', 'happy'),
       ('b@example.com', null, null, default, default);

insert into posts (author_id, title, body) values (1, 'hello', 'first'), (1, 'second', null);

insert into teams values ('acme', 1, 'red'), ('acme', 2, 'blue'), ('beta', 1, 'green');
insert into members (org, team_code, name)
values ('acme', 1, 'ann'), ('beta', 1, 'bob'), ('acme', 1, 'cy'), ('acme', null, 'dee');

-- A table the statement-cache tests alter under a running program.
create table drift (a int8 not null, b int8 not null);
insert into drift values (1, 2);
